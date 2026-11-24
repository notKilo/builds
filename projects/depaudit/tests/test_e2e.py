"""End-to-end runs of the CLI against a fixture repo with all three ecosystems."""
import json
from pathlib import Path

from depaudit.cli import main

FIXTURE = str(Path(__file__).parent / "fixtures" / "mixed")


def run(capsys, *extra):
    code = main([FIXTURE, "--format", "json", "--license-map", f"{FIXTURE}/licenses.json", *extra])
    return code, json.loads(capsys.readouterr().out)["findings"]


def test_finds_vulnerabilities_in_every_ecosystem(capsys):
    code, findings = run(capsys)
    assert code == 1
    assert {f["detail"] for f in findings} == {"SAMPLE-0001", "SAMPLE-0002", "SAMPLE-0003"}


def test_vendored_node_modules_are_ignored(capsys):
    _, findings = run(capsys)
    assert all("node_modules/x" not in f["source"] for f in findings)


def test_deny_list_flags_gpl_dependency(capsys):
    _, findings = run(capsys, "--deny", "GPL-3.0-only")
    licenses = [f for f in findings if f["type"] == "license"]
    assert [f["package"] for f in licenses] == ["leftpad"]


def test_unpinned_requirements_are_skipped(capsys):
    _, findings = run(capsys, "--fail-on-unknown")
    assert "numpy" not in {f["package"] for f in findings}


def test_no_bundled_advisories_leaves_only_license_findings(capsys):
    code, findings = run(capsys, "--no-bundled", "--deny", "GPL-3.0-only")
    assert code == 1
    assert {f["type"] for f in findings} == {"license"}


def test_table_output_lists_findings(capsys):
    assert main([FIXTURE, "--min-severity", "critical"]) == 1
    out = capsys.readouterr().out
    assert "SAMPLE-0003" in out and "SAMPLE-0001" not in out
