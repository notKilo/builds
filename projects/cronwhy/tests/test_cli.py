import json
from datetime import datetime
from zoneinfo import ZoneInfo

from cronwhy.cli import main

NOW = datetime(2026, 1, 1, 10, 7, tzinfo=ZoneInfo("UTC"))


def test_explain(capsys):
    assert main(["explain", "*/5 * * * *"]) == 0
    assert capsys.readouterr().out.strip() == "Every 5 minutes"


def test_next_json(capsys):
    assert main(["next", "0 12 * * *", "-n", "2", "--json"], now=NOW) == 0
    out = json.loads(capsys.readouterr().out)
    assert out == ["2026-01-01T12:00:00+00:00", "2026-01-02T12:00:00+00:00"]


def test_next_tz_text(capsys):
    main(["next", "0 9 * * *", "-n", "1", "--tz", "Asia/Tokyo"], now=NOW)
    assert "09:00 JST (+0900)" in capsys.readouterr().out


def test_invalid(capsys):
    assert main(["check", "61 * * * *"]) == 1
    assert "out of range" in capsys.readouterr().err


def test_bad_tz(capsys):
    assert main(["next", "* * * * *", "--tz", "Mars/Base"], now=NOW) == 1
