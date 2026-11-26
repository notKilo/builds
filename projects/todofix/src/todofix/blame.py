"""Attribute lines to commits using `git blame --porcelain`."""
from __future__ import annotations

import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


@dataclass(frozen=True)
class Blame:
    author: str
    when: datetime
    commit: str


def parse_porcelain(text: str) -> dict[int, Blame]:
    """Map final line number -> Blame from porcelain output."""
    commits: dict[str, dict[str, str]] = {}
    result: dict[int, Blame] = {}
    current: str | None = None
    final_line = 0
    for raw in text.splitlines():
        if raw.startswith("\t"):
            info = commits[current]
            result[final_line] = Blame(
                info.get("author", "unknown"),
                datetime.fromtimestamp(int(info["author-time"]), timezone.utc),
                current,
            )
            continue
        parts = raw.split(" ")
        if len(parts[0]) == 40 and len(parts) >= 3 and parts[2].isdigit():
            current, final_line = parts[0], int(parts[2])
            commits.setdefault(current, {})
        elif current and " " in raw:
            key, value = raw.split(" ", 1)
            commits[current].setdefault(key, value)
    return result


def blame_file(root: Path, rel: str, lines: list[int]) -> dict[int, Blame]:
    """Blame only the requested lines of a file; empty dict if untracked changes block it."""
    args = ["git", "-C", str(root), "blame", "--porcelain"]
    for n in lines:
        args += ["-L", f"{n},{n}"]
    proc = subprocess.run(args + ["--", rel], capture_output=True)
    if proc.returncode != 0:
        return {}
    return parse_porcelain(proc.stdout.decode(errors="replace"))
