"""List the files worth scanning in a git repository."""
from __future__ import annotations

import fnmatch
import subprocess
from pathlib import Path


def git_files(root: Path) -> list[str]:
    """Tracked files under root, as paths relative to root."""
    out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", "--", "."],
        check=True, capture_output=True,
    ).stdout
    return [p for p in out.decode().split("\0") if p]


def is_binary(path: Path, probe: int = 4096) -> bool:
    try:
        return b"\0" in path.read_bytes()[:probe]
    except OSError:
        return True


def is_ignored(rel: str, patterns: list[str]) -> bool:
    return any(
        fnmatch.fnmatch(rel, p) or fnmatch.fnmatch(Path(rel).name, p)
        for p in patterns
    )


def discover(root: Path, ignore: list[str] | None = None) -> list[str]:
    ignore = ignore or []
    return [
        rel for rel in git_files(root)
        if not is_ignored(rel, ignore) and not is_binary(root / rel)
    ]
