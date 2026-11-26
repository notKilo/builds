"""Find tagged comments (TODO, FIXME, ...) in source text."""
from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import PurePath

TAGS = ("TODO", "FIXME", "HACK", "XXX", "BUG")

# Comment leaders per file extension. A tag only counts after one of these.
_HASH = ("#",)
_SLASH = ("//", "/*", "*")
LEADERS: dict[str, tuple[str, ...]] = {
    ".py": _HASH, ".rb": _HASH, ".sh": _HASH, ".bash": _HASH, ".yml": _HASH,
    ".yaml": _HASH, ".toml": _HASH, ".pl": _HASH, ".r": _HASH,
    ".js": _SLASH, ".ts": _SLASH, ".tsx": _SLASH, ".jsx": _SLASH,
    ".go": _SLASH, ".rs": _SLASH, ".java": _SLASH, ".c": _SLASH, ".h": _SLASH,
    ".cpp": _SLASH, ".hpp": _SLASH, ".cs": _SLASH, ".kt": _SLASH,
    ".swift": _SLASH, ".php": _SLASH + _HASH,
    ".sql": ("--", "/*", "*"), ".lua": ("--",), ".hs": ("--",),
    ".html": ("<!--",), ".md": ("<!--",), ".css": ("/*", "*"),
}

_TAG_RE = re.compile(
    r"\b(?P<tag>" + "|".join(TAGS) + r")\b"
    r"(?:\((?P<owner>[^)]*)\))?"      # TODO(alice)
    r"\s*(?:\[(?P<ticket>[A-Z][A-Z0-9]*-\d+|#\d+)\])?"  # [ABC-12] or [#12]
    r"\s*[:\-]?\s*(?P<text>.*)$"
)


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    tag: str
    text: str
    owner: str | None = None
    ticket: str | None = None


def leaders_for(path: str) -> tuple[str, ...] | None:
    return LEADERS.get(PurePath(path).suffix.lower())


def scan_line(line: str, leaders: tuple[str, ...]) -> tuple[str, str | None, str | None, str] | None:
    """Return (tag, owner, ticket, text) if the line holds a tagged comment."""
    best = None
    for leader in leaders:
        idx = line.find(leader)
        if idx != -1 and (best is None or idx < best):
            best = idx
    if best is None:
        return None
    comment = line[best:]
    m = _TAG_RE.search(comment)
    if not m:
        return None
    # Require the tag to follow the leader directly (only punctuation between),
    # so words like "todo" inside prose or strings further along don't match.
    prefix = comment[: m.start()]
    if re.search(r"[A-Za-z0-9]", prefix):
        return None
    text = m.group("text").strip().removesuffix("*/").removesuffix("-->").strip()
    owner = (m.group("owner") or "").strip() or None
    return m.group("tag"), owner, m.group("ticket"), text


def scan_text(path: str, content: str) -> list[Finding]:
    leaders = leaders_for(path)
    if leaders is None:
        return []
    out = []
    for n, line in enumerate(content.splitlines(), start=1):
        hit = scan_line(line, leaders)
        if hit:
            tag, owner, ticket, text = hit
            out.append(Finding(path, n, tag, text, owner, ticket))
    return out
