//! Directory walking: collects indexable text files under a root.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &["target", "node_modules", "__pycache__", "dist", "build"];
const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024;

/// A file selected for indexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// Path relative to the walk root, using `/` separators.
    pub rel_path: String,
    pub size: u64,
    pub mtime_secs: u64,
}

/// Heuristic binary check: a NUL byte in the first 8 KiB.
pub fn looks_binary(data: &[u8]) -> bool {
    data.iter().take(8192).any(|&b| b == 0)
}

/// Walk `root` and list candidate files in sorted order.
/// Hidden entries, well-known build directories and oversized files are skipped.
/// Binary detection happens later, when content is read.
pub fn walk(root: &Path) -> io::Result<Vec<FileEntry>> {
    let mut out = Vec::new();
    walk_dir(root, root, &mut out)?;
    out.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(out)
}

fn walk_dir(root: &Path, dir: &Path, out: &mut Vec<FileEntry>) -> io::Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') {
            continue;
        }
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_dir() {
            if SKIP_DIRS.contains(&name) {
                continue;
            }
            walk_dir(root, &path, out)?;
        } else if meta.is_file() && meta.len() <= MAX_FILE_SIZE {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel_path = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let mtime_secs = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            out.push(FileEntry { rel_path, size: meta.len(), mtime_secs });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("trigrep-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn skips_hidden_and_build_dirs() {
        let root = tempdir("walk");
        fs::create_dir_all(root.join("src/deep")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("src/a.rs"), "fn a() {}").unwrap();
        fs::write(root.join("src/deep/b.rs"), "fn b() {}").unwrap();
        fs::write(root.join(".git/config"), "x").unwrap();
        fs::write(root.join("target/out"), "x").unwrap();
        fs::write(root.join(".hidden"), "x").unwrap();
        let files = walk(&root).unwrap();
        let names: Vec<_> = files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(names, ["src/a.rs", "src/deep/b.rs"]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn detects_binary() {
        assert!(looks_binary(b"ab\0cd"));
        assert!(!looks_binary(b"plain text\n"));
    }
}
