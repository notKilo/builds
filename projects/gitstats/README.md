# gitstats

A stdlib-only Go CLI that mines a git repository's history and reports:

- **Top churn**: files with the most added+deleted lines (renames are followed)
- **Hotspots**: files changed often *and recently* (recency-weighted score)
- **Bus factor**: per directory, the fewest authors covering over half the changed lines
- **Authors** and **commit activity** by weekday

Output is a text table, JSON, or a self-contained HTML page.

## Install

Requires Go 1.24+ and `git` on PATH.

    go install github.com/notkilo/builds/projects/gitstats/cmd/gitstats@latest

or from this directory: `go build ./cmd/gitstats`.

## Usage

    gitstats [-repo DIR] [-since DATE] [-path PATHSPEC] [-top N] [-depth N]
             [-half-life DAYS] [-format text|json|html]

Example:

    gitstats -repo ~/code/app -since "6 months ago" -top 5 -depth 2
    gitstats -format html > report.html

Sample output:

    Hotspots (recency-weighted)
    FILE                          SCORE  COMMITS  LAST CHANGE
    projects/throttle/src/lib.rs  11.27  6        2026-11-24

    Bus factor by directory
    DIR       FACTOR  LINES  OWNERS
    projects  1       9644   dev@example.com

## How scores work

Each change to a file contributes `ln(1 + lines changed) * 0.5^(age / half-life)`,
so a big edit yesterday outweighs a big edit a year ago. Default half-life is 90 days.

## Tests

    go test ./...

Tests include an end-to-end case that builds a temporary git repository.
