from datetime import datetime
from zoneinfo import ZoneInfo

from cronwhy import parse
from cronwhy.schedule import next_runs

UTC = ZoneInfo("UTC")


def fmt(runs):
    return [r.strftime("%Y-%m-%d %H:%M") for r in runs]


def test_every_15_minutes():
    runs = next_runs(parse("*/15 * * * *"), datetime(2026, 1, 1, 10, 7, tzinfo=UTC), 3)
    assert fmt(runs) == ["2026-01-01 10:15", "2026-01-01 10:30", "2026-01-01 10:45"]


def test_strictly_after():
    runs = next_runs(parse("0 12 * * *"), datetime(2026, 1, 1, 12, 0, tzinfo=UTC), 1)
    assert fmt(runs) == ["2026-01-02 12:00"]


def test_weekday_filter():
    # 2026-01-01 is a Thursday; next Monday is Jan 5
    runs = next_runs(parse("30 9 * * mon"), datetime(2026, 1, 1, tzinfo=UTC), 2)
    assert fmt(runs) == ["2026-01-05 09:30", "2026-01-12 09:30"]


def test_dom_or_dow():
    runs = next_runs(parse("0 0 13 * fri"), datetime(2026, 1, 1, tzinfo=UTC), 3)
    assert fmt(runs) == ["2026-01-02 00:00", "2026-01-09 00:00", "2026-01-13 00:00"]


def test_leap_day():
    runs = next_runs(parse("0 0 29 2 *"), datetime(2026, 1, 1, tzinfo=UTC), 2)
    assert fmt(runs) == ["2028-02-29 00:00", "2032-02-29 00:00"]


def test_year_rollover():
    runs = next_runs(parse("@yearly"), datetime(2026, 6, 1, tzinfo=UTC), 2)
    assert fmt(runs) == ["2027-01-01 00:00", "2028-01-01 00:00"]


def test_timezone_conversion():
    ny = ZoneInfo("America/New_York")
    runs = next_runs(parse("0 9 * * *"), datetime(2026, 1, 1, tzinfo=UTC), 1, "America/New_York")
    assert runs[0] == datetime(2026, 1, 1, 9, 0, tzinfo=ny)
    assert runs[0].utcoffset().total_seconds() == -5 * 3600


def test_dst_gap_shifts_forward():
    # US spring forward 2026-03-08: 02:00 -> 03:00; 02:30 does not exist
    runs = next_runs(parse("30 2 * * *"), datetime(2026, 3, 7, 3, tzinfo=UTC), 3, "America/New_York")
    assert [(r.month, r.day, r.hour, r.minute) for r in runs] == [
        (3, 7, 2, 30), (3, 8, 3, 30), (3, 9, 2, 30)]


def test_dst_overlap_runs_once():
    # US fall back 2026-11-01: 01:30 happens twice
    runs = next_runs(parse("30 1 * * *"), datetime(2026, 10, 31, 3, tzinfo=UTC), 3, "America/New_York")
    assert [(r.month, r.day) for r in runs] == [(10, 31), (11, 1), (11, 2)]
    assert runs[1].fold == 0 and runs[1].utcoffset().total_seconds() == -4 * 3600
    assert len({r.astimezone(UTC) for r in runs}) == 3
