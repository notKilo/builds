//! Two-pass text assembler.
//!
//! Syntax, one statement per line:
//!   ; comment
//!   .const NAME 10     named constant, usable as a push operand
//!   label:             marks the next instruction (may share a line with it)
//!   push 5 / jmp label / call label / add ...
//!   lload 0 / lstore 0     local slots (constants allowed as slot numbers)
//!   load / store           memory access, address on top of the stack

use crate::opcode::Op;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct AsmError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for AsmError {}

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, AsmError> {
    Err(AsmError {
        line,
        message: message.into(),
    })
}

/// A parsed statement whose jump target may still be a label name.
struct Pending<'a> {
    line: usize,
    mnemonic: &'a str,
    operand: Option<&'a str>,
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn assemble(source: &str) -> Result<Vec<Op>, AsmError> {
    let mut labels: HashMap<&str, usize> = HashMap::new();
    let mut consts: HashMap<&str, i64> = HashMap::new();
    let mut pending: Vec<Pending> = Vec::new();

    // Pass 1: collect labels, constants and raw statements.
    for (idx, raw) in source.lines().enumerate() {
        let line = idx + 1;
        let mut text = raw.split(';').next().unwrap_or("").trim();

        if let Some((head, rest)) = text.split_once(':') {
            let name = head.trim();
            if !is_ident(name) {
                return err(line, format!("invalid label name '{name}'"));
            }
            if labels.insert(name, pending.len()).is_some() {
                return err(line, format!("duplicate label '{name}'"));
            }
            text = rest.trim();
        }
        if text.is_empty() {
            continue;
        }

        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts[0] == ".const" {
            let [_, name, value] = parts[..] else {
                return err(line, ".const takes a name and a value");
            };
            if !is_ident(name) {
                return err(line, format!("invalid constant name '{name}'"));
            }
            let Ok(v) = value.parse::<i64>() else {
                return err(line, format!("invalid constant value '{value}'"));
            };
            if consts.insert(name, v).is_some() {
                return err(line, format!("duplicate constant '{name}'"));
            }
            continue;
        }
        if parts.len() > 2 {
            return err(line, format!("too many operands for '{}'", parts[0]));
        }
        pending.push(Pending {
            line,
            mnemonic: parts[0],
            operand: parts.get(1).copied(),
        });
    }

    // Pass 2: resolve operands.
    pending
        .iter()
        .map(|p| resolve(p, &labels, &consts))
        .collect()
}

fn resolve(
    p: &Pending,
    labels: &HashMap<&str, usize>,
    consts: &HashMap<&str, i64>,
) -> Result<Op, AsmError> {
    let line = p.line;
    let mnemonic = p.mnemonic.to_ascii_lowercase();

    let no_operand = |op: Op| match p.operand {
        None => Ok(op),
        Some(_) => err(line, format!("'{mnemonic}' takes no operand")),
    };
    let operand = || {
        p.operand
            .ok_or(())
            .or_else(|_| err(line, format!("'{mnemonic}' needs an operand")))
    };
    let target = || -> Result<usize, AsmError> {
        let name = operand()?;
        match labels.get(name) {
            Some(&addr) => Ok(addr),
            None => err(line, format!("undefined label '{name}'")),
        }
    };

    let slot = || -> Result<usize, AsmError> {
        let text = operand()?;
        let value = match consts.get(text) {
            Some(&v) => v,
            None => text
                .parse::<i64>()
                .or_else(|_| err(line, format!("invalid slot '{text}'")))?,
        };
        usize::try_from(value).or_else(|_| err(line, format!("slot must not be negative: {value}")))
    };

    match mnemonic.as_str() {
        "push" => {
            let text = operand()?;
            let value = match consts.get(text) {
                Some(&v) => v,
                None => match text.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => return err(line, format!("invalid number or constant '{text}'")),
                },
            };
            Ok(Op::Push(value))
        }
        "jmp" => Ok(Op::Jmp(target()?)),
        "jz" => Ok(Op::Jz(target()?)),
        "jnz" => Ok(Op::Jnz(target()?)),
        "call" => Ok(Op::Call(target()?)),
        "lload" => Ok(Op::LoadLocal(slot()?)),
        "lstore" => Ok(Op::StoreLocal(slot()?)),
        "ret" => no_operand(Op::Ret),
        "load" => no_operand(Op::Load),
        "store" => no_operand(Op::Store),
        "pop" => no_operand(Op::Pop),
        "dup" => no_operand(Op::Dup),
        "swap" => no_operand(Op::Swap),
        "add" => no_operand(Op::Add),
        "sub" => no_operand(Op::Sub),
        "mul" => no_operand(Op::Mul),
        "div" => no_operand(Op::Div),
        "mod" => no_operand(Op::Mod),
        "neg" => no_operand(Op::Neg),
        "eq" => no_operand(Op::Eq),
        "lt" => no_operand(Op::Lt),
        "gt" => no_operand(Op::Gt),
        "not" => no_operand(Op::Not),
        "print" => no_operand(Op::Print),
        "halt" => no_operand(Op::Halt),
        _ => err(line, format!("unknown instruction '{}'", p.mnemonic)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::Vm;

    #[test]
    fn assembles_and_runs_a_loop() {
        let src = "
            .const START 3
                push START
            loop: dup
                jz done      ; exit when zero
                dup
                print
                push 1
                sub
                jmp loop
            done: halt
        ";
        let prog = assemble(src).unwrap();
        let mut vm = Vm::new(prog);
        vm.run(1000).unwrap();
        assert_eq!(vm.output, vec![3, 2, 1]);
    }

    #[test]
    fn forward_and_backward_labels_resolve_to_instruction_index() {
        let prog = assemble("jmp end\nstart: halt\nend: jmp start").unwrap();
        assert_eq!(prog, vec![Op::Jmp(2), Op::Halt, Op::Jmp(1)]);
    }

    #[test]
    fn mnemonics_are_case_insensitive() {
        assert_eq!(assemble("PUSH 1\nAdd").unwrap(), vec![Op::Push(1), Op::Add]);
    }

    #[test]
    fn negative_numbers() {
        assert_eq!(assemble("push -5").unwrap(), vec![Op::Push(-5)]);
    }

    #[test]
    fn reports_errors_with_line_numbers() {
        let e = assemble("push 1\n\nbogus").unwrap_err();
        assert_eq!(
            (e.line, e.message.as_str()),
            (3, "unknown instruction 'bogus'")
        );

        let e = assemble("jmp nowhere").unwrap_err();
        assert_eq!(e.message, "undefined label 'nowhere'");

        let e = assemble("a: halt\na: halt").unwrap_err();
        assert_eq!((e.line, e.message.as_str()), (2, "duplicate label 'a'"));

        assert_eq!(
            assemble("push").unwrap_err().message,
            "'push' needs an operand"
        );
        assert_eq!(
            assemble("add 1").unwrap_err().message,
            "'add' takes no operand"
        );
        assert_eq!(
            assemble("push 1 2").unwrap_err().message,
            "too many operands for 'push'"
        );
        assert_eq!(
            assemble("push abc").unwrap_err().message,
            "invalid number or constant 'abc'"
        );
    }

    #[test]
    fn assembles_recursive_factorial() {
        let src = "
            push 6
            call fact
            print
            halt
            fact:   lstore 0
                    lload 0
                    push 2
                    lt
                    jnz base
                    lload 0
                    lload 0
                    push 1
                    sub
                    call fact
                    mul
                    ret
            base:   push 1
                    ret
        ";
        let mut vm = Vm::new(assemble(src).unwrap());
        vm.run(10_000).unwrap();
        assert_eq!(vm.output, vec![720]);
    }

    #[test]
    fn memory_and_slot_constants() {
        let src = ".const COUNTER 5\npush 9\npush COUNTER\nstore\npush COUNTER\nload\nlstore 2\nlload 2\nprint";
        let mut vm = Vm::new(assemble(src).unwrap());
        vm.run(100).unwrap();
        assert_eq!(vm.output, vec![9]);
    }

    #[test]
    fn rejects_bad_slots_and_call_targets() {
        assert_eq!(
            assemble("lload -1").unwrap_err().message,
            "slot must not be negative: -1"
        );
        assert_eq!(assemble("lload x").unwrap_err().message, "invalid slot 'x'");
        assert_eq!(
            assemble("call nope").unwrap_err().message,
            "undefined label 'nope'"
        );
        assert_eq!(
            assemble("ret 1").unwrap_err().message,
            "'ret' takes no operand"
        );
    }

    #[test]
    fn label_may_end_the_program() {
        assert_eq!(assemble("jmp end\nend:").unwrap(), vec![Op::Jmp(1)]);
    }
}
