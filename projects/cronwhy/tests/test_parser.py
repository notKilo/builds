import pytest

from cronwhy import CronError, parse


def vals(expr, field):
    return sorted(getattr(parse(expr), field).values)


def test_star_and_lists():
    assert vals("* * * * *", "minute") == list(range(60))
    assert vals("1,5,10 * * * *", "minute") == [1, 5, 10]


def test_ranges_and_steps():
    assert vals("*/15 * * * *", "minute") == [0, 15, 30, 45]
    assert vals("10-20/5 * * * *", "minute") == [10, 15, 20]
    assert vals("5/20 * * * *", "minute") == [5, 25, 45]


def test_names_case_insensitive():
    assert vals("0 0 * JAN,mar *", "month") == [1, 3]
    assert vals("0 0 * * mon-fri", "dow") == [1, 2, 3, 4, 5]


def test_sunday_seven_alias():
    assert vals("0 0 * * 7", "dow") == [0]
    assert vals("0 0 * * 5-7", "dow") == [0, 5, 6]


def test_aliases():
    assert vals("@hourly", "minute") == [0]
    assert vals("@daily", "hour") == [0]
    assert vals("@weekly", "dow") == [0]
    assert parse("@yearly").month.values == {1}


def test_day_rule_or_when_both_restricted():
    e = parse("0 0 13 * 5")  # the 13th OR any Friday
    assert e.day_matches(13, 2)
    assert e.day_matches(7, 5)
    assert not e.day_matches(7, 2)


def test_day_rule_and_when_star():
    e = parse("0 0 * * 5")
    assert e.day_matches(7, 5)
    assert not e.day_matches(7, 2)


@pytest.mark.parametrize(
    "bad",
    ["", "* * * *", "* * * * * *", "60 * * * *", "* 24 * * *", "* * 0 * *",
     "* * * 13 *", "*/0 * * * *", "5-1 * * * *", "a * * * *", "1,,2 * * * *", "@never"],
)
def test_errors(bad):
    with pytest.raises(CronError):
        parse(bad)
