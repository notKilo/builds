# mdlinkcheck

A Markdown link checker with no runtime dependencies. It verifies relative file links
and `#heading` anchors against the real files, can optionally check `http(s)` URLs
(with concurrency, timeouts and an on-disk cache), and reports as text, JSON or SARIF.

## Install

Requires Node 22.18+ (it runs TypeScript directly).

```sh
cd projects/mdlinkcheck
npm install        # dev dependencies only (typescript, @types/node)
node src/cli.ts --help
```

## Usage

```sh
# check local links and anchors in a docs tree
node src/cli.ts README.md docs/

# also check external URLs, skipping local dev servers and generated files
node src/cli.ts docs/ -e --ignore-url http://localhost --ignore-path 'docs/generated/**'

# CI: external + strict (warnings fail) + no cache, SARIF for code scanning
node src/cli.ts . --ci -f sarif > mdlinkcheck.sarif
```

Example output:

```text
README.md:3:33 error [missing-file] File not found: docs/nope.md
README.md:3:56 error [missing-anchor] No heading for anchor #missing in docs/guide.md
9 links in 7 files checked: 2 error(s), 0 warning(s)
```

### Options

| Flag | Meaning |
|---|---|
| `-f, --format text\|json\|sarif` | output format (default `text`) |
| `-e, --external` | also check `http(s)` URLs |
| `--timeout <ms>`, `--concurrency <n>` | request tuning (10000 / 8) |
| `--cache-ttl <hours>`, `--cache-file <path>`, `--no-cache` | URL result cache (default 24h, `.mdlinkcheck-cache.json`) |
| `--root <dir>` | base for `/absolute` links and relative ignore globs (default cwd) |
| `--strict` | exit 1 on warnings (unreachable URLs) too |
| `--ci` | shorthand for `--external --strict --no-cache` |
| `--config <path>` | config file (default `./.mdlinkcheck.json` if present) |
| `--ignore-path <glob>` | skip matching files; repeatable; supports `*`, `**`, `?` |
| `--ignore-url <prefix>` | skip URLs starting with this prefix; repeatable |

### Config file

`.mdlinkcheck.json` supplies defaults; explicit flags win. Unknown keys are an error.

```json
{
  "ignorePaths": ["CHANGELOG.md", "vendor/**"],
  "ignoreUrls": ["http://localhost", "https://example.com/private"],
  "external": true,
  "timeout": 5000,
  "format": "text"
}
```

Keys: `ignorePaths`, `ignoreUrls`, `format`, `external`, `strict`, `timeout`, `concurrency`, `cacheTtl`.

### Exit codes

`0` clean, `1` problems found (errors; warnings too with `--strict`), `2` usage or config error.

### Rules

`missing-file`, `missing-anchor` (local); `broken-url` (HTTP error status, an error);
`unreachable-url` (network failure or timeout, a warning).

### GitHub code scanning

```yaml
- run: node projects/mdlinkcheck/src/cli.ts . --external -f sarif > mdlinkcheck.sarif
  continue-on-error: true
- uses: github/codeql-action/upload-sarif@v3
  with: { sarif_file: mdlinkcheck.sarif }
```

## Development

```sh
npm test          # node:test
npm run typecheck
```

Source layout: `extract.ts` (links), `slug.ts` (heading anchors), `local.ts` (file/anchor checks),
`http.ts` + `cache.ts` (URLs), `config.ts` + `glob.ts`, `report.ts`, `run.ts`, `cli.ts`.
