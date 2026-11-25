"""Parse 5-field cron expressions into sets of allowed values."""
from __future__ import annotations

from dataclasses import dataclass

MONTHS = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"]
DAYS = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"]

ALIASES = {
    "@yearly": "0 0 1 1 *",
    "@annually": "0 0 1 1 *",
    "@monthly": "0 0 1 * *",
    "@weekly": "0 0 * * 0",
    "@daily": "0 0 * * *",
    "@midnight": "0 0 * * *",
    "@hourly": "0 * * * *",
}


class CronError(ValueError):
    """Raised for any invalid cron expression."""


@dataclass(frozen=True)
class FieldSpec:
    name: str
    lo: int
    hi: int
    names: tuple[str, ...] = ()
    names_offset: int = 0


SPECS = [
    FieldSpec("minute", 0, 59),
    FieldSpec("hour", 0, 23),
    FieldSpec("day of month", 1, 31),
    FieldSpec("month", 1, 12, tuple(MONTHS), 1),
    FieldSpec("day of week", 0, 7, tuple(DAYS), 0),
]


@dataclass(frozen=True)
class Field:
    spec: FieldSpec
    raw: str
    values: frozenset[int]
    wildcard: bool  # starts with * (matters for the DOM/DOW rule)

    @property
    def full(self) -> bool:
        return len(self.values) == self.spec.hi - self.spec.lo + 1


@dataclass(frozen=True)
class CronExpr:
    source: str
    minute: Field
    hour: Field
    dom: Field
    month: Field
    dow: Field

    def day_matches(self, day: int, weekday_sun0: int) -> bool:
        """Standard cron rule: if both DOM and DOW are restricted, either may match."""
        dom_ok = day in self.dom.values
        dow_ok = weekday_sun0 in self.dow.values
        if self.dom.wildcard or self.dow.wildcard:
            return dom_ok and dow_ok
        return dom_ok or dow_ok


def _number(token: str, spec: FieldSpec) -> int:
    t = token.lower()
    if t.isdigit():
        n = int(t)
    elif t in spec.names:
        n = spec.names.index(t) + spec.names_offset
    else:
        raise CronError(f"invalid value {token!r} in {spec.name} field")
    if not spec.lo <= n <= spec.hi:
        raise CronError(f"{spec.name} value {n} out of range {spec.lo}-{spec.hi}")
    return n


def _parse_part(part: str, spec: FieldSpec) -> set[int]:
    if not part:
        raise CronError(f"empty list item in {spec.name} field")
    base, slash, step_s = part.partition("/")
    step = 1
    if slash:
        if not step_s.isdigit() or int(step_s) == 0:
            raise CronError(f"invalid step {step_s!r} in {spec.name} field")
        step = int(step_s)
    if base == "*":
        lo, hi = spec.lo, spec.hi
        if spec.name == "day of week":
            hi = 6
    elif "-" in base:
        a, _, b = base.partition("-")
        lo, hi = _number(a, spec), _number(b, spec)
        if lo > hi:
            raise CronError(f"descending range {base!r} in {spec.name} field")
    else:
        lo = _number(base, spec)
        hi = spec.hi if slash else lo
        if slash and spec.name == "day of week":
            hi = 6
    return set(range(lo, hi + 1, step))


def _parse_field(raw: str, spec: FieldSpec) -> Field:
    values: set[int] = set()
    for part in raw.split(","):
        values |= _parse_part(part, spec)
    if spec.name == "day of week" and 7 in values:  # 7 is an alias for Sunday
        values.discard(7)
        values.add(0)
    return Field(spec, raw, frozenset(values), raw.startswith("*"))


def parse(expr: str) -> CronExpr:
    text = expr.strip()
    if text.startswith("@"):
        key = text.lower()
        if key not in ALIASES:
            raise CronError(f"unknown alias {text!r}")
        text = ALIASES[key]
    parts = text.split()
    if len(parts) != 5:
        raise CronError(f"expected 5 fields, got {len(parts)}")
    fields = [_parse_field(p, s) for p, s in zip(parts, SPECS)]
    return CronExpr(expr.strip(), *fields)
