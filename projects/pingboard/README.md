# pingboard

Self-hostable uptime monitor in Go (work in progress): HTTP/TCP checks, SQLite storage, dashboard.

Currently: JSON config, HTTP and TCP checkers, and a `check-once` command.

```
go run . check-once config.json
```
