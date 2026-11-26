# todofix

Find `TODO` / `FIXME` / `HACK` / `XXX` / `BUG` comments in a git repository and
report how old each one is, using `git blame` to find who wrote it and when.
Pure Python standard library; needs `git` on the PATH.

## Install

```sh
cd projects/todofix
pip install -e .
```

## Usage

```sh
todofix                       # scan the current repo, oldest comments first
todofix path/to/repo -g author          # group by author (also: tag, file, age)
todofix -f json                         # machine-readable output
todofix -f markdown > TODOS.md          # table for docs or PR comments
todofix -t fixme --min-age 90           # only FIXMEs at least 90 days old
todofix -i 'vendor/*' -i '*.min.js'     # skip files by glob
```

Example output:

```
  730d  TODO  src/db.py:41  handle negative offsets  (Ann)
   12d  FIXME src/api.go:9  recent | pipe  (Bob)

2 comment(s)
```

### CI mode

```sh
todofix --max-age 180 --max-count 50
```

Exits 1 (with a message on stderr) if any comment is older than 180 days or
there are more than 50. Exits 2 if the path is not a git repository. Use
`actions/checkout` with `fetch-depth: 0` so blame has history to work with.

## What counts as a comment

A tag must directly follow a comment leader for the file's language
(`#`, `//`, `/*`, `*`, `--`, `<!--`), so `print("TODO list")` and prose like
`# see the TODO below` are ignored. Optional forms are understood:
`TODO(alice) [ABC-12]: text` and `FIXME [#42] text`.

## Development

```sh
pip install -e '.[dev]' && pytest
```

Layout: `scanner.py` (comment detection), `discovery.py` (tracked files),
`blame.py` (porcelain parser), `report.py` (aggregation and rendering),
`cli.py`.
