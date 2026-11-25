//! Instruction set and binary encoding.
//!
//! Each instruction is one opcode byte, followed by an 8-byte little-endian
//! operand for instructions that take one.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Push(i64),
    Pop,
    Dup,
    Swap,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Eq,
    Lt,
    Gt,
    Not,
    Jmp(usize),
    Jz(usize),
    Jnz(usize),
    Print,
    Halt,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    UnknownOpcode { byte: u8, offset: usize },
    TruncatedOperand { offset: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnknownOpcode { byte, offset } => {
                write!(f, "unknown opcode 0x{byte:02x} at offset {offset}")
            }
            DecodeError::TruncatedOperand { offset } => {
                write!(f, "truncated operand at offset {offset}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

impl Op {
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Op::Push(_) => "push",
            Op::Pop => "pop",
            Op::Dup => "dup",
            Op::Swap => "swap",
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::Div => "div",
            Op::Mod => "mod",
            Op::Neg => "neg",
            Op::Eq => "eq",
            Op::Lt => "lt",
            Op::Gt => "gt",
            Op::Not => "not",
            Op::Jmp(_) => "jmp",
            Op::Jz(_) => "jz",
            Op::Jnz(_) => "jnz",
            Op::Print => "print",
            Op::Halt => "halt",
        }
    }

    fn code(&self) -> u8 {
        match self {
            Op::Halt => 0x00,
            Op::Push(_) => 0x01,
            Op::Pop => 0x02,
            Op::Dup => 0x03,
            Op::Swap => 0x04,
            Op::Add => 0x10,
            Op::Sub => 0x11,
            Op::Mul => 0x12,
            Op::Div => 0x13,
            Op::Mod => 0x14,
            Op::Neg => 0x15,
            Op::Eq => 0x20,
            Op::Lt => 0x21,
            Op::Gt => 0x22,
            Op::Not => 0x23,
            Op::Jmp(_) => 0x30,
            Op::Jz(_) => 0x31,
            Op::Jnz(_) => 0x32,
            Op::Print => 0x40,
        }
    }

    fn operand(&self) -> Option<i64> {
        match *self {
            Op::Push(v) => Some(v),
            Op::Jmp(a) | Op::Jz(a) | Op::Jnz(a) => Some(a as i64),
            _ => None,
        }
    }

    /// Encoded size in bytes.
    pub fn size(&self) -> usize {
        if self.operand().is_some() {
            9
        } else {
            1
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.operand() {
            Some(v) => write!(f, "{} {}", self.mnemonic(), v),
            None => write!(f, "{}", self.mnemonic()),
        }
    }
}

pub fn encode(program: &[Op]) -> Vec<u8> {
    let mut out = Vec::with_capacity(program.len() * 2);
    for op in program {
        out.push(op.code());
        if let Some(v) = op.operand() {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

pub fn decode(bytes: &[u8]) -> Result<Vec<Op>, DecodeError> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let code = bytes[i];
        let needs_operand = matches!(code, 0x01 | 0x30..=0x32);
        let operand = if needs_operand {
            let raw = bytes
                .get(i + 1..i + 9)
                .ok_or(DecodeError::TruncatedOperand { offset: i })?;
            i64::from_le_bytes(raw.try_into().unwrap())
        } else {
            0
        };
        let op = match code {
            0x00 => Op::Halt,
            0x01 => Op::Push(operand),
            0x02 => Op::Pop,
            0x03 => Op::Dup,
            0x04 => Op::Swap,
            0x10 => Op::Add,
            0x11 => Op::Sub,
            0x12 => Op::Mul,
            0x13 => Op::Div,
            0x14 => Op::Mod,
            0x15 => Op::Neg,
            0x20 => Op::Eq,
            0x21 => Op::Lt,
            0x22 => Op::Gt,
            0x23 => Op::Not,
            0x30 => Op::Jmp(operand as usize),
            0x31 => Op::Jz(operand as usize),
            0x32 => Op::Jnz(operand as usize),
            0x40 => Op::Print,
            byte => return Err(DecodeError::UnknownOpcode { byte, offset: i }),
        };
        i += op.size();
        ops.push(op);
    }
    Ok(ops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_shape() {
        let prog = vec![
            Op::Push(-42),
            Op::Dup,
            Op::Add,
            Op::Jz(7),
            Op::Jmp(0),
            Op::Jnz(1),
            Op::Print,
            Op::Halt,
        ];
        assert_eq!(decode(&encode(&prog)).unwrap(), prog);
    }

    #[test]
    fn rejects_unknown_opcode() {
        assert_eq!(
            decode(&[0x00, 0xff]),
            Err(DecodeError::UnknownOpcode { byte: 0xff, offset: 1 })
        );
    }

    #[test]
    fn rejects_truncated_operand() {
        assert_eq!(
            decode(&[0x01, 1, 2, 3]),
            Err(DecodeError::TruncatedOperand { offset: 0 })
        );
    }

    #[test]
    fn displays_like_assembly() {
        assert_eq!(Op::Push(5).to_string(), "push 5");
        assert_eq!(Op::Add.to_string(), "add");
    }
}
