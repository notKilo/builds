# tinyvm

A small stack-based bytecode virtual machine in Rust, with a two-pass text
assembler, a binary image format, a disassembler and an interactive step
debugger. Standard library only, no dependencies.

## Install

```sh
cd projects/tinyvm
cargo build --release      # binary at target/release/tinyvm
```

## Usage

```sh
tinyvm run <file.asm|file.tvm> [--max-steps N]   # assemble if needed, then execute
tinyvm asm <input.asm> -o <output.tvm>           # write a binary image
tinyvm disasm <file.tvm>                         # print assembler source
tinyvm debug <file.asm|file.tvm>                 # interactive debugger
```

Exit codes: `0` success, `1` assembly or runtime error (anything printed
before the error is still shown), `2` bad usage.

### Example

```sh
$ cargo run -q -- run examples/factorial.asm
3628800

$ cargo run -q -- asm examples/fib.asm -o fib.tvm
$ cargo run -q -- run fib.tvm
0
1
1
2
...
```

Debugging a program:

```
$ printf 'break 3\ncontinue\nstack\nquit\n' | cargo run -q -- debug examples/fib.asm
(tvm) breakpoint set at 3
(tvm) breakpoint at 3
   3: lstore 1
(tvm) [1]
```

## The language

One statement per line. `;` starts a comment.

```
.const LIMIT 50      ; named constant, usable as an operand
loop:   lload 0      ; labels may share a line with an instruction
        jnz loop
```

| Group | Instructions |
|---|---|
| Stack | `push <n>` `pop` `dup` `swap` |
| Arithmetic | `add` `sub` `mul` `div` `mod` `neg` (checked: overflow and divide by zero are errors) |
| Comparison | `eq` `lt` `gt` `not` (push 1 or 0) |
| Control flow | `jmp` `jz` `jnz` `call` `ret` `halt` |
| Locals | `lload <slot>` `lstore <slot>` (private to each call frame, default 0) |
| Memory | `load` `store` (address on top of the stack; 1024 cells) |
| Output | `print` |

Binary operations take the left operand deeper in the stack, so
`push 10; push 3; sub` leaves `7`. Running off the end of the program counts
as halting.

## Debugger commands

`step [n]`, `continue`, `break <pc>`, `delete <pc>`, `watch <addr>` /
`unwatch <addr>` (stop when a memory cell changes), `stack`, `locals`,
`mem <addr> [n]`, `list`, `where`, `output`, `help`, `quit`.

## Layout

- `src/opcode.rs` instruction set and encoding
- `src/vm.rs` the machine
- `src/asm.rs` assembler, `src/disasm.rs` image format and disassembler
- `src/debugger.rs` debugger, `src/cli.rs` command line
- `examples/` Fibonacci, factorial, primes

## Tests

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```
