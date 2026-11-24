import json

import pytest

from depaudit.licenses import (
    UNKNOWN, Policy, license_ids, normalize_id, parse_expression, resolve_license,
)
from depaudit.models import Package


def test_normalize_aliases_and_case():
    assert normalize_id("MIT License") == "MIT"
    assert normalize_id("apache 2.0") == "Apache-2.0"
    assert normalize_id("mit") == "MIT"
    assert normalize_id("GPL-3.0") == "GPL-3.0-only"
    assert normalize_id("GPL-2.0+") == "GPL-2.0-or-later"
    assert normalize_id("Weird-1.0") == "Weird-1.0"


def test_parse_expression_precedence():
    assert parse_expression("MIT OR Apache-2.0 AND ISC") == ("or", ["MIT", ("and", ["Apache-2.0", "ISC"])])
    assert parse_expression("(MIT OR ISC) AND Zlib") == ("and", [("or", ["MIT", "ISC"]), "Zlib"])


@pytest.mark.parametrize("bad", ["MIT OR", "(MIT", "MIT AND AND ISC", ") MIT"])
def test_parse_errors(bad):
    with pytest.raises(ValueError):
        parse_expression(bad)


def test_license_ids():
    assert license_ids("MIT OR (Apache-2.0 AND ISC)") == ["MIT", "Apache-2.0", "ISC"]


def test_deny_policy():
    p = Policy(deny={"GPL-3.0-only"})
    assert p.permits("MIT")
    assert not p.permits("GPL-3.0")
    assert p.permits("GPL-3.0 OR MIT")
    assert not p.permits("GPL-3.0 AND MIT")


def test_allow_policy_and_unknown():
    p = Policy(allow={"MIT", "ISC"}, allow_unknown=False)
    assert p.permits("MIT OR GPL-3.0")
    assert not p.permits("Apache-2.0")
    assert not p.permits(UNKNOWN)
    assert Policy(allow={"MIT"}).permits(UNKNOWN)


def test_resolve_license_fallback(tmp_path):
    m = {"PyPI:foo": "MIT"}
    assert resolve_license(Package("PyPI", "foo", "1"), m) == "MIT"
    assert resolve_license(Package("PyPI", "bar", "1"), m) == UNKNOWN
    assert resolve_license(Package("PyPI", "bar", "1", license="ISC"), m) == "ISC"
