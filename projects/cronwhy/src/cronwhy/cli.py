from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from .explain import explain
from .parser import CronError, parse
from .schedule import next_runs


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="cronwhy", description="Explain cron expressions and simulate runs")
    sub = p.add_subparsers(dest="cmd", required=True)
    for name, help_ in (("explain", "describe in English"), ("next", "list upcoming runs"),
                        ("check", "validate only (exit 1 if invalid)")):
        s = sub.add_parser(name, help=help_)
        s.add_argument("expr", help="cron expression, quoted")
        if name != "check":
            s.add_argument("--json", action="store_true", help="JSON output")
        if name == "next":
            s.add_argument("-n", "--count", type=int, default=5)
            s.add_argument("--tz", default="UTC", help="IANA timezone, default UTC")
            s.add_argument("--from", dest="start", help="start time, ISO 8601 (default: now)")
    w = sub.add_parser("serve", help="run the web UI")
    w.add_argument("--host", default="127.0.0.1")
    w.add_argument("--port", type=int, default=8000)
    return p


def main(argv: list[str] | None = None, now: datetime | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.cmd == "serve":
        from .web import serve
        serve(args.host, args.port)
        return 0
    try:
        expr = parse(args.expr)
        if args.cmd == "check":
            print("ok")
            return 0
        if args.cmd == "explain":
            text = explain(expr)
            print(json.dumps({"expression": args.expr, "explanation": text}) if args.json else text)
            return 0
        zone = ZoneInfo(args.tz)
        start = datetime.fromisoformat(args.start) if args.start else (now or datetime.now(zone))
        runs = next_runs(expr, start, args.count, args.tz)
    except CronError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    except ZoneInfoNotFoundError:
        print(f"error: unknown timezone {args.tz!r}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps([r.isoformat() for r in runs]))
    else:
        for r in runs:
            print(r.strftime("%a %Y-%m-%d %H:%M %Z (%z)"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
