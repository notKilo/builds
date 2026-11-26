# envdiff

Compare configuration across environments. envdiff loads `.env`, `export`-style
shell files and `.json` configs, lines them up key by key, and tells you what is
missing, what drifted, and what looks unfinished. Pure Go standard library.

## Install

```sh
cd projects/envdiff
go build -o envdiff .
```

## Usage

```
envdiff [flags] FILE FILE [FILE...]
```

```console
$ envdiff dev.env prod.env
KEY          STATUS   dev.env     prod.env
DB_PASSWORD  differs  ****(b767)  ****(e087)
DEBUG        missing  true        (missing)
HOST         differs  localhost   db.prod
```

| Flag | Meaning |
|---|---|
| `-format table\|markdown\|json` | output format (default `table`) |
| `-only-diff` | hide keys that are identical everywhere |
| `-show-secrets` | print secret values (default: masked) |
| `-ignore A,B` | skip these keys |
| `-example FILE` | check each file against a `.env.example`: reports missing and undocumented keys |

Secret-looking keys (`*PASSWORD*`, `*TOKEN*`, `*_KEY`, ...) are masked as
`****(abcd)`; the 4-hex tag is a hash fingerprint, so you can see whether two
environments hold the same secret without seeing it.

Empty values and placeholders (`changeme`, `<...>`, `TODO`) are reported as warnings.

JSON files are flattened to dotted keys (`db.host`, `list.0`).

### Exit codes

`0` everything matches, `1` differences or warnings, `2` usage or I/O error —
so `envdiff -example .env.example .env.* ` works as a CI check.

## Tests

```sh
go test -race ./...
```
