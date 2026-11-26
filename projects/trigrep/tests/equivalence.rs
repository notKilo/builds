//! The index must only prune work, never change results: for every query the indexed search
//! returns exactly what a full scan with the same matcher returns.

use std::fs;
use std::path::{Path, PathBuf};

use trigrep::regex::Regex;
use trigrep::search::{build, search, Hit};

fn corpus(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("trigrep-eq-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("a/b")).unwrap();
    // Deterministic pseudo-random words so files overlap partially.
    let words = ["alpha", "beta", "gamma", "delta", "parser", "lexer", "token", "index", "query", "Trigram"];
    let mut seed = 12345u32;
    for f in 0..40 {
        let mut body = String::new();
        for l in 0..30 {
            let mut line = Vec::new();
            for _ in 0..4 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                line.push(words[(seed >> 16) as usize % words.len()]);
            }
            body.push_str(&format!("{} {l}\n", line.join(" ")));
        }
        let dir = if f % 2 == 0 { "a" } else { "a/b" };
        fs::write(root.join(dir).join(format!("f{f}.txt")), body).unwrap();
    }
    root
}

fn naive(root: &Path, re: &Regex) -> Vec<Hit> {
    let mut hits = Vec::new();
    for entry in trigrep::walk::walk(root).unwrap() {
        let content = fs::read(root.join(&entry.rel_path)).unwrap();
        for (i, line) in content.split(|&b| b == b'\n').enumerate() {
            if re.is_match(line) {
                hits.push(Hit { path: entry.rel_path.clone(), line_no: i + 1, line: String::from_utf8_lossy(line).to_string() });
            }
        }
    }
    hits
}

fn sorted(mut v: Vec<Hit>) -> Vec<Hit> {
    v.sort_by(|a, b| (&a.path, a.line_no).cmp(&(&b.path, b.line_no)));
    v
}

#[test]
fn indexed_search_matches_full_scan() {
    let root = corpus("equiv");
    let idx = build(&root).unwrap();
    let queries: Vec<(Regex, &str)> = vec![
        (Regex::literal("parser lexer", false), "literal"),
        (Regex::literal("TRIGRAM", true), "literal, ignore case"),
        (Regex::new("alpha.*delta", false).unwrap(), "regex with gap"),
        (Regex::new("(token|query) index", false).unwrap(), "alternation"),
        (Regex::new("gam+a", false).unwrap(), "repetition"),
        (Regex::literal("nothing like this", false), "no match"),
        (Regex::new("a", false).unwrap(), "too short for trigrams"),
    ];
    for (re, name) in queries {
        let (hits, _) = search(&root, &idx, &re).unwrap();
        assert_eq!(sorted(hits), sorted(naive(&root, &re)), "mismatch for {name}");
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn index_prunes_candidates_for_selective_queries() {
    let root = corpus("prune");
    fs::write(root.join("a/needle.txt"), "the zxqvj marker\n").unwrap();
    let idx = build(&root).unwrap();
    let (hits, stats) = search(&root, &idx, &Regex::literal("zxqvj", false)).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(stats.candidates, 1);
    assert!(stats.total_files > 40);
    let _ = fs::remove_dir_all(&root);
}
