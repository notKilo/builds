use std::io;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = tinyvm::cli::run_cli(&args, &mut io::stdout(), &mut io::stderr());
    std::process::exit(code);
}
