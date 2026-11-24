# hammer

A tiny HTTP load tester written in Go (standard library only). It runs concurrent
workers against a URL and reports throughput, status codes, errors, latency
percentiles and an ASCII latency histogram, as text or JSON.

## Install

```sh
cd projects/hammer
go build -o hammer ./cmd/hammer
```

## Usage

```sh
hammer [flags] URL
```

| Flag | Meaning |
|---|---|
| `-n` | total number of requests |
| `-d` | test duration, e.g. `10s` (set `-n` or `-d`) |
| `-c` | concurrent workers (default 10) |
| `-qps` | overall requests-per-second cap |
| `-X`, `-H`, `-body` | method, header (repeatable), request body |
| `-timeout` | per-request timeout (default 10s) |
| `-json` | machine-readable output |
| `-buckets` | histogram buckets (default 10) |

Exit codes: `0` ok, `1` some requests failed with transport errors, `2` bad usage.

### Example

```sh
hammer -c 20 -d 5s -qps 200 -H "Authorization: Bearer $TOKEN" http://localhost:8080/health
```

## Development

```sh
go vet ./... && go test -race ./...
```

Layout: `internal/stats` (percentiles, histogram), `internal/runner` (worker pool,
rate limit), `internal/report` (text/JSON), `cmd/hammer` (CLI).
