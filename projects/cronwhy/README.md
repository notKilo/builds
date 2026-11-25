# cronwhy

Explain cron expressions in plain English and simulate their next runs, in any timezone.
Pure Python standard library (3.11+), no dependencies.

## Install

```sh
pip install -e '.[dev]'   # from projects/cronwhy
```

## Usage

```console
$ cronwhy explain "0 9 * * mon-fri"
At 09:00, on Monday through Friday

$ cronwhy next "30 2 * * *" --tz America/New_York --from 2026-03-07T00:00:00-05:00 -n 3
Sat 2026-03-07 02:30 EST (-0500)
Sun 2026-03-08 03:30 EDT (-0400)
Mon 2026-03-09 02:30 EDT (-0400)

$ cronwhy check "61 * * * *"; echo $?
error: minute value 61 out of range 0-59
1

$ cronwhy serve --port 8000     # web UI at http://127.0.0.1:8000
```

`explain` and `next` accept `--json`. The web server also exposes
`GET /api/explain?expr=...&tz=...&n=...`.

## Supported syntax

Five fields (`minute hour day-of-month month day-of-week`) with `*`, lists (`1,5`),
ranges (`10-20`), steps (`*/15`, `5/20`, `10-30/5`), month and day names
(`jan`, `mon-fri`), `7` as Sunday, and the `@yearly @monthly @weekly @daily @hourly` aliases.

If both day-of-month and day-of-week are restricted, a day matches when *either* does
(classic cron behavior); if one is `*`, both must match.

## DST policy

Matching uses wall-clock time in the chosen zone. A time skipped by a spring-forward gap
runs once, shifted forward by the gap (`02:30` -> `03:30`). A time repeated on fall-back
runs once, on its first occurrence.

## Tests

```sh
pytest -q
```
