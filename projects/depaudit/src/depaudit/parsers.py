"""Parsers that turn manifests and lockfiles into Package lists."""
from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path

from .models import Package

_REQ_LINE = re.compile(r"^([A-Za-z0-9][A-Za-z0-9._-]*)(?:\[[^\]]*\])?\s*==\s*([^\s;#]+)")


def _normalize_pypi(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def parse_requirements(path: Path) -> list[Package]:
    """Only exact pins (name==version) are auditable; other lines are skipped."""
    packages = []
    for line in path.read_text().splitlines():
        match = _REQ_LINE.match(line.strip())
        if match:
            packages.append(
                Package("PyPI", _normalize_pypi(match[1]), match[2], source=str(path))
            )
    return packages


def parse_package_lock(path: Path) -> list[Package]:
    """Handles lockfileVersion 2/3 ("packages") and falls back to v1 ("dependencies")."""
    data = json.loads(path.read_text())
    packages = []
    if "packages" in data:
        for key, entry in data["packages"].items():
            if not key or "version" not in entry:  # "" is the root project
                continue
            name = entry.get("name") or key.rsplit("node_modules/", 1)[-1]
            packages.append(
                Package("npm", name, entry["version"], str(path), entry.get("license"))
            )
    else:
        _walk_v1(data.get("dependencies", {}), path, packages)
    return packages


def _walk_v1(deps: dict, path: Path, out: list[Package]) -> None:
    for name, entry in deps.items():
        if "version" in entry:
            out.append(Package("npm", name, entry["version"], str(path)))
        _walk_v1(entry.get("dependencies", {}), path, out)


def parse_cargo_lock(path: Path) -> list[Package]:
    data = tomllib.loads(path.read_text())
    # path/workspace crates have no "source"; only registry/git crates are audited
    return [
        Package("crates.io", p["name"], p["version"], str(path))
        for p in data.get("package", [])
        if p.get("source")
    ]


PARSERS = {
    "requirements.txt": parse_requirements,
    "package-lock.json": parse_package_lock,
    "Cargo.lock": parse_cargo_lock,
}
