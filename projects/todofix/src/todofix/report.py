"""Combine findings with blame data and render reports."""
from __future__ import annotations

import json
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from .blame import Blame, blame_file
from .discovery import discover
from .scanner import Finding, scan_text

BUCKETS = [(30, "<30d"), (90, "30-90d"), (365, "90d-1y"), (10**9, ">1y")]


@dataclass(frozen=True)
class Item:
    finding: Finding
    author: str
    when: datetime
    age_days: int

    def as_dict(self) -> dict:
        f = self.finding
        return {"path": f.path, "line": f.line, "tag": f.tag, "text": f.text,
                "owner": f.owner, "ticket": f.ticket, "author": self.author,
                "date": self.when.date().isoformat(), "age_days": self.age_days}


def bucket(age_days: int) -> str:
    return next(label for limit, label in BUCKETS if age_days < limit)


def collect(root: Path, ignore: list[str] | None = None,
            now: datetime | None = None) -> list[Item]:
    """Scan the repo; items come back oldest first."""
    now = now or datetime.now(timezone.utc)
    items: list[Item] = []
    for rel in discover(root, ignore):
        text = (root / rel).read_text(errors="replace")
        findings = scan_text(rel, text)
        if not findings:
            continue
        blames = blame_file(root, rel, [f.line for f in findings])
        for f in findings:
            b: Blame | None = blames.get(f.line)
            when = b.when if b else now
            items.append(Item(f, b.author if b else "uncommitted", when,
                              max(0, (now - when).days)))
    items.sort(key=lambda i: (-i.age_days, i.finding.path, i.finding.line))
    return items


def group_by(items: list[Item], key: str) -> dict[str, list[Item]]:
    getters = {
        "author": lambda i: i.author,
        "tag": lambda i: i.finding.tag,
        "file": lambda i: i.finding.path,
        "age": lambda i: bucket(i.age_days),
    }
    groups: dict[str, list[Item]] = defaultdict(list)
    for i in items:
        groups[getters[key](i)].append(i)
    return dict(groups)


def render_text(items: list[Item], group: str | None = None) -> str:
    if not items:
        return "No TODO comments found."

    def line(i: Item) -> str:
        f = i.finding
        return f"{i.age_days:>5}d  {f.tag:<5} {f.path}:{f.line}  {f.text}  ({i.author})"

    if not group:
        return "\n".join(line(i) for i in items) + f"\n\n{len(items)} comment(s)"
    out = []
    for name, grp in group_by(items, group).items():
        out.append(f"== {name} ({len(grp)}) ==")
        out += [line(i) for i in grp]
        out.append("")
    return "\n".join(out).rstrip()


def render_json(items: list[Item]) -> str:
    return json.dumps({"count": len(items), "items": [i.as_dict() for i in items]}, indent=2)


def render_markdown(items: list[Item]) -> str:
    rows = ["| Age | Tag | Location | Text | Author |", "|---:|---|---|---|---|"]
    for i in items:
        f = i.finding
        rows.append(f"| {i.age_days}d | {f.tag} | `{f.path}:{f.line}` | "
                    f"{f.text.replace('|', chr(92) + '|')} | {i.author} |")
    return "\n".join(rows)
