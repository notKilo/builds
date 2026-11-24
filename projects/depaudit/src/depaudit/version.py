"""Version parsing and comparison.

Supports the common subset shared by PEP 440, semver and Cargo versions:
dotted numeric releases with an optional pre-release suffix. Pre-releases
sort before the matching final release.
"""
from __future__ import annotations

import re
from functools import total_ordering

_PATTERN = re.compile(
    r"^v?(?P<release>\d+(?:\.\d+)*)"
    r"(?:[-.]?(?P<pre>(?:a|b|rc|alpha|beta|c|pre|dev)\.?\d*))?"
    r"(?:\+[0-9A-Za-z.-]+)?$",
    re.IGNORECASE,
)

# pre-release kinds ordered lowest to highest; a final release outranks all
_PRE_ORDER = {"dev": 0, "alpha": 1, "a": 1, "beta": 2, "b": 2, "pre": 3, "c": 3, "rc": 3}


class InvalidVersion(ValueError):
    pass


@total_ordering
class Version:
    def __init__(self, text: str):
        match = _PATTERN.match(text.strip())
        if not match:
            raise InvalidVersion(f"unsupported version: {text!r}")
        self.text = text.strip()
        release = [int(p) for p in match["release"].split(".")]
        while len(release) > 1 and release[-1] == 0:
            release.pop()  # 1.0 == 1.0.0
        self.release = tuple(release)
        self.pre = self._parse_pre(match["pre"])

    @staticmethod
    def _parse_pre(raw: str | None) -> tuple[int, int] | None:
        if not raw:
            return None
        m = re.match(r"([a-z]+)\.?(\d*)", raw.lower())
        return (_PRE_ORDER[m[1]], int(m[2] or 0))

    def _key(self) -> tuple:
        # final releases get a rank above every pre-release rank
        return (self.release, (1, 0, 0) if self.pre is None else (0, *self.pre))

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Version) and self._key() == other._key()

    def __lt__(self, other: "Version") -> bool:
        return self._key() < other._key()

    def __hash__(self) -> int:
        return hash(self._key())

    def __repr__(self) -> str:
        return f"Version({self.text!r})"
