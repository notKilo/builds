import json

import pytest

from depaudit.advisories import Advisory, AdvisoryIndex, load_advisories
from depaudit.models import Package


def adv(**kw):
    base = dict(id="X-1", ecosystem="PyPI", package="req", severity="high", summary="")
    return Advisory(**{**base, **kw})


def test_fixed_bound_is_exclusive():
    a = adv(ranges=({"introduced": "1.0", "fixed": "1.2.3"},))
    assert a.affects("1.0") and a.affects("1.2.2")
    assert not a.affects("1.2.3")
    assert not a.affects("0.9")


def test_open_ended_and_last_affected():
    assert adv(ranges=({"introduced": "2.0"},)).affects("99.0")
    a = adv(ranges=({"last_affected": "1.5"},))
    assert a.affects("1.5") and not a.affects("1.5.1")


def test_prerelease_of_fix_is_still_affected():
    assert adv(ranges=({"fixed": "2.0"},)).affects("2.0rc1")


def test_unparseable_version_is_not_a_match():
    assert not adv(ranges=({"fixed": "2.0"},)).affects("weird-build")


def test_index_matches_by_ecosystem_and_name():
    idx = AdvisoryIndex([adv(ranges=({"fixed": "2.0"},))])
    assert idx.matches(Package("PyPI", "req", "1.0"))
    assert not idx.matches(Package("npm", "req", "1.0"))
    assert not idx.matches(Package("PyPI", "req", "2.0"))


def test_load_normalizes_pypi_names_and_validates_severity(tmp_path):
    f = tmp_path / "a.json"
    f.write_text(json.dumps([{"id": "A", "ecosystem": "PyPI", "package": "Django", "severity": "HIGH"}]))
    (loaded,) = load_advisories(f)
    assert loaded.package == "django" and loaded.severity == "high"
    f.write_text(json.dumps([{"id": "B", "ecosystem": "npm", "package": "x", "severity": "bad"}]))
    with pytest.raises(ValueError):
        load_advisories(f)
