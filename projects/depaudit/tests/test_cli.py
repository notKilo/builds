import json

from depaudit.cli import main


def make_repo(tmp_path):
    (tmp_path / "requirements.txt").write_text("requests==2.5.0\nflask==3.0.0\n")
    (tmp_path / "package-lock.json").write_text(json.dumps({
        "lockfileVersion": 2,
        "packages": {"": {}, "node_modules/lodash": {"version": "4.17.20", "license": "MIT"}},
    }))
    return tmp_path


def test_reports_findings_and_exit_code(tmp_path, capsys):
    code = main([str(make_repo(tmp_path)), "--format", "json"])
    data = json.loads(capsys.readouterr().out)
    assert code == 1
    ids = {f["detail"] for f in data["findings"]}
    assert {"SAMPLE-0001", "SAMPLE-0002"} <= ids


def test_clean_repo_exits_zero(tmp_path, capsys):
    (tmp_path / "requirements.txt").write_text("requests==2.31.0\n")
    assert main([str(tmp_path)]) == 0
    assert "No findings" in capsys.readouterr().out


def test_min_severity_filters(tmp_path, capsys):
    main([str(make_repo(tmp_path)), "--format", "json", "--min-severity", "high"])
    ids = {f["detail"] for f in json.loads(capsys.readouterr().out)["findings"]}
    assert ids == {"SAMPLE-0002"}


def test_config_file_deny_license(tmp_path, capsys):
    repo = make_repo(tmp_path)
    (repo / ".depaudit.toml").write_text('[licenses]\ndeny = ["MIT"]\n[severity]\nminimum = "critical"\n')
    main([str(repo), "--format", "json"])
    kinds = {f["type"] for f in json.loads(capsys.readouterr().out)["findings"]}
    assert "license" in kinds


def test_bad_path(capsys):
    assert main(["/definitely/not/here"]) == 2
