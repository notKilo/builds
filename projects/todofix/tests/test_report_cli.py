import json
from datetime import datetime, timezone

from conftest import commit
from todofix.cli import main
from todofix.report import bucket, collect, group_by

NOW = datetime(2024, 1, 1, tzinfo=timezone.utc)


def make(repo):
    commit(repo, {"a.py": "# TODO old thing\nx = 1\n"}, "2022-01-01T00:00:00", author="Ann")
    commit(repo, {"b.go": "// FIXME: recent | pipe\n"}, "2023-12-20T00:00:00", author="Bob")


def test_collect_sorted_oldest_first(repo):
    make(repo)
    items = collect(repo, now=NOW)
    assert [i.finding.path for i in items] == ["a.py", "b.go"]
    assert items[0].author == "Ann" and items[0].age_days == 730
    assert items[1].age_days == 12


def test_buckets_and_grouping(repo):
    make(repo)
    assert bucket(5) == "<30d" and bucket(400) == ">1y"
    g = group_by(collect(repo, now=NOW), "author")
    assert set(g) == {"Ann", "Bob"}


def test_cli_json(repo, capsys):
    make(repo)
    assert main([str(repo), "-f", "json"]) == 0
    data = json.loads(capsys.readouterr().out)
    assert data["count"] == 2


def test_cli_max_age_fails(repo, capsys):
    make(repo)
    assert main([str(repo), "--max-age", "365"]) == 1
    assert "older than 365" in capsys.readouterr().err


def test_cli_max_count_and_tag_filter(repo):
    make(repo)
    assert main([str(repo), "--max-count", "1"]) == 1
    assert main([str(repo), "--max-count", "1", "-t", "fixme"]) == 0


def test_cli_markdown_escapes_pipes(repo, capsys):
    make(repo)
    main([str(repo), "-f", "markdown"])
    assert "recent \\| pipe" in capsys.readouterr().out


def test_cli_not_a_repo(tmp_path):
    assert main([str(tmp_path)]) == 2
