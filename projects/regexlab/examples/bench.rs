//! Rough timing of the Pike VM vs the lazy DFA: `cargo run --release --example bench`
use regexlab::Regex;
use std::time::Instant;

fn main() {
    let cases = [("(a|b)*abb", "ab".repeat(100_000)), ("(a+)+b", "a".repeat(100_000)), (r"\d+\.\d+$", "x1.5 ".repeat(50_000))];
    for (pat, text) in cases {
        let re = Regex::new(pat).unwrap();
        let t = Instant::now();
        let a = re.is_match(&text);
        let pike = t.elapsed();
        let t = Instant::now();
        let b = re.is_match_dfa(&text);
        let dfa = t.elapsed();
        assert_eq!(a, b);
        println!("{pat:<14} {} bytes  pike {pike:>10.2?}  dfa {dfa:>10.2?}  match={a}", text.len());
    }
}
