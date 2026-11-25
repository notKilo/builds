//! Human-readable views of a compiled program: step-through traces and DOT.

use crate::dfa::closure;
use crate::nfa::{describe, Inst, Program};
use std::fmt::Write;

/// One row per input position: the NFA states alive before consuming it.
pub fn trace(prog: &Program, text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut kernel = vec![0usize];
    for pos in 0..=chars.len() {
        let at_end = pos == chars.len();
        let live = closure(prog, &kernel, pos == 0, at_end);
        let matched = live.iter().any(|&pc| prog.insts[pc] == Inst::Match);
        let label = chars.get(pos).map_or("<end>".to_string(), |c| format!("{c:?}"));
        let states: Vec<String> = live.iter().map(|pc| pc.to_string()).collect();
        let _ = writeln!(out, "{pos:>3} {label:<7} {{{}}}{}", states.join(","), if matched { "  MATCH" } else { "" });
        let Some(&c) = chars.get(pos) else { break };
        kernel = live
            .into_iter()
            .filter(|&pc| match &prog.insts[pc] {
                Inst::Char(x) => *x == c,
                Inst::Any => c != '\n',
                Inst::Class(cl) => cl.matches(c),
                _ => false,
            })
            .map(|pc| pc + 1)
            .collect();
        kernel.push(0);
    }
    out
}

/// Graphviz DOT for the NFA.
pub fn dot(prog: &Program) -> String {
    let mut out = String::from("digraph nfa {\n  rankdir=LR;\n  node [shape=circle];\n");
    for (i, inst) in prog.insts.iter().enumerate() {
        let label = describe(inst).replace('\\', "\\\\").replace('"', "\\\"");
        let shape = if *inst == Inst::Match { "doublecircle" } else { "circle" };
        let _ = writeln!(out, "  {i} [shape={shape}, label=\"{i}: {label}\"];");
        match inst {
            Inst::Split(a, b) => {
                let _ = writeln!(out, "  {i} -> {a} [label=\"1\"];\n  {i} -> {b} [label=\"2\"];");
            }
            Inst::Jmp(a) => {
                let _ = writeln!(out, "  {i} -> {a};");
            }
            Inst::Match => {}
            _ => {
                let _ = writeln!(out, "  {i} -> {};", i + 1);
            }
        }
    }
    out.push_str("}\n");
    out
}
