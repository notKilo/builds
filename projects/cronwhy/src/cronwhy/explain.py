"""Describe a CronExpr in plain English."""
from __future__ import annotations

from .parser import DAYS, MONTHS, CronExpr, Field

DAY_NAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
MONTH_NAMES = ["January", "February", "March", "April", "May", "June", "July",
               "August", "September", "October", "November", "December"]


def _ordinal(n: int) -> str:
    suffix = "th" if 10 <= n % 100 <= 20 else {1: "st", 2: "nd", 3: "rd"}.get(n % 10, "th")
    return f"{n}{suffix}"


def _join(items: list[str]) -> str:
    if len(items) <= 1:
        return "".join(items)
    return ", ".join(items[:-1]) + " and " + items[-1]


def _runs(values: list[int]) -> list[tuple[int, int]]:
    """Collapse sorted ints into (start, end) consecutive runs."""
    out: list[tuple[int, int]] = []
    for v in values:
        if out and out[-1][1] == v - 1:
            out[-1] = (out[-1][0], v)
        else:
            out.append((v, v))
    return out


def _describe(values: list[int], label) -> str:
    parts = []
    for a, b in _runs(values):
        if a == b:
            parts.append(label(a))
        elif b == a + 1:
            parts += [label(a), label(b)]
        else:
            parts.append(f"{label(a)} through {label(b)}")
    return _join(parts)


def _step(f: Field) -> int | None:
    """If the values form an arithmetic progression from the field start, return the step."""
    vals = sorted(f.values)
    if len(vals) < 3:
        return None
    d = vals[1] - vals[0]
    if d > 1 and all(b - a == d for a, b in zip(vals, vals[1:])) and vals[0] == f.spec.lo:
        return d
    return None


def _time_part(expr: CronExpr) -> str:
    mins, hrs = sorted(expr.minute.values), sorted(expr.hour.values)
    if expr.minute.full and expr.hour.full:
        return "Every minute"
    if len(mins) == 1 and len(hrs) == 1:
        return f"At {hrs[0]:02d}:{mins[0]:02d}"
    if expr.hour.full:
        step = _step(expr.minute)
        if step:
            return f"Every {step} minutes"
        if len(mins) == 1:
            return f"At minute {mins[0]} of every hour" if mins[0] else "Every hour, on the hour"
        return f"At minutes {_describe(mins, str)} of every hour"
    if len(mins) == 1:
        step = _step(expr.hour)
        if step:
            return f"At minute {mins[0]} every {step} hours"
        return "At " + _join([f"{h:02d}:{mins[0]:02d}" for h in hrs])
    hour_txt = _describe(hrs, lambda h: f"{h:02d}:00")
    return f"Every minute{'' if expr.minute.full else ' matching ' + _describe(mins, str)} during {hour_txt}"


def _day_part(expr: CronExpr) -> str:
    dom, dow = expr.dom, expr.dow
    dom_txt = None if dom.full else "on the " + _describe(sorted(dom.values), _ordinal)
    dow_txt = None if dow.full else "on " + _describe(sorted(dow.values), lambda d: DAY_NAMES[d])
    if dom_txt and dow_txt:
        if dom.wildcard or dow.wildcard:
            return f"{dom_txt} when it is {dow_txt[3:]}"
        return f"{dom_txt} or {dow_txt}"
    return dom_txt or dow_txt or ""


def explain(expr: CronExpr) -> str:
    pieces = [_time_part(expr)]
    day = _day_part(expr)
    if day:
        pieces.append(day)
    elif not expr.month.full:
        pieces.append("every day")
    if not expr.month.full:
        pieces.append("in " + _describe(sorted(expr.month.values), lambda m: MONTH_NAMES[m - 1]))
    return ", ".join(pieces)
