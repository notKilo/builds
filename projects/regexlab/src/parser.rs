//! Recursive-descent parser: pattern string -> `Ast`.

use crate::ast::{Ast, Class, Node};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub pos: usize,
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at {}: {}", self.pos, self.msg)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(pattern: &str) -> Result<Ast, ParseError> {
    let mut p = Parser { chars: pattern.chars().collect(), pos: 0, groups: 0 };
    let root = p.alternation()?;
    if p.pos < p.chars.len() {
        return Err(p.err("unmatched ')'"));
    }
    Ok(Ast { root, group_count: p.groups })
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    groups: usize,
}

const MAX_REPEAT: u32 = 1000;

impl Parser {
    fn err(&self, msg: &str) -> ParseError {
        ParseError { pos: self.pos, msg: msg.to_string() }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn alternation(&mut self) -> Result<Node, ParseError> {
        let mut branches = vec![self.concat()?];
        while self.eat('|') {
            branches.push(self.concat()?);
        }
        Ok(if branches.len() == 1 { branches.pop().unwrap() } else { Node::Alt(branches) })
    }

    fn concat(&mut self) -> Result<Node, ParseError> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            items.push(self.repeat()?);
        }
        Ok(match items.len() {
            0 => Node::Empty,
            1 => items.pop().unwrap(),
            _ => Node::Concat(items),
        })
    }

    fn repeat(&mut self) -> Result<Node, ParseError> {
        let mut node = self.atom()?;
        loop {
            let (min, max) = match self.peek() {
                Some('*') => (0, None),
                Some('+') => (1, None),
                Some('?') => (0, Some(1)),
                Some('{') => match self.braces()? {
                    Some(b) => {
                        self.pos -= 1; // braces() consumed through '}', undo for the shared advance below
                        b
                    }
                    None => break,
                },
                _ => break,
            };
            self.pos += 1;
            if matches!(node, Node::Start | Node::End | Node::Empty) {
                return Err(self.err("nothing to repeat"));
            }
            let greedy = !self.eat('?');
            node = Node::Repeat { node: Box::new(node), min, max, greedy };
        }
        Ok(node)
    }

    /// Parses `{n}`, `{n,}` or `{n,m}` at the cursor. A `{` that does not form
    /// a valid counted repetition is treated as a literal (returns `None`).
    fn braces(&mut self) -> Result<Option<(u32, Option<u32>)>, ParseError> {
        let start = self.pos;
        self.pos += 1;
        let min = self.number();
        let parsed = match (min, self.peek()) {
            (Some(n), Some('}')) => Some((n, Some(n))),
            (Some(n), Some(',')) => {
                self.pos += 1;
                let max = self.number();
                if self.peek() == Some('}') { Some((n, max)) } else { None }
            }
            _ => None,
        };
        match parsed {
            Some((min, max)) => {
                if min > MAX_REPEAT || max.is_some_and(|m| m > MAX_REPEAT) {
                    return Err(self.err("repetition count too large"));
                }
                if max.is_some_and(|m| m < min) {
                    return Err(self.err("invalid repetition range"));
                }
                self.pos += 1; // '}'
                Ok(Some((min, max)))
            }
            None => {
                self.pos = start;
                Ok(None)
            }
        }
    }

    fn number(&mut self) -> Option<u32> {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        if start == self.pos {
            return None;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        Some(s.parse().unwrap_or(u32::MAX))
    }

    fn atom(&mut self) -> Result<Node, ParseError> {
        let c = self.peek().ok_or_else(|| self.err("unexpected end"))?;
        match c {
            '(' => {
                self.pos += 1;
                let index = if self.peek() == Some('?') {
                    if self.chars.get(self.pos + 1) != Some(&':') {
                        return Err(self.err("unsupported group syntax"));
                    }
                    self.pos += 2;
                    None
                } else {
                    self.groups += 1;
                    Some(self.groups)
                };
                let node = self.alternation()?;
                if !self.eat(')') {
                    return Err(self.err("unclosed group"));
                }
                Ok(Node::Group { node: Box::new(node), index })
            }
            '[' => self.class(),
            '.' => {
                self.pos += 1;
                Ok(Node::Any)
            }
            '^' => {
                self.pos += 1;
                Ok(Node::Start)
            }
            '$' => {
                self.pos += 1;
                Ok(Node::End)
            }
            '\\' => self.escape(),
            '*' | '+' | '?' => Err(self.err("nothing to repeat")),
            _ => {
                self.pos += 1;
                Ok(Node::Char(c))
            }
        }
    }

    fn escape(&mut self) -> Result<Node, ParseError> {
        self.pos += 1;
        let c = self.peek().ok_or_else(|| self.err("trailing backslash"))?;
        self.pos += 1;
        if let Some(class) = shorthand(c) {
            return Ok(Node::Class(class));
        }
        Ok(Node::Char(escaped_char(c)))
    }

    fn class(&mut self) -> Result<Node, ParseError> {
        self.pos += 1; // '['
        let negated = self.eat('^');
        let mut ranges: Vec<(char, char)> = Vec::new();
        let mut first = true;
        loop {
            let c = self.peek().ok_or_else(|| self.err("unclosed class"))?;
            self.pos += 1;
            if c == ']' && !first {
                break;
            }
            first = false;
            let lo = if c == '\\' {
                let e = self.peek().ok_or_else(|| self.err("trailing backslash"))?;
                self.pos += 1;
                if let Some(cls) = shorthand(e) {
                    if cls.negated {
                        return Err(self.err("negated shorthand inside class is unsupported"));
                    }
                    ranges.extend(cls.ranges);
                    continue;
                }
                escaped_char(e)
            } else {
                c
            };
            // `a-z` range, but a trailing '-' is a literal.
            if self.peek() == Some('-') && self.chars.get(self.pos + 1).is_some_and(|&n| n != ']') {
                self.pos += 1;
                let mut hi = self.peek().unwrap();
                self.pos += 1;
                if hi == '\\' {
                    let e = self.peek().ok_or_else(|| self.err("trailing backslash"))?;
                    self.pos += 1;
                    hi = escaped_char(e);
                }
                if hi < lo {
                    return Err(self.err("invalid class range"));
                }
                ranges.push((lo, hi));
            } else {
                ranges.push((lo, lo));
            }
        }
        Ok(Node::Class(Class { negated, ranges }))
    }
}

fn escaped_char(c: char) -> char {
    match c {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        other => other,
    }
}

fn shorthand(c: char) -> Option<Class> {
    let digits = vec![('0', '9')];
    let word = vec![('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')];
    let space = vec![(' ', ' '), ('\t', '\r')];
    let (ranges, negated) = match c {
        'd' => (digits, false),
        'D' => (digits, true),
        'w' => (word, false),
        'W' => (word, true),
        's' => (space, false),
        'S' => (space, true),
        _ => return None,
    };
    Some(Class { negated, ranges })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(p: &str) -> Node {
        parse(p).unwrap().root
    }

    #[test]
    fn literals_and_concat() {
        assert_eq!(root("ab"), Node::Concat(vec![Node::Char('a'), Node::Char('b')]));
        assert_eq!(root(""), Node::Empty);
    }

    #[test]
    fn alternation_has_lowest_precedence() {
        assert_eq!(
            root("ab|c"),
            Node::Alt(vec![Node::Concat(vec![Node::Char('a'), Node::Char('b')]), Node::Char('c')])
        );
    }

    #[test]
    fn quantifiers() {
        assert_eq!(
            root("a+?"),
            Node::Repeat { node: Box::new(Node::Char('a')), min: 1, max: None, greedy: false }
        );
        assert_eq!(
            root("a{2,5}"),
            Node::Repeat { node: Box::new(Node::Char('a')), min: 2, max: Some(5), greedy: true }
        );
        assert_eq!(
            root("a{3,}"),
            Node::Repeat { node: Box::new(Node::Char('a')), min: 3, max: None, greedy: true }
        );
    }

    #[test]
    fn brace_without_count_is_literal() {
        assert_eq!(root("a{x"), Node::Concat(vec![Node::Char('a'), Node::Char('{'), Node::Char('x')]));
    }

    #[test]
    fn groups_are_numbered_left_to_right() {
        let ast = parse("(a)(?:b)(c(d))").unwrap();
        assert_eq!(ast.group_count, 3);
    }

    #[test]
    fn classes() {
        match root("[^a-c_\\d-]") {
            Node::Class(c) => {
                assert!(c.negated);
                assert_eq!(c.ranges, vec![('a', 'c'), ('_', '_'), ('0', '9'), ('-', '-')]);
            }
            other => panic!("unexpected {other:?}"),
        }
        assert!(matches!(root("[]a]"), Node::Class(c) if c.matches(']')));
    }

    #[test]
    fn errors() {
        for bad in ["(", "a)", "*a", "[a", "a{3,2}", "\\", "[z-a]", "(?x)", "^*"] {
            assert!(parse(bad).is_err(), "{bad} should fail");
        }
    }
}
