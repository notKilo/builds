//! Thompson-style NFA program compiled from the AST.

use crate::ast::{Ast, Class, Node};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inst {
    Char(char),
    Any,
    Class(Class),
    /// Try the first target, then the second (priority order).
    Split(usize, usize),
    Jmp(usize),
    /// Record the current position in capture slot `n`.
    Save(usize),
    AssertStart,
    AssertEnd,
    Match,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub insts: Vec<Inst>,
    /// Number of capture groups, excluding the implicit group 0.
    pub group_count: usize,
}

impl Program {
    pub fn slot_count(&self) -> usize {
        (self.group_count + 1) * 2
    }
}

pub fn compile(ast: &Ast) -> Program {
    let mut c = Compiler { insts: Vec::new() };
    c.emit(Inst::Save(0));
    c.node(&ast.root);
    c.emit(Inst::Save(1));
    c.emit(Inst::Match);
    Program { insts: c.insts, group_count: ast.group_count }
}

struct Compiler {
    insts: Vec<Inst>,
}

impl Compiler {
    fn emit(&mut self, i: Inst) -> usize {
        self.insts.push(i);
        self.insts.len() - 1
    }

    fn pc(&self) -> usize {
        self.insts.len()
    }

    fn node(&mut self, n: &Node) {
        match n {
            Node::Empty => {}
            Node::Char(c) => {
                self.emit(Inst::Char(*c));
            }
            Node::Any => {
                self.emit(Inst::Any);
            }
            Node::Class(cl) => {
                self.emit(Inst::Class(cl.clone()));
            }
            Node::Start => {
                self.emit(Inst::AssertStart);
            }
            Node::End => {
                self.emit(Inst::AssertEnd);
            }
            Node::Group { node, index } => match index {
                Some(i) => {
                    self.emit(Inst::Save(i * 2));
                    self.node(node);
                    self.emit(Inst::Save(i * 2 + 1));
                }
                None => self.node(node),
            },
            Node::Concat(items) => items.iter().for_each(|i| self.node(i)),
            Node::Alt(branches) => self.alt(branches),
            Node::Repeat { node, min, max, greedy } => self.repeat(node, *min, *max, *greedy),
        }
    }

    fn alt(&mut self, branches: &[Node]) {
        let mut jumps = Vec::new();
        for (i, b) in branches.iter().enumerate() {
            if i + 1 < branches.len() {
                let split = self.emit(Inst::Split(0, 0));
                self.node(b);
                jumps.push(self.emit(Inst::Jmp(0)));
                let next = self.pc();
                self.insts[split] = Inst::Split(split + 1, next);
            } else {
                self.node(b);
            }
        }
        let end = self.pc();
        for j in jumps {
            self.insts[j] = Inst::Jmp(end);
        }
    }

    fn split(&self, body: usize, exit: usize, greedy: bool) -> Inst {
        if greedy { Inst::Split(body, exit) } else { Inst::Split(exit, body) }
    }

    fn repeat(&mut self, node: &Node, min: u32, max: Option<u32>, greedy: bool) {
        for _ in 0..min {
            self.node(node);
        }
        match max {
            None => {
                // L1: split L2, L3; L2: body; jmp L1; L3:
                let l1 = self.emit(Inst::Split(0, 0));
                self.node(node);
                self.emit(Inst::Jmp(l1));
                let l3 = self.pc();
                self.insts[l1] = self.split(l1 + 1, l3, greedy);
            }
            Some(max) => {
                // Each optional copy may bail out to the common end.
                let mut splits = Vec::new();
                for _ in min..max {
                    splits.push(self.emit(Inst::Split(0, 0)));
                    self.node(node);
                }
                let end = self.pc();
                for s in splits {
                    self.insts[s] = self.split(s + 1, end, greedy);
                }
            }
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, inst) in self.insts.iter().enumerate() {
            writeln!(f, "{i:03} {}", describe(inst))?;
        }
        Ok(())
    }
}

pub fn describe(inst: &Inst) -> String {
    match inst {
        Inst::Char(c) => format!("char {c:?}"),
        Inst::Any => "any".into(),
        Inst::Class(c) => {
            let body: Vec<String> = c
                .ranges
                .iter()
                .map(|&(a, b)| if a == b { format!("{a:?}") } else { format!("{a:?}-{b:?}") })
                .collect();
            format!("class {}[{}]", if c.negated { "^" } else { "" }, body.join(" "))
        }
        Inst::Split(a, b) => format!("split {a:03}, {b:03}"),
        Inst::Jmp(a) => format!("jmp {a:03}"),
        Inst::Save(n) => format!("save {n}"),
        Inst::AssertStart => "assert ^".into(),
        Inst::AssertEnd => "assert $".into(),
        Inst::Match => "match".into(),
    }
}
