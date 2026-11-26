//! Compare indexed search with a full scan: `cargo run --release --example bench -- DIR PATTERN...`

use std::path::Path;
use std::time::Instant;

use trigrep::regex::Regex;
use trigrep::search::{build, search};
use trigrep::walk::walk;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (dir, patterns) = match args.split_first() {
        Some((d, p)) if !p.is_empty() => (Path::new(d), p),
        _ => {
            eprintln!("usage: bench DIR PATTERN...");
            std::process::exit(2);
        }
    };
    let t = Instant::now();
    let idx = build(dir).expect("index");
    println!("index {} files in {:?}", idx.files.len(), t.elapsed());
    println!("{:<24} {:>8} {:>10} {:>12} {:>12}", "pattern", "hits", "candidates", "indexed", "full scan");
    for p in patterns {
        let re = Regex::new(p, false).expect("pattern");
        let t = Instant::now();
        let (hits, stats) = search(dir, &idx, &re).expect("search");
        let indexed = t.elapsed();
        let t = Instant::now();
        let mut naive = 0;
        for e in walk(dir).expect("walk") {
            if let Ok(c) = std::fs::read(dir.join(&e.rel_path)) {
                naive += c.split(|&b| b == b'\n').filter(|l| re.is_match(l)).count();
            }
        }
        let full = t.elapsed();
        assert_eq!(hits.len(), naive, "indexed and full scan disagree for {p}");
        println!("{:<24} {:>8} {:>7}/{:<3} {:>12.2?} {:>12.2?}", p, hits.len(), stats.candidates, stats.total_files, indexed, full);
    }
}
