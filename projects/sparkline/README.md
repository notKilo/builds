# sparkline

A tiny self-hostable metrics collector. Clients push numbers to named series; the server keeps downsampled ring buffers in memory and serves them as SVG sparklines you can embed in READMEs and dashboards. TypeScript on Node 22, no runtime dependencies.

## Install

Requires Node >= 22.18 (runs TypeScript directly).

```sh
cd projects/sparkline
npm install        # dev dependencies only (typescript, @types/node)
npm test
```

## Usage

```sh
node src/cli.ts serve --port 8080 --snapshot data.json   # dashboard at http://127.0.0.1:8080/
node src/cli.ts push cpu 42 --url http://127.0.0.1:8080
```

`serve` options: `--port`, `--host`, `--snapshot FILE` (restore on start, autosave, save on SIGINT/SIGTERM), `--interval SECONDS` (autosave period, default 30).

### HTTP API

```sh
# ingest: line protocol or JSON (single object or array; optional "at" in ms)
curl -d $'cpu:1.5\nweb.hits:3' localhost:8080/ingest
curl -H 'content-type: application/json' -d '{"name":"cpu","value":2}' localhost:8080/ingest

curl localhost:8080/api/series                              # list names
curl 'localhost:8080/api/series/cpu?agg=p95&window=1h'      # avg|min|max|sum|count|p95
```

### Embedding sparklines

```md
![cpu](http://localhost:8080/spark/cpu.svg?style=area&color=3b82f6&w=200&h=40&window=3600000)
```

Query parameters: `style` (line, area, bars), `color` (hex without `#`), `w`, `h`, `window` (ms), `markers=0` to hide min/max markers.

## Notes

- Ingest has no authentication; run it on a trusted network or behind a proxy.
- p95 is approximated from bucket means because raw samples are not stored.
