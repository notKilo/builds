//! Interactive step debugger. `Debugger::execute` handles one command line
//! and returns text, so the whole thing is testable without a terminal.

use crate::vm::{Status, Vm};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::{BufRead, Write};

pub const HELP: &str = "commands:
  step [n]        execute n instructions (default 1)
  continue        run until a breakpoint, watch change, halt or error
  break <pc>      set a breakpoint      delete <pc>   remove it
  watch <addr>    stop when memory[addr] changes   unwatch <addr>
  stack           show the value stack  locals   show current locals
  mem <addr> [n]  show n memory cells   list      show the program
  where           show pc and next op   output    show printed values
  help            this text             quit";

pub struct Debugger {
    vm: Vm,
    breakpoints: BTreeSet<usize>,
    watches: BTreeSet<usize>,
    halted: bool,
}

/// What the REPL should do after a command.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Continue(String),
    Quit,
}

impl Debugger {
    pub fn new(vm: Vm) -> Self {
        Debugger {
            vm,
            breakpoints: BTreeSet::new(),
            watches: BTreeSet::new(),
            halted: false,
        }
    }

    fn here(&self) -> String {
        match self.vm.program().get(self.vm.pc) {
            Some(op) => format!("{:>4}: {op}", self.vm.pc),
            None => format!("{:>4}: <end>", self.vm.pc),
        }
    }

    /// Executes one instruction, reporting why it stopped, if it did.
    fn step_once(&mut self) -> Result<Option<String>, String> {
        let before: Vec<i64> = self.watches.iter().map(|&a| self.vm.memory[a]).collect();
        let status = self.vm.step().map_err(|e| e.to_string())?;
        if status == Status::Halted || self.vm.pc >= self.vm.program().len() {
            self.halted = true;
            return Ok(Some("program halted".into()));
        }
        for (&addr, old) in self.watches.iter().zip(before) {
            let new = self.vm.memory[addr];
            if new != old {
                return Ok(Some(format!("watch: memory[{addr}] {old} -> {new}")));
            }
        }
        Ok(None)
    }

    fn parse(arg: Option<&&str>, what: &str) -> Result<usize, String> {
        arg.ok_or_else(|| format!("{what} required"))?
            .parse()
            .map_err(|_| format!("invalid {what}"))
    }

    pub fn execute(&mut self, line: &str) -> Outcome {
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some(&cmd) = words.first() else {
            return Outcome::Continue(String::new());
        };
        if matches!(cmd, "quit" | "q" | "exit") {
            return Outcome::Quit;
        }
        Outcome::Continue(
            self.run_command(cmd, &words[1..])
                .unwrap_or_else(|e| format!("error: {e}")),
        )
    }

    fn run_command(&mut self, cmd: &str, args: &[&str]) -> Result<String, String> {
        let live = matches!(cmd, "step" | "s" | "continue" | "c");
        if live && self.halted {
            return Ok("program has halted".into());
        }
        match cmd {
            "step" | "s" => {
                let n = if args.is_empty() {
                    1
                } else {
                    Self::parse(args.first(), "count")?
                };
                for _ in 0..n {
                    if let Some(stop) = self.step_once()? {
                        return Ok(format!("{stop}\n{}", self.here()));
                    }
                }
                Ok(self.here())
            }
            "continue" | "c" => loop {
                if let Some(stop) = self.step_once()? {
                    return Ok(format!("{stop}\n{}", self.here()));
                }
                if self.breakpoints.contains(&self.vm.pc) {
                    return Ok(format!("breakpoint at {}\n{}", self.vm.pc, self.here()));
                }
            },
            "break" | "b" => {
                let pc = Self::parse(args.first(), "pc")?;
                if pc >= self.vm.program().len() {
                    return Err(format!("pc {pc} is past the end of the program"));
                }
                self.breakpoints.insert(pc);
                Ok(format!("breakpoint set at {pc}"))
            }
            "delete" => {
                let pc = Self::parse(args.first(), "pc")?;
                let had = self.breakpoints.remove(&pc);
                Ok(if had {
                    format!("breakpoint {pc} removed")
                } else {
                    format!("no breakpoint at {pc}")
                })
            }
            "watch" | "unwatch" => {
                let addr = Self::parse(args.first(), "address")?;
                if addr >= self.vm.memory.len() {
                    return Err(format!("address {addr} out of range"));
                }
                if cmd == "watch" {
                    self.watches.insert(addr);
                    Ok(format!("watching memory[{addr}]"))
                } else {
                    self.watches.remove(&addr);
                    Ok(format!("no longer watching memory[{addr}]"))
                }
            }
            "stack" => Ok(format!("{:?}", self.vm.stack)),
            "locals" => {
                let frame = self.vm.frames.last().expect("frame");
                Ok(format!(
                    "depth {} locals {:?}",
                    self.vm.frames.len() - 1,
                    frame.locals
                ))
            }
            "mem" => {
                let addr = Self::parse(args.first(), "address")?;
                let n = if args.len() > 1 {
                    Self::parse(args.get(1), "count")?
                } else {
                    1
                };
                let end = addr.saturating_add(n).min(self.vm.memory.len());
                if addr >= end {
                    return Err(format!("address {addr} out of range"));
                }
                Ok(format!(
                    "memory[{addr}..{end}] = {:?}",
                    &self.vm.memory[addr..end]
                ))
            }
            "where" | "w" => Ok(self.here()),
            "output" => Ok(format!("{:?}", self.vm.output)),
            "list" | "l" => {
                let mut s = String::new();
                for (i, op) in self.vm.program().iter().enumerate() {
                    let marker = if i == self.vm.pc {
                        "=>"
                    } else if self.breakpoints.contains(&i) {
                        " *"
                    } else {
                        "  "
                    };
                    let _ = writeln!(s, "{marker} {i:>4}: {op}");
                }
                Ok(s.trim_end().to_string())
            }
            "help" | "h" | "?" => Ok(HELP.to_string()),
            other => Err(format!("unknown command '{other}' (try help)")),
        }
    }

    /// Reads commands until EOF or `quit`.
    pub fn repl(&mut self, input: &mut dyn BufRead, out: &mut dyn Write) -> std::io::Result<()> {
        writeln!(
            out,
            "tinyvm debugger - type help for commands\n{}",
            self.here()
        )?;
        let mut line = String::new();
        loop {
            write!(out, "(tvm) ")?;
            out.flush()?;
            line.clear();
            if input.read_line(&mut line)? == 0 {
                writeln!(out)?;
                return Ok(());
            }
            match self.execute(&line) {
                Outcome::Quit => return Ok(()),
                Outcome::Continue(text) if text.is_empty() => {}
                Outcome::Continue(text) => writeln!(out, "{text}")?,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::assemble;

    fn dbg(src: &str) -> Debugger {
        Debugger::new(Vm::new(assemble(src).unwrap()))
    }

    fn text(d: &mut Debugger, cmd: &str) -> String {
        match d.execute(cmd) {
            Outcome::Continue(t) => t,
            Outcome::Quit => "QUIT".into(),
        }
    }

    const PROG: &str = "push 1\npush 2\nadd\nprint\npush 9\npush 5\nstore\nhalt\n";

    #[test]
    fn step_and_stack() {
        let mut d = dbg(PROG);
        assert!(text(&mut d, "step 2").contains("2: add"));
        assert_eq!(text(&mut d, "stack"), "[1, 2]");
        text(&mut d, "step");
        assert_eq!(text(&mut d, "stack"), "[3]");
    }

    #[test]
    fn breakpoint_stops_continue() {
        let mut d = dbg(PROG);
        text(&mut d, "break 3");
        let out = text(&mut d, "continue");
        assert!(out.contains("breakpoint at 3"), "{out}");
        assert_eq!(text(&mut d, "stack"), "[3]");
        assert!(text(&mut d, "list").contains("=>    3: print"));
    }

    #[test]
    fn watch_triggers_on_change() {
        let mut d = dbg(PROG);
        text(&mut d, "watch 5");
        let out = text(&mut d, "continue");
        assert!(out.contains("memory[5] 0 -> 9"), "{out}");
        assert_eq!(text(&mut d, "mem 5 2"), "memory[5..7] = [9, 0]");
    }

    #[test]
    fn halting_is_reported_and_sticky() {
        let mut d = dbg(PROG);
        assert!(text(&mut d, "continue").contains("program halted"));
        assert_eq!(text(&mut d, "output"), "[3]");
        assert_eq!(text(&mut d, "step"), "program has halted");
    }

    #[test]
    fn errors_are_reported_not_fatal() {
        let mut d = dbg("add\n");
        assert!(text(&mut d, "step").contains("stack underflow"));
        assert!(text(&mut d, "break x").contains("invalid pc"));
        assert!(text(&mut d, "break 99").contains("past the end"));
        assert!(text(&mut d, "nope").contains("unknown command"));
        assert_eq!(text(&mut d, "quit"), "QUIT");
    }

    #[test]
    fn locals_show_call_depth() {
        let mut d = dbg("call f\nhalt\nf: push 4\nlstore 0\nret\n");
        text(&mut d, "step 2");
        assert_eq!(text(&mut d, "locals"), "depth 1 locals []");
        text(&mut d, "step");
        assert_eq!(text(&mut d, "locals"), "depth 1 locals [4]");
    }

    #[test]
    fn repl_reads_until_eof() {
        let mut d = dbg(PROG);
        let mut input = std::io::Cursor::new(b"step\nstack\n".to_vec());
        let mut out = Vec::new();
        d.repl(&mut input, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("(tvm)") && s.contains("[1]"));
    }
}
