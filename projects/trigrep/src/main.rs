use std::path::PathBuf;
use std::process::ExitCode;

use trigrep::regex::Regex;
use trigrep::search;

const USAGE: &str = "usage:
  trigrep index [DIR]                         build or incrementally update DIR/.trigrep.idx
  trigrep search [-i] [-F] [-d DIR] PATTERN   search using the index (-i ignore case, -F literal)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("trigrep: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    match args.first().map(String::as_str) {
        Some("index") => {
            let root = PathBuf::from(args.get(1).map_or(".", String::as_str));
            let err = |e: std::io::Error| e.to_string();
            let idx = match search::load(&root) {
                Ok(mut idx) => {
                    let st = search::update(&root, &mut idx).map_err(err)?;
                    println!("updated: +{} -{} ={}", st.added, st.removed, st.unchanged);
                    idx
                }
                Err(_) => search::build(&root).map_err(err)?,
            };
            search::save(&root, &idx).map_err(err)?;
            println!("indexed {} files, {} trigrams", idx.files.len(), idx.postings.len());
            Ok(ExitCode::SUCCESS)
        }
        Some("search") => run_search(&args[1..]),
        _ => Err(USAGE.to_string()),
    }
}

fn run_search(args: &[String]) -> Result<ExitCode, String> {
    let (mut ignore_case, mut literal) = (false, false);
    let mut dir = PathBuf::from(".");
    let mut pattern = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-i" => ignore_case = true,
            "-F" => literal = true,
            "-d" => dir = PathBuf::from(it.next().ok_or("-d needs a directory")?),
            _ if pattern.is_none() => pattern = Some(a.clone()),
            _ => return Err(USAGE.to_string()),
        }
    }
    let pattern = pattern.ok_or(USAGE)?;
    let re = if literal {
        Regex::literal(&pattern, ignore_case)
    } else {
        Regex::new(&pattern, ignore_case).map_err(|e| format!("bad pattern: {e}"))?
    };
    let idx = search::load(&dir).map_err(|e| format!("cannot load index (run `trigrep index` first): {e}"))?;
    let (hits, stats) = search::search(&dir, &idx, &re).map_err(|e| e.to_string())?;
    for h in &hits {
        println!("{}:{}:{}", h.path, h.line_no, h.line);
    }
    eprintln!("{} matches; scanned {} of {} indexed files", hits.len(), stats.candidates, stats.total_files);
    Ok(if hits.is_empty() { ExitCode::from(1) } else { ExitCode::SUCCESS })
}
