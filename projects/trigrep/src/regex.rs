//! A small backtracking regex: literals, `.`, classes, `\d \w \s`, `* + ?`, `|`, groups.
//! Also extracts literal runs that every match must contain, for index prefiltering.

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Lit(u8),
    Any,
    Class { negated: bool, ranges: Vec<(u8, u8)> },
    Cat(Vec<Node>),
    Alt(Vec<Node>),
    Star(Box<Node>),
    Plus(Box<Node>),
    Opt(Box<Node>),
}

#[derive(Debug, Clone)]
pub struct Regex {
    root: Node,
    ignore_case: bool,
}

struct Parser<'a> {
    s: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.pos).copied()
    }

    fn alt(&mut self) -> Result<Node, String> {
        let mut branches = vec![self.cat()?];
        while self.peek() == Some(b'|') {
            self.pos += 1;
            branches.push(self.cat()?);
        }
        Ok(if branches.len() == 1 { branches.pop().unwrap() } else { Node::Alt(branches) })
    }

    fn cat(&mut self) -> Result<Node, String> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == b'|' || c == b')' {
                break;
            }
            let atom = self.atom()?;
            items.push(self.postfix(atom));
        }
        Ok(Node::Cat(items))
    }

    fn postfix(&mut self, atom: Node) -> Node {
        let mut node = atom;
        while let Some(c) = self.peek() {
            node = match c {
                b'*' => Node::Star(Box::new(node)),
                b'+' => Node::Plus(Box::new(node)),
                b'?' => Node::Opt(Box::new(node)),
                _ => break,
            };
            self.pos += 1;
        }
        node
    }

    fn atom(&mut self) -> Result<Node, String> {
        let c = self.peek().ok_or("unexpected end of pattern")?;
        self.pos += 1;
        match c {
            b'.' => Ok(Node::Any),
            b'(' => {
                let inner = self.alt()?;
                if self.peek() != Some(b')') {
                    return Err("unclosed group".into());
                }
                self.pos += 1;
                Ok(inner)
            }
            b'[' => self.class(),
            b'\\' => self.escape(),
            b'*' | b'+' | b'?' => Err(format!("nothing to repeat before '{}'", c as char)),
            _ => Ok(Node::Lit(c)),
        }
    }

    fn escape(&mut self) -> Result<Node, String> {
        let c = self.peek().ok_or("trailing backslash")?;
        self.pos += 1;
        Ok(match c {
            b'd' => Node::Class { negated: false, ranges: vec![(b'0', b'9')] },
            b'w' => Node::Class {
                negated: false,
                ranges: vec![(b'a', b'z'), (b'A', b'Z'), (b'0', b'9'), (b'_', b'_')],
            },
            b's' => Node::Class { negated: false, ranges: vec![(b' ', b' '), (b'\t', b'\r')] },
            b'n' => Node::Lit(b'\n'),
            b't' => Node::Lit(b'\t'),
            other => Node::Lit(other),
        })
    }

    fn class(&mut self) -> Result<Node, String> {
        let negated = self.peek() == Some(b'^');
        if negated {
            self.pos += 1;
        }
        let mut ranges = Vec::new();
        let mut first = true;
        loop {
            let c = self.peek().ok_or("unclosed character class")?;
            self.pos += 1;
            if c == b']' && !first {
                break;
            }
            first = false;
            let lo = if c == b'\\' {
                let e = self.peek().ok_or("trailing backslash")?;
                self.pos += 1;
                e
            } else {
                c
            };
            if self.peek() == Some(b'-') && self.s.get(self.pos + 1).is_some_and(|&n| n != b']') {
                let hi = self.s[self.pos + 1];
                self.pos += 2;
                if hi < lo {
                    return Err("invalid range in class".into());
                }
                ranges.push((lo, hi));
            } else {
                ranges.push((lo, lo));
            }
        }
        Ok(Node::Class { negated, ranges })
    }
}

impl Regex {
    pub fn new(pattern: &str, ignore_case: bool) -> Result<Regex, String> {
        let mut p = Parser { s: pattern.as_bytes(), pos: 0 };
        let root = p.alt()?;
        if p.pos != p.s.len() {
            return Err("unmatched ')'".into());
        }
        Ok(Regex { root, ignore_case })
    }

    /// Pattern matching `text` literally.
    pub fn literal(text: &str, ignore_case: bool) -> Regex {
        let items = text.bytes().map(Node::Lit).collect();
        Regex { root: Node::Cat(items), ignore_case }
    }

    /// Does the pattern match anywhere in `line`?
    pub fn is_match(&self, line: &[u8]) -> bool {
        (0..=line.len()).any(|start| self.m(&self.root, line, start, &mut |_| true))
    }

    fn eq(&self, a: u8, b: u8) -> bool {
        if self.ignore_case { a.eq_ignore_ascii_case(&b) } else { a == b }
    }

    fn class_has(&self, ranges: &[(u8, u8)], b: u8) -> bool {
        let hit = |c: u8| ranges.iter().any(|&(lo, hi)| lo <= c && c <= hi);
        hit(b) || (self.ignore_case && (hit(b.to_ascii_lowercase()) || hit(b.to_ascii_uppercase())))
    }

    /// Match `node` at `pos`, calling `k` with each possible end position until it accepts.
    fn m(&self, node: &Node, s: &[u8], pos: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
        match node {
            Node::Lit(c) => pos < s.len() && self.eq(s[pos], *c) && k(pos + 1),
            Node::Any => pos < s.len() && k(pos + 1),
            Node::Class { negated, ranges } => {
                pos < s.len() && self.class_has(ranges, s[pos]) != *negated && k(pos + 1)
            }
            Node::Cat(items) => self.cat(items, s, pos, k),
            Node::Alt(branches) => branches.iter().any(|b| self.m(b, s, pos, k)),
            Node::Opt(n) => self.m(n, s, pos, k) || k(pos),
            Node::Plus(n) => self.m(n, s, pos, &mut |p| p > pos && self.star(n, s, p, k)),
            Node::Star(n) => self.star(n, s, pos, k),
        }
    }

    fn cat(&self, items: &[Node], s: &[u8], pos: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
        match items.split_first() {
            None => k(pos),
            Some((first, rest)) => self.m(first, s, pos, &mut |p| self.cat(rest, s, p, k)),
        }
    }

    fn star(&self, n: &Node, s: &[u8], pos: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
        // Greedy; the progress check (p > pos) prevents looping on empty matches.
        self.m(n, s, pos, &mut |p| p > pos && self.star(n, s, p, k)) || k(pos)
    }

    /// Literal strings that every match must contain (lowercased when ignoring case is
    /// irrelevant: the index lowercases anyway). Runs shorter than 3 bytes are dropped.
    pub fn required_literals(&self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        collect(&self.root, &mut out);
        out.retain(|r| r.len() >= 3);
        out
    }
}

fn collect(node: &Node, out: &mut Vec<Vec<u8>>) {
    match node {
        Node::Lit(c) => out.push(vec![*c]),
        Node::Cat(items) => {
            let mut run: Vec<u8> = Vec::new();
            for item in items {
                if let Node::Lit(c) = item {
                    run.push(*c);
                    continue;
                }
                if !run.is_empty() {
                    out.push(std::mem::take(&mut run));
                }
                // Optional or alternative parts impose nothing; a `+` body is required.
                if matches!(item, Node::Plus(_) | Node::Cat(_)) {
                    let inner = if let Node::Plus(n) = item { n.as_ref() } else { item };
                    collect(inner, out);
                }
            }
            if !run.is_empty() {
                out.push(run);
            }
        }
        Node::Plus(n) => collect(n, out),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(p: &str, s: &str) -> bool {
        Regex::new(p, false).unwrap().is_match(s.as_bytes())
    }

    #[test]
    fn basics() {
        assert!(hit("abc", "xxabcxx"));
        assert!(!hit("abc", "ab"));
        assert!(hit("a.c", "abc"));
        assert!(hit("ab*c", "ac"));
        assert!(hit("ab+c", "abbbc"));
        assert!(!hit("ab+c", "ac"));
        assert!(hit("colou?r", "color"));
        assert!(hit("cat|dog", "hotdog"));
        assert!(hit("(ab)+c", "ababc"));
    }

    #[test]
    fn classes_and_escapes() {
        assert!(hit("[a-c]+z", "bbz"));
        assert!(!hit("[^a-c]z", "bz"));
        assert!(hit(r"\d+\.\d+", "v 3.14"));
        assert!(hit(r"fn\s\w+\(", "fn main("));
        assert!(hit("[]]", "]"));
    }

    #[test]
    fn ignore_case() {
        assert!(Regex::new("hello", true).unwrap().is_match(b"HeLLo"));
        assert!(Regex::new("[a-z]+", true).unwrap().is_match(b"ABC"));
        assert!(!Regex::new("hello", false).unwrap().is_match(b"HELLO"));
    }

    #[test]
    fn nested_star_terminates() {
        assert!(!hit("(a*)*b", "aaaa"));
        assert!(hit("(a*)*b", "aaab"));
    }

    #[test]
    fn errors() {
        for p in ["(ab", "ab)", "*a", "[a-", "a\\", "[z-a]"] {
            assert!(Regex::new(p, false).is_err(), "{p}");
        }
    }

    #[test]
    fn literal_constructor_escapes_metachars() {
        assert!(Regex::literal("a.b*", false).is_match(b"xa.b*y"));
        assert!(!Regex::literal("a.b", false).is_match(b"axb"));
    }

    #[test]
    fn required_literals() {
        let r = |p: &str| Regex::new(p, false).unwrap().required_literals();
        assert_eq!(r("hello.*world"), vec![b"hello".to_vec(), b"world".to_vec()]);
        assert_eq!(r("foo(bar)+"), vec![b"foo".to_vec(), b"bar".to_vec()]);
        assert!(r("cat|dog").is_empty());
        assert!(r("ab?cd").is_empty());
        assert!(r(".*").is_empty());
    }
}
