//! The DFA and the Pike VM are independent implementations; they must agree.
use regexlab::Regex;

const PATTERNS: &[&str] = &[
    "a", "ab|cd", "a*b", "(a|b)*abb", "^a+$", "a$", "^$", "x?y?z?", "[a-c]+d", "(ab)+",
    "a{2,3}b", "(a*)*b", "\\d+\\.\\d+", "^(a|ab)(c|bcd)$", "b+$", ".*a.*b", "[^a]b",
];

/// Deterministic xorshift so failures are reproducible without a rand crate.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn dfa_agrees_with_pike_vm() {
    let alphabet: Vec<char> = "abcdxyz.19\n".chars().collect();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for pat in PATTERNS {
        let re = Regex::new(pat).unwrap();
        for _ in 0..400 {
            let len = (rng.next() % 12) as usize;
            let s: String = (0..len).map(|_| alphabet[(rng.next() % alphabet.len() as u64) as usize]).collect();
            assert_eq!(re.is_match(&s), re.is_match_dfa(&s), "pattern {pat:?} input {s:?}");
        }
    }
}

#[test]
fn dfa_state_count_stays_small() {
    let re = Regex::new("(a|b)*abb").unwrap();
    let mut dfa = regexlab::dfa::Dfa::new(re.program());
    assert!(!dfa.is_match(&"ab".repeat(1000)));
    assert!(dfa.is_match(&format!("{}b", "ab".repeat(1000))));
    assert!(dfa.state_count() < 20, "got {}", dfa.state_count());
}
