use std::path::Path;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    match trigrep::walk::walk(Path::new(&root)) {
        Ok(files) => println!("{} files", files.len()),
        Err(e) => eprintln!("error: {e}"),
    }
}
