from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Package:
    """A resolved dependency found in a manifest or lockfile."""

    ecosystem: str  # "PyPI", "npm" or "crates.io" (OSV ecosystem names)
    name: str
    version: str
    source: str = ""  # manifest path the package came from
    license: str | None = None  # license declared in the manifest, if any
