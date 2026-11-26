# trigrep

Trigram-indexed code search in Rust, standard library only. `trigrep index` writes a compact
index of a source tree; `trigrep search` uses it to skip every file that cannot match, then
verifies the remaining candidates line by line.

## Install

```sh
cargo install --path .        # or: cargo build --release
```

## Usage

```sh
trigrep index [DIR]                       # build, or incrementally update, DIR/.trigrep.idx
trigrep search [-i] [-F] [-c] [-C N] [-d DIR] PATTERN
trigrep stats [DIR]                       # index size and most common trigrams
```

- `-i` ignore case, `-F` treat PATTERN as a literal, `-c` per-file match counts,
  `-C N` N lines of context (grep-style `path:line:text` for matches, `path-line-text` for context).
- Exit status: 0 on matches, 1 on none, 2 on errors. A summary of how many files were scanned goes to stderr.

Example:

```console
$ trigrep index .
indexed 216 files, 22410 trigrams
$ trigrep search -F "fn render_file" -d .
projects/trigrep/src/output.rs:12:pub fn render_file(path: &str, ...
1 matches; scanned 1 of 216 indexed files
```

Patterns support literals, `.`, `*`, `+`, `?`, `|`, groups, character classes and escapes like `\s` `\w`
(own matcher in `src/regex.rs`, a backtracking engine).

## How it works

1. **Walk** (`walk.rs`): skips hidden entries, build directories and binary files.
2. **Index** (`index.rs`): every file's set of 3-byte sequences maps to a sorted posting list of
   file ids. Serialized with varint delta encoding.
3. **Plan** (`regex.rs`, `search.rs`): the matcher extracts literals every match must contain.
   Their trigrams are intersected across posting lists to give candidate files.
   Patterns with no literal of 3+ bytes fall back to scanning every file.
4. **Verify**: candidates are read and each line is run through the matcher. The index only
   prunes work; `tests/equivalence.rs` checks results equal a full scan.
5. **Update**: `trigrep index` again re-reads only files whose size or mtime changed.

Caveat: change detection trusts size and mtime, so an edit that preserves both (rare) is missed
until the file changes again. The index stores ASCII-lowercased trigrams, so one index serves both case-sensitive and case-insensitive queries (verification then applies exact case).

## Benchmark

`cargo run --release --example bench -- DIR PATTERN...` compares indexed and full-scan time
(and asserts they agree). On a 218-file tree:

| pattern | hits | candidates | indexed | full scan |
|---|---|---|---|---|
| `fn render` | 4 | 9/218 | 0.33 ms | 4.3 ms |
| `trigram` | 28 | 5/218 | 0.17 ms | 4.2 ms |
| `wal.*compact` | 0 | 1/218 | 0.03 ms | 4.0 ms |

The gap grows with tree size; the index file here is ~270 KB.

## Tests

```sh
cargo test
```
