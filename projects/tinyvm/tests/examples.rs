use std::path::PathBuf;
use tinyvm::asm::assemble;
use tinyvm::disasm::{disassemble, from_image, to_image};
use tinyvm::vm::Vm;

fn example(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn output_of(name: &str) -> Vec<i64> {
    let mut vm = Vm::new(assemble(&example(name)).unwrap());
    vm.run(1_000_000).unwrap();
    vm.output
}

#[test]
fn fibonacci() {
    assert_eq!(output_of("fib.asm"), vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
}

#[test]
fn factorial() {
    assert_eq!(output_of("factorial.asm"), vec![3_628_800]);
}

#[test]
fn primes() {
    assert_eq!(
        output_of("primes.asm"),
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
    );
}

#[test]
fn examples_survive_binary_and_text_round_trips() {
    for name in ["fib.asm", "factorial.asm", "primes.asm"] {
        let program = assemble(&example(name)).unwrap();
        assert_eq!(from_image(&to_image(&program)).unwrap(), program, "{name}");
        assert_eq!(assemble(&disassemble(&program)).unwrap(), program, "{name}");
    }
}
