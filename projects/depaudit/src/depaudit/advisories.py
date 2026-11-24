"""Advisory database loading and version matching.

Advisories use a trimmed OSV-like shape::

    {"id": "...", "ecosystem": "PyPI", "package": "name", "severity": "high",
     "summary": "...", "ranges": [{"introduced": "1.0", "fixed": "1.2.3"}]}

A range matches versions >= introduced (default: everything) and < fixed
(default: no upper bound). "last_affected" gives an inclusive upper bound.
"""
from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path

from .models import Package
from .version import InvalidVersion, Version

SEVERITIES = ["low", "medium", "high", "critical"]


@dataclass(frozen=True)
class Advisory:
    id: str
    ecosystem: str
    package: str
    severity: str
    summary: str
    ranges: tuple[dict, ...] = field(default_factory=tuple)

    def affects(self, version: str) -> bool:
        try:
            v = Version(version)
            return any(_in_range(v, r) for r in self.ranges)
        except InvalidVersion:
            return False

    @property
    def fixed_versions(self) -> list[str]:
        return [r["fixed"] for r in self.ranges if "fixed" in r]


def _in_range(v: Version, r: dict) -> bool:
    if "introduced" in r and v < Version(r["introduced"]):
        return False
    if "fixed" in r and v >= Version(r["fixed"]):
        return False
    if "last_affected" in r and v > Version(r["last_affected"]):
        return False
    return True


def load_advisories(path: Path) -> list[Advisory]:
    items = json.loads(path.read_text())
    advisories = []
    for item in items:
        severity = item.get("severity", "medium").lower()
        if severity not in SEVERITIES:
            raise ValueError(f"{item.get('id')}: unknown severity {severity!r}")
        advisories.append(
            Advisory(
                id=item["id"],
                ecosystem=item["ecosystem"],
                package=item["package"].lower() if item["ecosystem"] == "PyPI" else item["package"],
                severity=severity,
                summary=item.get("summary", ""),
                ranges=tuple(item.get("ranges", [])),
            )
        )
    return advisories


class AdvisoryIndex:
    def __init__(self, advisories: list[Advisory]):
        self._by_package: dict[tuple[str, str], list[Advisory]] = {}
        for adv in advisories:
            self._by_package.setdefault((adv.ecosystem, adv.package), []).append(adv)

    def matches(self, pkg: Package) -> list[Advisory]:
        candidates = self._by_package.get((pkg.ecosystem, pkg.name), [])
        return [a for a in candidates if a.affects(pkg.version)]
