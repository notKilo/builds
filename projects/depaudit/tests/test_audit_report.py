import json

from depaudit.advisories import Advisory, AdvisoryIndex
from depaudit.audit import audit_packages, filter_severity
from depaudit.licenses import Policy
from depaudit.models import Package
from depaudit.report import render_json, render_table

ADV = Advisory("OSV-1", "PyPI", "foo", "high", "bad bug", ({"introduced": "1.0", "fixed": "1.2"},))
LOW = Advisory("OSV-2", "PyPI", "bar", "low", "meh", ({"fixed": "9"},))


def run():
    pkgs = [
        Package("PyPI", "foo", "1.1"),
        Package("PyPI", "bar", "1.0", license="GPL-3.0"),
        Package("PyPI", "ok", "1.0", license="MIT"),
    ]
    return audit_packages(pkgs, AdvisoryIndex([ADV, LOW]), Policy(deny={"GPL-3.0-only"}))


def test_audit_finds_vulns_and_license_violations():
    kinds = sorted((f.kind, f.package.name) for f in run())
    assert kinds == [("license", "bar"), ("vulnerability", "bar"), ("vulnerability", "foo")]


def test_fixed_version_in_message():
    msg = next(f.message for f in run() if f.detail == "OSV-1")
    assert "fixed in 1.2" in msg


def test_filter_severity():
    names = {f.detail for f in filter_severity(run(), "high")}
    assert "OSV-2" not in names and "OSV-1" in names


def test_json_output():
    data = json.loads(render_json(run()))
    assert data["count"] == 3
    assert data["findings"][-1]["severity"] == "low"


def test_table_output():
    out = render_table(run())
    assert out.splitlines()[0].startswith("SEVERITY")
    assert "3 finding(s)" in out
    assert render_table([]) == "No findings."
