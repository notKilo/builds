"""License normalization and allow/deny policy evaluation.

Handles SPDX-style expressions with AND / OR and parentheses. An OR
expression passes if any alternative passes; an AND expression needs all.
"""
from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from .models import Package

UNKNOWN = "UNKNOWN"

_ALIASES = {
    "mit license": "MIT",
    "the mit license": "MIT",
    "apache 2.0": "Apache-2.0",
    "apache-2": "Apache-2.0",
    "apache license 2.0": "Apache-2.0",
    "apache software license": "Apache-2.0",
    "bsd": "BSD-3-Clause",
    "bsd license": "BSD-3-Clause",
    "new bsd": "BSD-3-Clause",
    "isc license": "ISC",
    "gpl-3.0": "GPL-3.0-only",
    "gpl-2.0": "GPL-2.0-only",
    "lgpl-3.0": "LGPL-3.0-only",
    "lgpl-2.1": "LGPL-2.1-only",
    "agpl-3.0": "AGPL-3.0-only",
    "mpl 2.0": "MPL-2.0",
}
_CANONICAL = {
    s.lower(): s
    for s in [
        "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "MPL-2.0",
        "GPL-2.0-only", "GPL-2.0-or-later", "GPL-3.0-only", "GPL-3.0-or-later",
        "LGPL-2.1-only", "LGPL-3.0-only", "AGPL-3.0-only", "Unlicense", "CC0-1.0",
        "0BSD", "Zlib", "Python-2.0", "BlueOak-1.0.0",
    ]
}


def normalize_id(raw: str) -> str:
    """Map a single license name to its canonical SPDX id when known."""
    key = raw.strip().lower()
    if key.endswith("+") and key[:-1] + "-or-later" in _CANONICAL:
        key = key[:-1] + "-or-later"
    key = _ALIASES.get(key, key).lower()
    return _CANONICAL.get(key, raw.strip())


_TOKEN = re.compile(r"\(|\)|[^\s()]+")


def parse_expression(expr: str):
    """Parse into nested tuples: ('and'|'or', [children]) or a license id str."""
    tokens = _TOKEN.findall(expr)
    pos = 0

    def parse_or():
        nonlocal pos
        parts = [parse_and()]
        while pos < len(tokens) and tokens[pos].upper() == "OR":
            pos += 1
            parts.append(parse_and())
        return parts[0] if len(parts) == 1 else ("or", parts)

    def parse_and():
        nonlocal pos
        parts = [parse_atom()]
        while pos < len(tokens) and tokens[pos].upper() == "AND":
            pos += 1
            parts.append(parse_atom())
        return parts[0] if len(parts) == 1 else ("and", parts)

    def parse_atom():
        nonlocal pos
        if pos >= len(tokens):
            raise ValueError(f"unexpected end of license expression: {expr!r}")
        tok = tokens[pos]
        pos += 1
        if tok == "(":
            node = parse_or()
            if pos >= len(tokens) or tokens[pos] != ")":
                raise ValueError(f"unbalanced parentheses: {expr!r}")
            pos += 1
            return node
        if tok == ")" or tok.upper() in ("AND", "OR"):
            raise ValueError(f"unexpected {tok!r} in {expr!r}")
        return normalize_id(tok)

    node = parse_or()
    if pos != len(tokens):
        raise ValueError(f"trailing tokens in license expression: {expr!r}")
    return node


def license_ids(expr: str) -> list[str]:
    """Flat list of license ids mentioned in an expression."""
    def walk(node):
        if isinstance(node, str):
            return [node]
        return [i for child in node[1] for i in walk(child)]

    try:
        return walk(parse_expression(expr))
    except ValueError:
        return [normalize_id(expr)]


@dataclass
class Policy:
    allow: set[str] = field(default_factory=set)  # empty means "allow anything not denied"
    deny: set[str] = field(default_factory=set)
    allow_unknown: bool = True

    def _ok(self, lic: str) -> bool:
        if lic == UNKNOWN:
            return self.allow_unknown
        if lic in self.deny:
            return False
        return not self.allow or lic in self.allow

    def permits(self, expr: str) -> bool:
        try:
            node = parse_expression(expr)
        except ValueError:
            return self._ok(normalize_id(expr))

        def ok(n) -> bool:
            if isinstance(n, str):
                return self._ok(n)
            op, children = n
            return any(map(ok, children)) if op == "or" else all(map(ok, children))

        return ok(node)


def load_license_map(path: Path) -> dict[str, str]:
    """JSON object of "ecosystem:name" -> license, used when a manifest has none."""
    return json.loads(path.read_text())


def resolve_license(pkg: Package, license_map: dict[str, str] | None = None) -> str:
    if pkg.license:
        return pkg.license
    if license_map:
        return license_map.get(f"{pkg.ecosystem}:{pkg.name}", UNKNOWN)
    return UNKNOWN
