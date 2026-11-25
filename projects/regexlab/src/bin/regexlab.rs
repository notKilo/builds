use regexlab::{trace, Regex};
use std::process::ExitCode;

const USAGE: &str = "usage:
  regexlab match <pattern> <text>   print the match and capture groups (exit 1 if none)
  regexlab all   <pattern> <text>   print every non-overlapping match
  regexlab trace <pattern> <text>   step through the NFA, one row per character
  regexlab prog  <pattern>          print the compiled program
  regexlab dot   <pattern>          print the NFA as Graphviz DOT";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let (cmd, pattern, text) = match args {
        [c, p] if c == "prog" || c == "dot" => (c.as_str(), p, ""),
        [c, p, t] if matches!(c.as_str(), "match" | "all" | "trace") => (c.as_str(), p, t.as_str()),
        _ => return Err(USAGE.to_string()),
    };
    let re = Regex::new(pattern).map_err(|e| e.to_string())?;
    match cmd {
        "match" => match re.captures(text) {
            None => return Ok(ExitCode::from(1)),
            Some(caps) => {
                for i in 0..caps.len() {
                    match (caps.span(i), caps.get(i)) {
                        (Some((a, b)), Some(s)) => println!("{i}: [{a}..{b}] {s:?}"),
                        _ => println!("{i}: (unset)"),
                    }
                }
            }
        },
        "all" => {
            let found = re.find_iter(text);
            for (a, b) in &found {
                println!("[{a}..{b}] {:?}", &text[*a..*b]);
            }
            if found.is_empty() {
                return Ok(ExitCode::from(1));
            }
        }
        "trace" => print!("{}", trace::trace(re.program(), text)),
        "prog" => print!("{}", re.program()),
        _ => print!("{}", trace::dot(re.program())),
    }
    Ok(ExitCode::SUCCESS)
}
