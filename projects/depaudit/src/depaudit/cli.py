from __future__ import annotations

import argparse
import sys
from importlib import resources
from pathlib import Path

from .advisories import SEVERITIES, AdvisoryIndex, load_advisories
from .audit import audit_repo, filter_severity
from .config import load_config
from .licenses import Policy, load_license_map, normalize_id
from .report import render_json, render_table


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="depaudit", description="Audit dependencies for licenses and vulnerabilities.")
    p.add_argument("path", nargs="?", default=".", help="repository root (default: .)")
    p.add_argument("--format", choices=["table", "json"], default="table")
    p.add_argument("--min-severity", choices=SEVERITIES, help="hide advisories below this level")
    p.add_argument("--advisories", action="append", default=[], metavar="FILE", help="extra advisory JSON file (repeatable)")
    p.add_argument("--no-bundled", action="store_true", help="skip the bundled sample advisories")
    p.add_argument("--allow", action="append", default=[], metavar="LICENSE")
    p.add_argument("--deny", action="append", default=[], metavar="LICENSE")
    p.add_argument("--license-map", metavar="FILE", help='JSON of "ecosystem:name" -> license')
    p.add_argument("--fail-on-unknown", action="store_true", help="treat unknown licenses as violations")
    return p


def _resolve(name: str, base: Path) -> Path:
    path = Path(name)
    return path if path.is_absolute() else base / path


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    root = Path(args.path)
    if not root.is_dir():
        print(f"depaudit: not a directory: {root}", file=sys.stderr)
        return 2
    cfg = load_config(root)

    advisories = []
    try:
        if not args.no_bundled:
            bundled = resources.files("depaudit") / "data" / "advisories.json"
            with resources.as_file(bundled) as f:
                advisories += load_advisories(f)
        # Paths from the config file are relative to the audited repo; CLI paths to the cwd.
        paths = [_resolve(n, root) for n in cfg.advisories] + [Path(n) for n in args.advisories]
        for path in paths:
            advisories += load_advisories(path)
        if args.license_map:
            license_map = load_license_map(Path(args.license_map))
        elif cfg.license_map:
            license_map = load_license_map(_resolve(cfg.license_map, root))
        else:
            license_map = None
    except (OSError, ValueError, KeyError) as e:
        print(f"depaudit: {e}", file=sys.stderr)
        return 2

    policy = Policy(
        allow={normalize_id(x) for x in [*cfg.allow, *args.allow]},
        deny={normalize_id(x) for x in [*cfg.deny, *args.deny]},
        allow_unknown=cfg.allow_unknown and not args.fail_on_unknown,
    )
    findings = audit_repo(root, AdvisoryIndex(advisories), policy, license_map)
    findings = filter_severity(findings, args.min_severity or cfg.min_severity)
    print(render_json(findings) if args.format == "json" else render_table(findings))
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
