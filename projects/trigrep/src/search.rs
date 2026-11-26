//! Index building, persistence and querying.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::index::{trigrams, Index};
use crate::regex::Regex;
use crate::walk::{looks_binary, walk, FileEntry};

pub const INDEX_FILE: &str = ".trigrep.idx";

#[derive(Debug, PartialEq, Eq)]
pub struct Hit {
    pub path: String,
    pub line_no: usize,
    pub line: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SearchStats {
    pub total_files: usize,
    pub candidates: usize,
}

pub fn index_path(root: &Path) -> PathBuf {
    root.join(INDEX_FILE)
}

/// Build a fresh index of everything under `root`.
pub fn build(root: &Path) -> io::Result<Index> {
    let mut idx = Index::default();
    for entry in walk(root)? {
        add_file(&mut idx, root, entry);
    }
    Ok(idx)
}

fn add_file(idx: &mut Index, root: &Path, entry: FileEntry) {
    // Unreadable and binary files are simply left out of the index.
    if let Ok(content) = fs::read(root.join(&entry.rel_path)) {
        if !looks_binary(&content) {
            idx.add(entry, &content);
        }
    }
}

pub fn save(root: &Path, idx: &Index) -> io::Result<()> {
    let tmp = root.join(format!("{INDEX_FILE}.tmp"));
    fs::write(&tmp, idx.to_bytes())?;
    fs::rename(tmp, index_path(root))
}

pub fn load(root: &Path) -> io::Result<Index> {
    Index::from_bytes(&fs::read(index_path(root))?)
}

/// Trigrams every match must contain, derived from the regex's required literals.
pub fn required_trigrams(re: &Regex) -> BTreeSet<[u8; 3]> {
    re.required_literals().iter().flat_map(|lit| trigrams(lit)).collect()
}

/// Run `re` over the candidate files selected by the index.
pub fn search(root: &Path, idx: &Index, re: &Regex) -> io::Result<(Vec<Hit>, SearchStats)> {
    let ids = idx.candidates(&required_trigrams(re));
    let stats = SearchStats { total_files: idx.files.len(), candidates: ids.len() };
    let mut hits = Vec::new();
    for id in ids {
        let rel = &idx.files[id as usize].rel_path;
        let Ok(content) = fs::read(root.join(rel)) else { continue };
        for (i, line) in content.split(|&b| b == b'\n').enumerate() {
            if re.is_match(line) {
                hits.push(Hit {
                    path: rel.clone(),
                    line_no: i + 1,
                    line: String::from_utf8_lossy(line).trim_end_matches('\r').to_string(),
                });
            }
        }
    }
    Ok((hits, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("trigrep-s-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(p.join("src")).unwrap();
        fs::write(p.join("src/a.rs"), "fn alpha() {}\nfn beta() {}\n").unwrap();
        fs::write(p.join("src/b.rs"), "// Alpha mentioned\nlet x = 42;\n").unwrap();
        fs::write(p.join("blob.bin"), b"alpha\0\0binary").unwrap();
        p
    }

    #[test]
    fn finds_literal_and_prunes_files() {
        let root = fixture("lit");
        let idx = build(&root).unwrap();
        assert_eq!(idx.files.len(), 2, "binary file is not indexed");
        let (hits, stats) = search(&root, &idx, &Regex::literal("beta", false)).unwrap();
        assert_eq!(hits, vec![Hit { path: "src/a.rs".into(), line_no: 2, line: "fn beta() {}".into() }]);
        assert_eq!(stats, SearchStats { total_files: 2, candidates: 1 });
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn case_insensitive_and_regex() {
        let root = fixture("re");
        let idx = build(&root).unwrap();
        let (hits, _) = search(&root, &idx, &Regex::literal("alpha", true)).unwrap();
        assert_eq!(hits.len(), 2);
        let re = Regex::new(r"let \w+ = \d+", false).unwrap();
        let (hits, _) = search(&root, &idx, &re).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line_no, 2);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn short_patterns_scan_everything() {
        let root = fixture("short");
        let idx = build(&root).unwrap();
        let (hits, stats) = search(&root, &idx, &Regex::literal("42", false)).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(stats.candidates, 2);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn save_and_load() {
        let root = fixture("persist");
        let idx = build(&root).unwrap();
        save(&root, &idx).unwrap();
        assert_eq!(load(&root).unwrap(), idx);
        fs::remove_dir_all(&root).unwrap();
    }
}
