# regexlab

A regular-expression engine written from scratch in Rust (std only), with a CLI that shows how a match unfolds.

- **Parser** to an AST: literals, `.`, classes (`[a-z]`, `[^x]`), `\d \w \s` (and negations), groups (capturing and `(?:...)`), alternation, `* + ? {n} {n,} {n,m}`, lazy variants, `^` and `$`.
- **Thompson NFA** compiler with priority splits.
- **Pike VM**: simulates all threads in lock step. Linear time, leftmost-first (Perl-like) semantics, capture groups. No catastrophic backtracking, so `(a+)+b` is fast.
- **Lazy DFA** for yes/no matching, built on demand and cached; tested against the Pike VM on random inputs.
- **CLI** that matches, traces and exports the automaton as Graphviz DOT.

Not supported: backreferences, lookaround, Unicode classes beyond literal characters, flags.

## Install

```
cargo install --path .      # installs the `regexlab` binary
```

## Library

```rust
use regexlab::Regex;

let re = Regex::new(r"(\d{4})-(\d\d)-(\d\d)").unwrap();
let caps = re.captures("released 2026-11-25.").unwrap();
assert_eq!(caps.get(1), Some("2026"));
assert!(re.is_match_dfa("2026-11-25"));
```

Spans are byte offsets, so slicing the haystack with them is always valid.

## CLI

```
$ regexlab match '(\w+)@(\w+)' 'to bob@host'
0: [3..11] "bob@host"
1: [3..6] "bob"
2: [7..11] "host"

$ regexlab trace 'a(b|c)+d' 'xabcd'
  0 'x'     {1}
  1 'a'     {1}
  2 'b'     {1,4,6}
  3 'c'     {1,11,13,16}
  4 'd'     {1,11,13,16}
  5 <end>   {1,18}  MATCH
```

Each trace row lists the NFA states that are alive before that character is read; `regexlab prog <pattern>` prints what the numbers mean, and `regexlab dot <pattern> | dot -Tsvg > nfa.svg` draws the graph. Other commands: `all` (every match). Exit codes: 0 match, 1 no match, 2 bad usage or pattern.

## Tests and benchmark

```
cargo test
cargo run --release --example bench   # Pike VM vs DFA timings
```
