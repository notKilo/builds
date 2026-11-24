"""Render findings as a text table or JSON."""
from __future__ import annotations

import json

from .advisories import SEVERITIES
from .audit import Finding

HEADERS = ("SEVERITY", "TYPE", "PACKAGE", "VERSION", "DETAIL", "MESSAGE")


def _sorted(findings: list[Finding]) -> list[Finding]:
    return sorted(
        findings,
        key=lambda f: (-SEVERITIES.index(f.severity), f.package.ecosystem, f.package.name, f.detail),
    )


def render_json(findings: list[Finding]) -> str:
    items = [
        {
            "type": f.kind,
            "severity": f.severity,
            "ecosystem": f.package.ecosystem,
            "package": f.package.name,
            "version": f.package.version,
            "detail": f.detail,
            "message": f.message,
            "source": f.package.source,
        }
        for f in _sorted(findings)
    ]
    return json.dumps({"count": len(items), "findings": items}, indent=2)


def render_table(findings: list[Finding]) -> str:
    if not findings:
        return "No findings."
    rows = [
        (f.severity, f.kind, f.package.name, f.package.version, f.detail, f.message)
        for f in _sorted(findings)
    ]
    widths = [max(len(h), *(len(r[i]) for r in rows)) for i, h in enumerate(HEADERS)]
    def line(cells):
        return "  ".join(c.ljust(w) for c, w in zip(cells, widths)).rstrip()
    out = [line(HEADERS), line(["-" * w for w in widths])]
    out += [line(r) for r in rows]
    out.append(f"\n{len(rows)} finding(s)")
    return "\n".join(out)
