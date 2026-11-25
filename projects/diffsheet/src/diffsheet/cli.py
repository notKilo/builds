"""Command-line interface. Exit status: 0 identical, 1 differences, 2 error."""

from __future__ import annotations

import argparse
import sys
from typing import Sequence

from .diff import DiffError, DiffOptions, diff_tables
from .loaders import LoadError, load_table, parse_table
from .render import RENDERERS, summary_line


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="diffsheet", description="Semantic diff for CSV/JSON tables.")
    p.add_argument("old", help="old file ('-' reads stdin)")
    p.add_argument("new", help="new file ('-' reads stdin)")
    p.add_argument("-k", "--key", required=True, help="comma-separated key column(s)")
    p.add_argument("-i", "--ignore", default="", help="comma-separated columns to ignore")
    p.add_argument("-t", "--tolerance", type=float, default=0.0, help="max numeric difference treated as equal")
    p.add_argument("--trim", action="store_true", help="ignore leading/trailing whitespace")
    p.add_argument("--ignore-case", action="store_true", help="compare strings case-insensitively")
    p.add_argument("-f", "--format", choices=sorted(RENDERERS), default="text", help="output format")
    p.add_argument("--input-format", choices=["csv", "tsv", "json"], help="override input format detection")
    p.add_argument("-s", "--summary", action="store_true", help="print only the summary line")
    return p


def _load(path: str, fmt: str | None):
    if path == "-":
        if not fmt:
            raise LoadError("--input-format is required when reading stdin")
        return parse_table(sys.stdin.read(), fmt)
    return load_table(path, fmt)


def _split(value: str) -> list[str]:
    return [v.strip() for v in value.split(",") if v.strip()]


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        old = _load(args.old, args.input_format)
        new = _load(args.new, args.input_format)
        options = DiffOptions(
            keys=_split(args.key),
            ignore=set(_split(args.ignore)),
            tolerance=args.tolerance,
            trim=args.trim,
            ignore_case=args.ignore_case,
        )
        result = diff_tables(old, new, options)
    except (LoadError, DiffError, OSError) as exc:
        print(f"diffsheet: {exc}", file=sys.stderr)
        return 2
    print(summary_line(result) if args.summary else RENDERERS[args.format](result))
    return 1 if result.has_differences else 0


if __name__ == "__main__":
    sys.exit(main())
