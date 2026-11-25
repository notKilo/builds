# diffsheet

A semantic diff for tabular data. Line diffs fall apart on CSV and JSON: reordered rows and moved columns look like rewrites. diffsheet matches rows by key columns and reports what actually changed.

- Added, removed and changed rows, with old/new values per field
- Column additions and removals
- Composite keys, duplicate-key detection
- CSV, TSV and JSON inputs, mixable (`1` in JSON equals `"1"` in CSV)
- Numeric tolerance, trimming, case-insensitive compare, ignored columns
- Output as text, markdown, JSON or a self-contained HTML report
- Exit status 0 (identical), 1 (differences), 2 (error), so it can gate CI

Standard library only; Python 3.11+.

## Install

```sh
cd projects/diffsheet
pip install -e '.[dev]'
pytest
```

## Usage

```sh
diffsheet OLD NEW --key id [options]
```

| Option | Meaning |
|---|---|
| `-k, --key a,b` | key column(s) used to match rows (required) |
| `-i, --ignore a,b` | columns to skip |
| `-t, --tolerance N` | numbers within N are equal |
| `--trim`, `--ignore-case` | string normalisation |
| `-f, --format` | `text` (default), `markdown`, `json`, `html` |
| `--input-format` | `csv`, `tsv`, `json`; required for stdin (`-`) |
| `-s, --summary` | print only the summary line |

JSON input may be an array of objects, or an object of objects (the outer key becomes a `_key` column).

### Example

`old.csv`:

```
id,name,qty
1,apple,5
2,pear,7
3,fig,1
```

`new.csv`:

```
id,name,qty
3,fig,1
1,apple,6
4,kiwi,2
```

```
$ diffsheet old.csv new.csv -k id
- id=2
+ id=4
~ id=1
    qty: 5 -> 6
1 added, 1 removed, 1 changed, 1 unchanged
```

Reordering rows produced no noise. Use `-f html > report.html` for a shareable report, or `-f json` for tooling.

## Layout

- `loaders.py`: file parsing into a `Table`
- `diff.py`: the diff engine (`diff_tables`, `DiffOptions`)
- `render.py`: output formats
- `cli.py`: argument parsing and exit codes
