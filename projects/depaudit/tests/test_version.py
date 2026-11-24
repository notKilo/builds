import pytest

from depaudit.version import InvalidVersion, Version


def v(text):
    return Version(text)


def test_numeric_ordering_is_not_lexical():
    assert v("1.10.0") > v("1.9.9")
    assert v("2.0") > v("1.99.99")


def test_trailing_zeros_are_equal():
    assert v("1.0") == v("1.0.0")
    assert hash(v("1.0")) == hash(v("1.0.0"))


def test_prerelease_sorts_before_release():
    assert v("1.0.0rc1") < v("1.0.0")
    assert v("1.0.0-beta.2") < v("1.0.0-rc.1")
    assert v("1.0.0a1") < v("1.0.0b1")


def test_prerelease_numbers_compare():
    assert v("1.0rc2") > v("1.0rc1")


def test_leading_v_and_build_metadata_ignored():
    assert v("v1.2.3") == v("1.2.3")
    assert v("1.2.3+build5") == v("1.2.3")


def test_invalid_version_raises():
    with pytest.raises(InvalidVersion):
        Version("not-a-version")
