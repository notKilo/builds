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

### Sample output

```
$ hammer -n 2000 -c 20 http://127.0.0.1:8765/
Requests:   2000 in 2.464s (811.7 req/s)
Latency:    min 1.631ms  mean 20.166ms  max 2.048093s
Percentile: p50 6.176ms  p90 8.461ms  p95 9.482ms  p99 1.01398s
Status codes:
  200: 2000
Latency histogram:
   1.631ms ######################################## 1977
 206.277ms  0
 ...
 1.843447s # 3
```

(Run against Python's single-threaded `http.server`, hence the slow tail.)

## Development

```sh
go vet ./... && go test -race ./...
```

Layout: `internal/stats` (percentiles, histogram), `internal/runner` (worker pool,
rate limit), `internal/report` (text/JSON), `cmd/hammer` (CLI).
