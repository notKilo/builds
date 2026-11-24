# depaudit

A small command-line tool that audits a repository's dependencies for
**known-vulnerable versions** and **license policy violations**. It reads
lockfiles directly, needs no network access, and has no runtime dependencies
beyond the Python standard library.

Supported manifests (found recursively, skipping `node_modules`, `.git`, `target`, venvs and similar):

| File | Ecosystem | Notes |
|---|---|---|
| `requirements.txt` | PyPI | exact pins (`name==version`) only |
| `package-lock.json` | npm | lockfile v1, v2 and v3; license read from the lockfile |
| `Cargo.lock` | crates.io | registry and git crates; workspace crates are skipped |

## Install

Requires Python 3.11+.

```sh
cd projects/depaudit
pip install -e .          # add '.[dev]' to get pytest
```

## Usage

```sh
depaudit [PATH] [--format table|json] [--min-severity low|medium|high|critical]
         [--advisories FILE]... [--no-bundled]
         [--allow LICENSE]... [--deny LICENSE]... [--license-map FILE]
         [--fail-on-unknown]
```

Example, against the fixture repo used by the tests:

```console
$ depaudit tests/fixtures/mixed --deny GPL-3.0-only --license-map tests/fixtures/mixed/licenses.json
SEVERITY  TYPE           PACKAGE   VERSION  DETAIL        MESSAGE
--------  -------------  --------  -------  ------------  -----------------------------------------------------------
critical  vulnerability  smallvec  0.6.9    SAMPLE-0003   Sample memory-safety advisory (fixed in 0.6.14)
high      license        leftpad   1.3.0    GPL-3.0-only  license GPL-3.0-only violates policy
high      vulnerability  lodash    4.17.20  SAMPLE-0002   Sample prototype pollution advisory (fixed in 4.17.21)
medium    vulnerability  requests  2.5.0    SAMPLE-0001   Sample advisory for old requests releases (fixed in 2.20.0)

4 finding(s)
```

Use `--format json` for machine-readable output (`{"count": N, "findings": [...]}`).

**Exit codes:** `0` no findings, `1` findings reported, `2` usage or input error. This makes it
suitable as a CI gate.

### Advisories

A small sample database ships with the package (`SAMPLE-*` ids) so the tool works out of the box.
For real use, supply your own with `--advisories FILE` (repeatable) — a JSON list of:

```json
{"id": "EX-1", "ecosystem": "npm", "package": "lodash", "severity": "high",
 "summary": "Prototype pollution",
 "ranges": [{"introduced": "4.0.0", "fixed": "4.17.21"}]}
```

`introduced` is optional (defaults to the first version); a version is affected when
`introduced <= version < fixed`. Severities: `low`, `medium`, `high`, `critical`.
Pass `--no-bundled` to skip the sample data.

### Licenses

Licenses come from the lockfile where available (npm), otherwise from a license map:
a JSON object of `"Ecosystem:name": "SPDX expression"`, e.g. `{"PyPI:requests": "Apache-2.0"}`.
Packages with no known license are `UNKNOWN`, which passes unless `--fail-on-unknown` is set.
SPDX expressions with `AND`, `OR` and parentheses are evaluated: `MIT OR GPL-3.0-only` passes a
GPL deny list, `MIT AND GPL-3.0-only` does not. If `--allow` is given, only listed licenses pass.

### Configuration

Place a `.depaudit.toml` in the audited directory. CLI flags add to (or override) it.
Relative paths in the file resolve against the audited directory.

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]   # optional allow-list
deny = ["AGPL-3.0-only"]
allow_unknown = true
map = "license-map.json"

[severity]
minimum = "medium"        # hide advisories below this level (license findings always show)

[advisories]
files = ["advisories/osv-export.json"]
```

## Development

```sh
pip install -e '.[dev]'
pytest
```

Layout: `parsers.py` (manifests), `discovery.py` (repo walk), `advisories.py` (matching),
`version.py` (version comparison), `licenses.py` (SPDX policy), `audit.py` (findings),
`report.py` (table/JSON), `cli.py` and `config.py`. CI runs the tests on every change via
`.github/workflows/depaudit.yml`.
