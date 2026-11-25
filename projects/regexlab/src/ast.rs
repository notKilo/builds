//! Abstract syntax tree produced by the parser.

/// A set of inclusive character ranges, optionally negated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub negated: bool,
    pub ranges: Vec<(char, char)>,
}

impl Class {
    pub fn matches(&self, c: char) -> bool {
        let hit = self.ranges.iter().any(|&(lo, hi)| lo <= c && c <= hi);
        hit != self.negated
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Empty,
    Char(char),
    Any,
    Class(Class),
    /// `^`
    Start,
    /// `$`
    End,
    /// Capturing group when `index` is `Some`, non-capturing otherwise.
    Group { node: Box<Node>, index: Option<usize> },
    Concat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat { node: Box<Node>, min: u32, max: Option<u32>, greedy: bool },
}

/// A parsed pattern plus the number of capture groups it contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    pub root: Node,
    pub group_count: usize,
}
