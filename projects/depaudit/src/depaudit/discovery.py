from __future__ import annotations

import os
from pathlib import Path

from .models import Package
from .parsers import PARSERS

IGNORED_DIRS = {".git", "node_modules", "target", "venv", ".venv", "__pycache__", "dist", "build"}


def find_manifests(root: Path) -> list[Path]:
    found = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if d not in IGNORED_DIRS)
        found.extend(Path(dirpath) / f for f in sorted(filenames) if f in PARSERS)
    return found


def collect_packages(root: Path) -> list[Package]:
    """All distinct packages under root, in discovery order."""
    seen: set[tuple[str, str, str]] = set()
    packages = []
    for manifest in find_manifests(root):
        for pkg in PARSERS[manifest.name](manifest):
            key = (pkg.ecosystem, pkg.name, pkg.version)
            if key not in seen:
                seen.add(key)
                packages.append(pkg)
    return packages
