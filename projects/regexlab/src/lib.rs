//! regexlab: a from-scratch regular expression engine.
//!
//! ```
//! let re = regexlab::Regex::new(r"(\w+)@(\w+)\.com").unwrap();
//! let caps = re.captures("mail bob@example.com now").unwrap();
//! assert_eq!(caps.get(1), Some("bob"));
//! assert_eq!(caps.get(2), Some("example"));
//! ```

pub mod ast;
pub mod dfa;
pub mod nfa;
pub mod parser;
pub mod pike;
pub mod trace;

pub use parser::{parse, ParseError};

use nfa::Program;

/// A compiled regular expression.
#[derive(Debug, Clone)]
pub struct Regex {
    prog: Program,
}

/// Capture spans for one match, as byte offsets into the haystack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Captures<'h> {
    haystack: &'h str,
    spans: Vec<Option<(usize, usize)>>,
}

impl<'h> Captures<'h> {
    pub fn get(&self, i: usize) -> Option<&'h str> {
        self.span(i).map(|(a, b)| &self.haystack[a..b])
    }

    pub fn span(&self, i: usize) -> Option<(usize, usize)> {
        self.spans.get(i).copied().flatten()
    }

    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Regex, ParseError> {
        let ast = parse(pattern)?;
        Ok(Regex { prog: nfa::compile(&ast) })
    }

    pub fn program(&self) -> &Program {
        &self.prog
    }

    pub fn is_match(&self, haystack: &str) -> bool {
        self.find(haystack).is_some()
    }

    /// Yes/no matching through a lazily built DFA. Faster than `is_match` on
    /// long inputs because no per-thread capture state is carried.
    pub fn is_match_dfa(&self, haystack: &str) -> bool {
        dfa::Dfa::new(&self.prog).is_match(haystack)
    }

    /// Leftmost-first match as a byte range.
    pub fn find(&self, haystack: &str) -> Option<(usize, usize)> {
        self.captures(haystack).and_then(|c| c.span(0))
    }

    pub fn captures<'h>(&self, haystack: &'h str) -> Option<Captures<'h>> {
        let chars: Vec<char> = haystack.chars().collect();
        let slots = pike::search(&self.prog, &chars, 0)?;
        // char index -> byte offset (one extra entry for the end of input)
        let mut offsets: Vec<usize> = haystack.char_indices().map(|(i, _)| i).collect();
        offsets.push(haystack.len());
        let spans = slots
            .chunks(2)
            .map(|p| match (p[0], p[1]) {
                (Some(a), Some(b)) => Some((offsets[a], offsets[b])),
                _ => None,
            })
            .collect();
        Some(Captures { haystack, spans })
    }

    /// All non-overlapping matches, left to right.
    pub fn find_iter(&self, haystack: &str) -> Vec<(usize, usize)> {
        let chars: Vec<char> = haystack.chars().collect();
        let mut offsets: Vec<usize> = haystack.char_indices().map(|(i, _)| i).collect();
        offsets.push(haystack.len());
        let mut out = Vec::new();
        let mut at = 0;
        while at <= chars.len() {
            let Some(s) = pike::search(&self.prog, &chars, at) else { break };
            let (a, b) = (s[0].unwrap(), s[1].unwrap());
            out.push((offsets[a], offsets[b]));
            at = if b == a { b + 1 } else { b };
        }
        out
    }
}
