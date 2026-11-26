from __future__ import annotations

import argparse
import sys
from pathlib import Path

from . import __version__
from .report import collect, render_json, render_markdown, render_text


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="todofix", description="Report how old your TODO comments are.")
    p.add_argument("path", nargs="?", default=".", help="git repository (default: .)")
    p.add_argument("-f", "--format", choices=["text", "json", "markdown"], default="text")
    p.add_argument("-g", "--group", choices=["author", "tag", "file", "age"],
                   help="group text output")
    p.add_argument("-i", "--ignore", action="append", default=[], metavar="GLOB",
                   help="skip files matching GLOB (repeatable)")
    p.add_argument("-t", "--tag", action="append", default=[], help="only these tags (repeatable)")
    p.add_argument("--min-age", type=int, default=0, metavar="DAYS", help="only comments at least this old")
    p.add_argument("--max-age", type=int, metavar="DAYS",
                   help="CI: exit 1 if any comment is older than DAYS")
    p.add_argument("--max-count", type=int, metavar="N",
                   help="CI: exit 1 if more than N comments are found")
    p.add_argument("--version", action="version", version=__version__)
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    root = Path(args.path)
    try:
        items = collect(root, args.ignore)
    except Exception as exc:  # not a git repo, git missing, ...
        print(f"todofix: {exc}", file=sys.stderr)
        return 2
    if args.tag:
        wanted = {t.upper() for t in args.tag}
        items = [i for i in items if i.finding.tag in wanted]
    items = [i for i in items if i.age_days >= args.min_age]

    if args.format == "json":
        print(render_json(items))
    elif args.format == "markdown":
        print(render_markdown(items))
    else:
        print(render_text(items, args.group))

    failed = False
    if args.max_age is not None:
        old = [i for i in items if i.age_days > args.max_age]
        if old:
            print(f"todofix: {len(old)} comment(s) older than {args.max_age} days", file=sys.stderr)
            failed = True
    if args.max_count is not None and len(items) > args.max_count:
        print(f"todofix: {len(items)} comments exceeds budget of {args.max_count}", file=sys.stderr)
        failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
