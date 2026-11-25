import json

import pytest

from diffsheet.cli import main


@pytest.fixture
def files(tmp_path):
    a = tmp_path / "a.csv"
    b = tmp_path / "b.csv"
    a.write_text("id,v\n1,10\n2,20\n")
    b.write_text("id,v\n2,20\n1,10\n")
    return a, b, tmp_path


def test_identical_exit_zero(files, capsys):
    a, b, _ = files
    assert main([str(a), str(b), "-k", "id"]) == 0
    assert "0 added" in capsys.readouterr().out


def test_differences_exit_one(files, capsys):
    a, b, tmp = files
    b.write_text("id,v\n1,11\n")
    assert main([str(a), str(b), "-k", "id", "-f", "json"]) == 1
    data = json.loads(capsys.readouterr().out)
    assert data["summary"]["changed"] == 1 and data["summary"]["removed"] == 1


def test_csv_vs_json(files, capsys):
    a, _, tmp = files
    j = tmp / "b.json"
    j.write_text('[{"id": 1, "v": 10}, {"id": 2, "v": 20}]')
    assert main([str(a), str(j), "-k", "id"]) == 0


def test_summary_flag(files, capsys):
    a, b, _ = files
    main([str(a), str(b), "-k", "id", "-s"])
    assert capsys.readouterr().out.strip() == "0 added, 0 removed, 0 changed, 2 unchanged"


def test_error_exit_two(files, capsys):
    a, b, _ = files
    assert main([str(a), str(b), "-k", "nope"]) == 2
    assert "missing" in capsys.readouterr().err


def test_missing_file(capsys):
    assert main(["/no/such.csv", "/no/such2.csv", "-k", "id"]) == 2


def test_stdin_requires_format(files, capsys):
    a, _, _ = files
    assert main(["-", str(a), "-k", "id"]) == 2


def test_tolerance_option(files):
    a, b, _ = files
    b.write_text("id,v\n1,10.4\n2,20\n")
    assert main([str(a), str(b), "-k", "id"]) == 1
    assert main([str(a), str(b), "-k", "id", "-t", "0.5"]) == 0
