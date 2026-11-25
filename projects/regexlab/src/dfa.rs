//! Lazily built DFA for fast yes/no matching (no captures).
//!
//! A DFA state is the set of NFA "kernel" program counters reached after
//! consuming a character. Epsilon closure (including `^`/`$` assertions) is
//! recomputed from the kernel when a transition or accept check is needed;
//! the results are cached per (state, char).

use crate::nfa::{Inst, Program};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Key {
    kernel: Vec<usize>,
    at_start: bool,
}

pub struct Dfa<'p> {
    prog: &'p Program,
    keys: Vec<Key>,
    ids: HashMap<Key, usize>,
    trans: HashMap<(usize, char), usize>,
    accept_mid: Vec<Option<bool>>,
    accept_end: Vec<Option<bool>>,
}

impl<'p> Dfa<'p> {
    pub fn new(prog: &'p Program) -> Self {
        Dfa {
            prog,
            keys: Vec::new(),
            ids: HashMap::new(),
            trans: HashMap::new(),
            accept_mid: Vec::new(),
            accept_end: Vec::new(),
        }
    }

    /// Number of DFA states materialised so far.
    pub fn state_count(&self) -> usize {
        self.keys.len()
    }

    fn intern(&mut self, key: Key) -> usize {
        if let Some(&id) = self.ids.get(&key) {
            return id;
        }
        let id = self.keys.len();
        self.keys.push(key.clone());
        self.ids.insert(key, id);
        self.accept_mid.push(None);
        self.accept_end.push(None);
        id
    }

    /// Unanchored search: does any substring of `text` match?
    pub fn is_match(&mut self, text: &str) -> bool {
        let mut state = self.intern(Key { kernel: vec![0], at_start: true });
        for c in text.chars() {
            if self.accepts(state, false) {
                return true;
            }
            state = self.step(state, c);
        }
        self.accepts(state, true)
    }

    fn accepts(&mut self, state: usize, at_end: bool) -> bool {
        let cache = if at_end { &self.accept_end[state] } else { &self.accept_mid[state] };
        if let Some(v) = *cache {
            return v;
        }
        let key = &self.keys[state];
        let hit = closure(self.prog, &key.kernel, key.at_start, at_end)
            .iter()
            .any(|&pc| self.prog.insts[pc] == Inst::Match);
        if at_end {
            self.accept_end[state] = Some(hit);
        } else {
            self.accept_mid[state] = Some(hit);
        }
        hit
    }

    fn step(&mut self, state: usize, c: char) -> usize {
        if let Some(&next) = self.trans.get(&(state, c)) {
            return next;
        }
        let key = &self.keys[state];
        let mut kernel: Vec<usize> = closure(self.prog, &key.kernel, key.at_start, false)
            .into_iter()
            .filter(|&pc| match &self.prog.insts[pc] {
                Inst::Char(x) => *x == c,
                Inst::Any => c != '\n',
                Inst::Class(cl) => cl.matches(c),
                _ => false,
            })
            .map(|pc| pc + 1)
            .collect();
        kernel.push(0); // unanchored: a new attempt may begin at every position
        kernel.sort_unstable();
        kernel.dedup();
        let next = self.intern(Key { kernel, at_start: false });
        self.trans.insert((state, c), next);
        next
    }
}

/// Epsilon closure of `kernel`, returning consuming instructions and `Match`.
fn closure(prog: &Program, kernel: &[usize], at_start: bool, at_end: bool) -> Vec<usize> {
    let mut seen = vec![false; prog.insts.len()];
    let mut out = Vec::new();
    let mut stack: Vec<usize> = kernel.iter().rev().copied().collect();
    while let Some(pc) = stack.pop() {
        if std::mem::replace(&mut seen[pc], true) {
            continue;
        }
        match &prog.insts[pc] {
            Inst::Jmp(x) => stack.push(*x),
            Inst::Split(x, y) => {
                stack.push(*y);
                stack.push(*x);
            }
            Inst::Save(_) => stack.push(pc + 1),
            Inst::AssertStart => {
                if at_start {
                    stack.push(pc + 1)
                }
            }
            Inst::AssertEnd => {
                if at_end {
                    stack.push(pc + 1)
                }
            }
            _ => out.push(pc),
        }
    }
    out.sort_unstable();
    out
}
