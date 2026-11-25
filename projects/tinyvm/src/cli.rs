//! Command-line front end, kept in the library so it can be tested without
//! spawning a process.

use crate::asm::assemble;
use crate::debugger::Debugger;
use crate::disasm::{disassemble, from_image, to_image};
use crate::opcode::Op;
use crate::vm::Vm;
use std::fs;
use std::io::Write;

pub const USAGE: &str = "usage:
  tinyvm run <file.asm|file.tvm> [--max-steps N]
  tinyvm asm <input.asm> -o <output.tvm>
  tinyvm disasm <file.tvm>
  tinyvm debug <file.asm|file.tvm>";

pub const DEFAULT_MAX_STEPS: u64 = 10_000_000;

/// Exit codes: 0 success, 1 program/runtime failure, 2 bad usage.
pub const EXIT_OK: i32 = 0;
pub const EXIT_FAIL: i32 = 1;
pub const EXIT_USAGE: i32 = 2;

/// Loads a program from assembly source (`.asm`) or a binary image.
pub fn load_program(path: &str) -> Result<Vec<Op>, String> {
    if path.ends_with(".asm") {
        let source = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        assemble(&source).map_err(|e| format!("{path}: {e}"))
    } else {
        let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        from_image(&bytes).map_err(|e| format!("{path}: {e}"))
    }
}

pub fn run_cli(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let Some(cmd) = args.first() else {
        let _ = writeln!(err, "{USAGE}");
        return EXIT_USAGE;
    };
    let rest = &args[1..];
    let result = match cmd.as_str() {
        "run" => cmd_run(rest, out),
        "asm" => cmd_asm(rest),
        "disasm" => cmd_disasm(rest, out),
        "debug" => cmd_debug(rest, out),
        "-h" | "--help" | "help" => {
            let _ = writeln!(out, "{USAGE}");
            return EXIT_OK;
        }
        other => Err(CliError::Usage(format!("unknown command '{other}'"))),
    };
    match result {
        Ok(()) => EXIT_OK,
        Err(CliError::Usage(m)) => {
            let _ = writeln!(err, "error: {m}\n{USAGE}");
            EXIT_USAGE
        }
        Err(CliError::Failed(m)) => {
            let _ = writeln!(err, "error: {m}");
            EXIT_FAIL
        }
    }
}

enum CliError {
    Usage(String),
    Failed(String),
}

impl From<String> for CliError {
    fn from(m: String) -> Self {
        CliError::Failed(m)
    }
}

fn cmd_run(args: &[String], out: &mut dyn Write) -> Result<(), CliError> {
    let mut path = None;
    let mut max_steps = DEFAULT_MAX_STEPS;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--max-steps" {
            let v = it
                .next()
                .ok_or_else(|| CliError::Usage("--max-steps needs a value".into()))?;
            max_steps = v
                .parse()
                .map_err(|_| CliError::Usage(format!("invalid step count '{v}'")))?;
        } else if path.is_none() {
            path = Some(a);
        } else {
            return Err(CliError::Usage(format!("unexpected argument '{a}'")));
        }
    }
    let path = path.ok_or_else(|| CliError::Usage("run needs a file".into()))?;
    let mut vm = Vm::new(load_program(path)?);
    let result = vm.run(max_steps);
    // Show whatever the program printed even if it then failed.
    for v in &vm.output {
        writeln!(out, "{v}").map_err(|e| CliError::Failed(e.to_string()))?;
    }
    result.map_err(|e| CliError::Failed(e.to_string()))
}

fn cmd_debug(args: &[String], out: &mut dyn Write) -> Result<(), CliError> {
    let [path] = args else {
        return Err(CliError::Usage("debug <file>".into()));
    };
    let mut dbg = Debugger::new(Vm::new(load_program(path)?));
    let stdin = std::io::stdin();
    dbg.repl(&mut stdin.lock(), out)
        .map_err(|e| CliError::Failed(e.to_string()))
}

fn cmd_asm(args: &[String]) -> Result<(), CliError> {
    let (input, output) = match args {
        [i, flag, o] if flag == "-o" => (i, o),
        _ => return Err(CliError::Usage("asm <input.asm> -o <output.tvm>".into())),
    };
    let source = fs::read_to_string(input).map_err(|e| format!("{input}: {e}"))?;
    let program = assemble(&source).map_err(|e| format!("{input}: {e}"))?;
    fs::write(output, to_image(&program)).map_err(|e| format!("{output}: {e}"))?;
    Ok(())
}

fn cmd_disasm(args: &[String], out: &mut dyn Write) -> Result<(), CliError> {
    let [path] = args else {
        return Err(CliError::Usage("disasm <file.tvm>".into()));
    };
    let program = load_program(path)?;
    write!(out, "{}", disassemble(&program)).map_err(|e| CliError::Failed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn exec(args: &[&str]) -> (i32, String, String) {
        let (mut o, mut e) = (Vec::new(), Vec::new());
        let code = run_cli(&s(args), &mut o, &mut e);
        (
            code,
            String::from_utf8(o).unwrap(),
            String::from_utf8(e).unwrap(),
        )
    }

    fn tmp(name: &str, body: &str) -> String {
        let dir = std::env::temp_dir().join(format!("tinyvm-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        fs::write(&p, body).unwrap();
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn run_asm_file() {
        let p = tmp("a.asm", "push 2\npush 3\nadd\nprint\nhalt\n");
        assert_eq!(exec(&["run", &p]), (0, "5\n".into(), String::new()));
    }

    #[test]
    fn asm_then_run_and_disasm() {
        let src = tmp("b.asm", "push 7\nprint\n");
        let bin = src.replace("b.asm", "b.tvm");
        assert_eq!(exec(&["asm", &src, "-o", &bin]).0, 0);
        assert_eq!(exec(&["run", &bin]).1, "7\n");
        let (code, text, _) = exec(&["disasm", &bin]);
        assert_eq!(code, 0);
        assert!(text.contains("push 7"));
    }

    #[test]
    fn runtime_error_exits_1_but_keeps_output() {
        let p = tmp("c.asm", "push 1\nprint\npush 1\npush 0\ndiv\n");
        let (code, out, err) = exec(&["run", &p]);
        assert_eq!((code, out.as_str()), (1, "1\n"));
        assert!(err.contains("division by zero"));
    }

    #[test]
    fn assembly_error_reports_line() {
        let p = tmp("d.asm", "push 1\nbogus\n");
        let (code, _, err) = exec(&["run", &p]);
        assert_eq!(code, 1);
        assert!(err.contains("line 2"));
    }

    #[test]
    fn step_limit_flag() {
        let p = tmp("e.asm", "top: jmp top\n");
        let (code, _, err) = exec(&["run", &p, "--max-steps", "50"]);
        assert_eq!(code, 1);
        assert!(err.contains("step limit"));
    }

    #[test]
    fn usage_errors_exit_2() {
        assert_eq!(exec(&[]).0, 2);
        assert_eq!(exec(&["frobnicate"]).0, 2);
        assert_eq!(exec(&["run"]).0, 2);
        assert_eq!(exec(&["run", "x.asm", "--max-steps", "abc"]).0, 2);
    }

    #[test]
    fn missing_file_is_failure() {
        assert_eq!(exec(&["run", "/nonexistent/x.asm"]).0, 1);
    }
}
