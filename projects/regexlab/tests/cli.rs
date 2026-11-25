use std::process::Command;

fn run(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_regexlab")).args(args).output().unwrap();
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap())
}

#[test]
fn match_prints_captures() {
    let (out, code) = run(&["match", r"(\w+)@(\w+)", "to bob@host"]);
    assert_eq!(code, 0);
    assert_eq!(out, "0: [3..11] \"bob@host\"\n1: [3..6] \"bob\"\n2: [7..11] \"host\"\n");
}

#[test]
fn no_match_exits_1_and_bad_pattern_exits_2() {
    assert_eq!(run(&["match", "z", "abc"]).1, 1);
    assert_eq!(run(&["match", "(", "abc"]).1, 2);
}

#[test]
fn trace_marks_the_match() {
    let (out, _) = run(&["trace", "ab", "xab"]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[3].contains("MATCH"), "{out}");
    assert!(!lines[2].contains("MATCH"), "{out}");
}

#[test]
fn dot_is_a_digraph() {
    let (out, code) = run(&["dot", "a|b"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("digraph nfa {") && out.trim_end().ends_with('}'));
}
