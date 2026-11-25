# kvlite

A small embeddable key-value store in Go, built on an append-only log with an in-memory index (the Bitcask idea). Standard library only.

- Every write appends a CRC32C-checked record to a segment file; reads are one `ReadAt`.
- On open the log is replayed to rebuild the index. A torn or corrupt tail on the newest segment (a crash mid-write) is truncated; corruption elsewhere is reported.
- Segments roll at a size limit. `Compact` merges closed segments and drops overwritten and deleted records.
- Safe for concurrent use. Keys and values are limited to 64 KiB and 256 MiB.

## Install

```
go install github.com/notkilo/builds/projects/kvlite/cmd/kvlite@latest
```

## CLI

```
$ kvlite -dir ./data put user:1 alice
$ kvlite -dir ./data put user:2 bob
$ kvlite -dir ./data scan user:
user:1=alice
user:2=bob
$ kvlite -dir ./data del user:1
$ kvlite -dir ./data compact
$ kvlite -dir ./data stats
keys: 1
segments: 2
...
```

## Library

```go
s, err := kvlite.Open("data", kvlite.Options{MaxSegmentSize: 8 << 20})
if err != nil { /* ... */ }
defer s.Close()

s.Put([]byte("k"), []byte("v"))
v, err := s.Get([]byte("k"))   // kvlite.ErrNotFound if absent
s.Scan([]byte("k"), func(k, v []byte) bool { return true })
s.Compact()
```

`Options.SyncEvery` fsyncs on every write (the CLI enables it).

## Limits

The whole key set must fit in memory, and `Compact` blocks other operations while it runs. There is no multi-process locking: open a directory from one process at a time.

## Tests and benchmarks

```
go test -race ./...
go test -bench . -run xxx
```

On a small VM: about 0.8 µs per 256-byte `Put` (no fsync) and per `Get`.
