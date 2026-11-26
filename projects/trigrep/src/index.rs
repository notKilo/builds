//! Trigram index: maps every 3-byte (ASCII-lowercased) window to the files containing it.

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use crate::walk::FileEntry;

pub type Trigram = [u8; 3];

const MAGIC: &[u8; 5] = b"TGRP1";

/// All distinct trigrams of `data`, ASCII-lowercased so one index serves
/// both case-sensitive and case-insensitive queries.
pub fn trigrams(data: &[u8]) -> BTreeSet<Trigram> {
    data.windows(3)
        .map(|w| [w[0].to_ascii_lowercase(), w[1].to_ascii_lowercase(), w[2].to_ascii_lowercase()])
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Index {
    pub files: Vec<FileEntry>,
    /// Sorted file ids per trigram.
    pub postings: BTreeMap<Trigram, Vec<u32>>,
}

impl Index {
    /// Add a file and its content; returns the new file id.
    pub fn add(&mut self, entry: FileEntry, content: &[u8]) -> u32 {
        let id = self.files.len() as u32;
        self.files.push(entry);
        for t in trigrams(content) {
            self.postings.entry(t).or_default().push(id);
        }
        id
    }

    /// Drop the files whose `keep` flag is false and renumber the rest.
    pub fn retain_files(&mut self, keep: &[bool]) {
        let mut remap = vec![u32::MAX; self.files.len()];
        let mut next = 0u32;
        for (old, &k) in keep.iter().enumerate() {
            if k {
                remap[old] = next;
                next += 1;
            }
        }
        let mut i = 0;
        self.files.retain(|_| {
            i += 1;
            keep[i - 1]
        });
        self.postings.retain(|_, ids| {
            ids.retain(|&id| remap[id as usize] != u32::MAX);
            for id in ids.iter_mut() {
                *id = remap[*id as usize];
            }
            !ids.is_empty()
        });
    }

    /// Files containing every trigram in `required` (all files when empty).
    pub fn candidates(&self, required: &BTreeSet<Trigram>) -> Vec<u32> {
        let mut lists: Vec<&Vec<u32>> = Vec::new();
        for t in required {
            match self.postings.get(t) {
                Some(l) => lists.push(l),
                None => return Vec::new(),
            }
        }
        if lists.is_empty() {
            return (0..self.files.len() as u32).collect();
        }
        lists.sort_by_key(|l| l.len());
        let mut result = lists[0].clone();
        for l in &lists[1..] {
            result = intersect(&result, l);
            if result.is_empty() {
                break;
            }
        }
        result
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        put_varint(&mut out, self.files.len() as u64);
        for f in &self.files {
            put_varint(&mut out, f.rel_path.len() as u64);
            out.extend_from_slice(f.rel_path.as_bytes());
            put_varint(&mut out, f.size);
            put_varint(&mut out, f.mtime_secs);
        }
        put_varint(&mut out, self.postings.len() as u64);
        for (t, ids) in &self.postings {
            out.extend_from_slice(t);
            put_varint(&mut out, ids.len() as u64);
            let mut prev = 0u32;
            for &id in ids {
                put_varint(&mut out, u64::from(id - prev));
                prev = id;
            }
        }
        out
    }

    pub fn from_bytes(data: &[u8]) -> io::Result<Index> {
        let mut r = Reader { data, pos: 0 };
        if r.take(MAGIC.len())? != MAGIC {
            return Err(bad("not a trigrep index"));
        }
        let nfiles = r.varint()? as usize;
        let mut files = Vec::with_capacity(nfiles.min(1 << 20));
        for _ in 0..nfiles {
            let len = r.varint()? as usize;
            let rel_path = String::from_utf8(r.take(len)?.to_vec()).map_err(|_| bad("bad path"))?;
            let size = r.varint()?;
            let mtime_secs = r.varint()?;
            files.push(FileEntry { rel_path, size, mtime_secs });
        }
        let ntri = r.varint()? as usize;
        let mut postings = BTreeMap::new();
        for _ in 0..ntri {
            let t: Trigram = r.take(3)?.try_into().unwrap();
            let n = r.varint()? as usize;
            let mut ids = Vec::with_capacity(n.min(1 << 20));
            let mut prev = 0u32;
            for _ in 0..n {
                prev = prev.checked_add(r.varint()? as u32).ok_or_else(|| bad("id overflow"))?;
                if prev as usize >= nfiles {
                    return Err(bad("file id out of range"));
                }
                ids.push(prev);
            }
            postings.insert(t, ids);
        }
        Ok(Index { files, postings })
    }
}

fn intersect(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                out.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    out
}

fn bad(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8 & 0x7f) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.data.len()).ok_or_else(|| bad("truncated index"))?;
        let s = &self.data[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn varint(&mut self) -> io::Result<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.take(1)?[0];
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(bad("varint too long"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(p: &str) -> FileEntry {
        FileEntry { rel_path: p.into(), size: 1, mtime_secs: 2 }
    }

    fn sample() -> Index {
        let mut idx = Index::default();
        idx.add(entry("a.txt"), b"hello world");
        idx.add(entry("b.txt"), b"HELLO there");
        idx.add(entry("c.txt"), b"nothing here");
        idx
    }

    #[test]
    fn trigrams_are_lowercased_and_distinct() {
        let t = trigrams(b"AbcABC");
        assert_eq!(t.len(), 3); // abc, bca, cab
        assert!(t.contains(b"abc"));
        assert!(trigrams(b"ab").is_empty());
    }

    #[test]
    fn candidates_intersect_postings() {
        let idx = sample();
        assert_eq!(idx.candidates(&trigrams(b"hello")), vec![0, 1]);
        assert_eq!(idx.candidates(&trigrams(b"world")), vec![0]);
        assert_eq!(idx.candidates(&trigrams(b"zzz")), Vec::<u32>::new());
        assert_eq!(idx.candidates(&BTreeSet::new()), vec![0, 1, 2]);
    }

    #[test]
    fn retain_files_renumbers() {
        let mut idx = sample();
        idx.retain_files(&[true, false, true]);
        assert_eq!(idx.files.len(), 2);
        assert_eq!(idx.files[1].rel_path, "c.txt");
        assert_eq!(idx.candidates(&trigrams(b"hello")), vec![0]);
        assert_eq!(idx.candidates(&trigrams(b"nothing")), vec![1]);
    }

    #[test]
    fn roundtrip() {
        let idx = sample();
        assert_eq!(Index::from_bytes(&idx.to_bytes()).unwrap(), idx);
    }

    #[test]
    fn rejects_corrupt_input() {
        let bytes = sample().to_bytes();
        assert!(Index::from_bytes(b"nope").is_err());
        for cut in [6, bytes.len() / 2, bytes.len() - 1] {
            assert!(Index::from_bytes(&bytes[..cut]).is_err());
        }
    }

    #[test]
    fn varint_roundtrip() {
        let mut buf = Vec::new();
        for v in [0u64, 1, 127, 128, 300, u32::MAX as u64] {
            put_varint(&mut buf, v);
        }
        let mut r = Reader { data: &buf, pos: 0 };
        for v in [0u64, 1, 127, 128, 300, u32::MAX as u64] {
            assert_eq!(r.varint().unwrap(), v);
        }
    }
}
