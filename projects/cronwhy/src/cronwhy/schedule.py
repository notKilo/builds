"""Compute upcoming run times for a CronExpr in a timezone.

Matching happens on wall-clock (local) time. DST policy:
  * a local time skipped by a spring-forward gap runs once, shifted forward
    by the size of the gap (02:30 becomes 03:30 when 02:00 jumps to 03:00);
  * a local time repeated by a fall-back overlap runs once, on its first pass.
"""
from __future__ import annotations

import calendar
from datetime import date, datetime, timedelta, timezone
from typing import Iterator
from zoneinfo import ZoneInfo

from .parser import CronExpr


def _days_in_month(year: int, month: int) -> int:
    return calendar.monthrange(year, month)[1]


def _matching_days(expr: CronExpr, year: int, month: int) -> list[int]:
    out = []
    for day in range(1, _days_in_month(year, month) + 1):
        weekday_sun0 = (date(year, month, day).weekday() + 1) % 7
        if expr.day_matches(day, weekday_sun0):
            out.append(day)
    return out


def _naive_matches(expr: CronExpr, start: datetime) -> Iterator[datetime]:
    """Yield naive local datetimes >= start matching expr, in order."""
    hours = sorted(expr.hour.values)
    minutes = sorted(expr.minute.values)
    year, month = start.year, start.month
    for _ in range(12 * 30):  # search horizon: 30 years covers Feb 29 on odd weekdays
        if month in expr.month.values:
            for day in _matching_days(expr, year, month):
                for h in hours:
                    for m in minutes:
                        cand = datetime(year, month, day, h, m)
                        if cand >= start:
                            yield cand
        month += 1
        if month > 12:
            month, year = 1, year + 1
    return


def _resolve(naive: datetime, tz: ZoneInfo) -> datetime:
    """Attach tz to a naive wall time, applying the DST policy above."""
    local = naive.replace(tzinfo=tz, fold=0)
    roundtrip = local.astimezone(timezone.utc).astimezone(tz)
    if roundtrip.replace(tzinfo=None) == naive:
        return local
    # Nonexistent time: interpret with the pre-transition offset, which lands past the gap.
    utc = local.astimezone(timezone.utc)
    fixed = utc.astimezone(tz)
    return fixed.replace(second=0, microsecond=0)


def next_runs(expr: CronExpr, after: datetime, count: int, tz: str = "UTC") -> list[datetime]:
    """The next `count` run times strictly after `after` (aware or naive-in-tz)."""
    zone = ZoneInfo(tz)
    if after.tzinfo is None:
        after = after.replace(tzinfo=zone)
    after = after.astimezone(zone)
    start = after.replace(tzinfo=None, second=0, microsecond=0) - timedelta(days=1)
    runs: list[datetime] = []
    seen: set[datetime] = set()
    for naive in _naive_matches(expr, start):
        when = _resolve(naive, zone)
        key = when.astimezone(timezone.utc)
        if key <= after.astimezone(timezone.utc) or key in seen:
            continue
        seen.add(key)
        runs.append(when)
        if len(runs) >= count:
            break
    return runs
