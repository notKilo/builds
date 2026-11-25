//! The virtual machine: a value stack and a program counter.

use crate::opcode::Op;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    StackUnderflow { pc: usize },
    DivideByZero { pc: usize },
    BadJump { pc: usize, target: usize },
    Overflow { pc: usize },
    StepLimit(u64),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow { pc } => write!(f, "stack underflow at pc {pc}"),
            VmError::DivideByZero { pc } => write!(f, "division by zero at pc {pc}"),
            VmError::BadJump { pc, target } => {
                write!(f, "jump to invalid address {target} at pc {pc}")
            }
            VmError::Overflow { pc } => write!(f, "arithmetic overflow at pc {pc}"),
            VmError::StepLimit(n) => write!(f, "step limit of {n} exceeded"),
        }
    }
}

impl std::error::Error for VmError {}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Halted,
}

pub struct Vm {
    program: Vec<Op>,
    pub stack: Vec<i64>,
    pub pc: usize,
    pub output: Vec<i64>,
    pub steps: u64,
}

impl Vm {
    pub fn new(program: Vec<Op>) -> Self {
        Vm { program, stack: Vec::new(), pc: 0, output: Vec::new(), steps: 0 }
    }

    pub fn program(&self) -> &[Op] {
        &self.program
    }

    fn pop(&mut self) -> Result<i64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow { pc: self.pc })
    }

    fn binary(&mut self, f: impl Fn(i64, i64) -> Option<i64>) -> Result<(), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let r = f(a, b).ok_or(VmError::Overflow { pc: self.pc })?;
        self.stack.push(r);
        Ok(())
    }

    fn jump(&mut self, target: usize) -> Result<(), VmError> {
        if target > self.program.len() {
            return Err(VmError::BadJump { pc: self.pc, target });
        }
        self.pc = target;
        Ok(())
    }

    /// Executes one instruction. Running off the end counts as halting.
    pub fn step(&mut self) -> Result<Status, VmError> {
        let Some(&op) = self.program.get(self.pc) else {
            return Ok(Status::Halted);
        };
        self.steps += 1;
        let mut next = self.pc + 1;
        match op {
            Op::Push(v) => self.stack.push(v),
            Op::Pop => {
                self.pop()?;
            }
            Op::Dup => {
                let v = *self.stack.last().ok_or(VmError::StackUnderflow { pc: self.pc })?;
                self.stack.push(v);
            }
            Op::Swap => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(b);
                self.stack.push(a);
            }
            Op::Add => self.binary(i64::checked_add)?,
            Op::Sub => self.binary(i64::checked_sub)?,
            Op::Mul => self.binary(i64::checked_mul)?,
            Op::Div | Op::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err(VmError::DivideByZero { pc: self.pc });
                }
                let r = if op == Op::Div { a.checked_div(b) } else { a.checked_rem(b) };
                self.stack.push(r.ok_or(VmError::Overflow { pc: self.pc })?);
            }
            Op::Neg => {
                let a = self.pop()?;
                self.stack.push(a.checked_neg().ok_or(VmError::Overflow { pc: self.pc })?);
            }
            Op::Eq => self.binary(|a, b| Some((a == b) as i64))?,
            Op::Lt => self.binary(|a, b| Some((a < b) as i64))?,
            Op::Gt => self.binary(|a, b| Some((a > b) as i64))?,
            Op::Not => {
                let a = self.pop()?;
                self.stack.push((a == 0) as i64);
            }
            Op::Jmp(t) => {
                self.jump(t)?;
                next = self.pc;
            }
            Op::Jz(t) | Op::Jnz(t) => {
                let cond = self.pop()?;
                if (cond == 0) == matches!(op, Op::Jz(_)) {
                    self.jump(t)?;
                    next = self.pc;
                }
            }
            Op::Print => {
                let v = self.pop()?;
                self.output.push(v);
            }
            Op::Halt => return Ok(Status::Halted),
        }
        self.pc = next;
        Ok(Status::Running)
    }

    /// Runs until the program halts, errors, or exceeds `max_steps`.
    pub fn run(&mut self, max_steps: u64) -> Result<(), VmError> {
        while self.step()? == Status::Running {
            if self.steps >= max_steps {
                return Err(VmError::StepLimit(max_steps));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Op::*;

    fn run(prog: Vec<Op>) -> Result<Vm, VmError> {
        let mut vm = Vm::new(prog);
        vm.run(10_000).map(|_| vm)
    }

    #[test]
    fn arithmetic_order_of_operands() {
        let vm = run(vec![Push(10), Push(3), Sub, Push(7), Div, Print]).unwrap();
        assert_eq!(vm.output, vec![1]);
    }

    #[test]
    fn modulo_and_neg() {
        let vm = run(vec![Push(17), Push(5), Mod, Neg, Print]).unwrap();
        assert_eq!(vm.output, vec![-2]);
    }

    #[test]
    fn comparisons_and_not() {
        let vm = run(vec![Push(1), Push(2), Lt, Print, Push(1), Push(2), Gt, Not, Print]).unwrap();
        assert_eq!(vm.output, vec![1, 1]);
    }

    #[test]
    fn dup_and_swap() {
        let vm = run(vec![Push(1), Push(2), Swap, Dup, Print, Print, Print]).unwrap();
        assert_eq!(vm.output, vec![1, 1, 2]);
    }

    #[test]
    fn countdown_loop() {
        // n = 3; while n != 0 { print n; n -= 1 }
        let prog = vec![
            Push(3),
            Dup, Jz(9),
            Dup, Print,
            Push(1), Sub,
            Jmp(1),
            Halt,
            Pop,
        ];
        let vm = run(prog).unwrap();
        assert_eq!(vm.output, vec![3, 2, 1]);
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn errors_report_pc() {
        assert_eq!(run(vec![Add]).err(), Some(VmError::StackUnderflow { pc: 0 }));
        assert_eq!(
            run(vec![Push(1), Push(0), Div]).err(),
            Some(VmError::DivideByZero { pc: 2 })
        );
        assert_eq!(run(vec![Jmp(99)]).err(), Some(VmError::BadJump { pc: 0, target: 99 }));
        assert_eq!(
            run(vec![Push(i64::MAX), Push(1), Add]).err(),
            Some(VmError::Overflow { pc: 2 })
        );
    }

    #[test]
    fn step_limit_stops_infinite_loops() {
        let mut vm = Vm::new(vec![Jmp(0)]);
        assert_eq!(vm.run(100), Err(VmError::StepLimit(100)));
    }

    #[test]
    fn running_off_the_end_halts() {
        let vm = run(vec![Push(1)]).unwrap();
        assert_eq!(vm.stack, vec![1]);
    }
}
