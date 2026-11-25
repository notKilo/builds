//! Binary file format and disassembler.
//!
//! A program file is the 4-byte magic `TVM1` followed by the encoded
//! instructions from [`crate::opcode::encode`].

use crate::opcode::{decode, encode, DecodeError, Op};
use std::collections::BTreeMap;
use std::fmt;

pub const MAGIC: &[u8; 4] = b"TVM1";

#[derive(Debug, PartialEq, Eq)]
pub enum ImageError {
    BadMagic,
    Decode(DecodeError),
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::BadMagic => write!(f, "not a tinyvm program (bad magic)"),
            ImageError::Decode(e) => write!(f, "corrupt program: {e}"),
        }
    }
}

impl std::error::Error for ImageError {}

impl From<DecodeError> for ImageError {
    fn from(e: DecodeError) -> Self {
        ImageError::Decode(e)
    }
}

pub fn to_image(program: &[Op]) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    out.extend(encode(program));
    out
}

pub fn from_image(bytes: &[u8]) -> Result<Vec<Op>, ImageError> {
    let body = bytes
        .strip_prefix(MAGIC.as_slice())
        .ok_or(ImageError::BadMagic)?;
    Ok(decode(body)?)
}

fn jump_target(op: &Op) -> Option<usize> {
    match *op {
        Op::Jmp(t) | Op::Jz(t) | Op::Jnz(t) | Op::Call(t) => Some(t),
        _ => None,
    }
}

/// Renders a program as assembler source that reassembles to the same ops.
/// Every jump or call target gets a generated label (`L<index>`).
pub fn disassemble(program: &[Op]) -> String {
    let mut labels: BTreeMap<usize, String> = BTreeMap::new();
    for t in program.iter().filter_map(jump_target) {
        labels.entry(t).or_insert_with(|| format!("L{t}"));
    }

    let mut out = String::new();
    for (i, op) in program.iter().enumerate() {
        if let Some(name) = labels.get(&i) {
            out.push_str(name);
            out.push_str(":\n");
        }
        out.push_str("    ");
        match jump_target(op) {
            Some(t) => out.push_str(&format!("{} {}", op.mnemonic(), labels[&t])),
            None => out.push_str(&op.to_string()),
        }
        out.push('\n');
    }
    // A target one past the last instruction (jump to end) needs its label too.
    if let Some(name) = labels.get(&program.len()) {
        out.push_str(name);
        out.push_str(":\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::assemble;

    #[test]
    fn image_round_trip() {
        let prog = vec![Op::Push(1), Op::Call(0), Op::Halt];
        assert_eq!(from_image(&to_image(&prog)).unwrap(), prog);
    }

    #[test]
    fn rejects_bad_magic_and_corruption() {
        assert_eq!(from_image(b"nope"), Err(ImageError::BadMagic));
        assert_eq!(from_image(b""), Err(ImageError::BadMagic));
        let mut bytes = MAGIC.to_vec();
        bytes.push(0xff);
        assert!(matches!(from_image(&bytes), Err(ImageError::Decode(_))));
    }

    #[test]
    fn disassembly_reassembles_to_the_same_program() {
        let src = "
            push 3
            top: dup
            jz end
            dup
            print
            push 1
            sub
            call helper
            jmp top
            end: halt
            helper: lload 1
            pop
            ret
        ";
        let prog = assemble(src).unwrap();
        let text = disassemble(&prog);
        assert_eq!(assemble(&text).unwrap(), prog);
    }

    #[test]
    fn labels_only_where_targeted() {
        let text = disassemble(&[Op::Jmp(2), Op::Halt, Op::Halt]);
        assert_eq!(text, "    jmp L2\n    halt\nL2:\n    halt\n");
    }

    #[test]
    fn jump_to_end_label_is_emitted() {
        let prog = vec![Op::Jmp(1)];
        assert_eq!(assemble(&disassemble(&prog)).unwrap(), prog);
    }
}
