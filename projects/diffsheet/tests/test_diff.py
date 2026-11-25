import pytest

from diffsheet.diff import DiffError, DiffOptions, diff_tables
from diffsheet.loaders import Table


def table(columns, *rows):
    return Table(columns, [dict(zip(columns, r)) for r in rows])


OLD = table(["id", "name", "qty"], ("1", "apple", "5"), ("2", "pear", "7"), ("3", "fig", "1"))


def run(old, new, **kw):
    kw.setdefault("keys", ["id"])
    return diff_tables(old, new, DiffOptions(**kw))


def test_identical_tables():
    r = run(OLD, OLD)
    assert not r.has_differences
    assert r.unchanged == 3


def test_reordered_rows_are_not_changes():
    new = table(["id", "name", "qty"], ("3", "fig", "1"), ("1", "apple", "5"), ("2", "pear", "7"))
    assert not run(OLD, new).has_differences


def test_added_removed_changed():
    new = table(["id", "name", "qty"], ("1", "apple", "6"), ("2", "pear", "7"), ("4", "kiwi", "2"))
    r = run(OLD, new)
    assert [row["id"] for row in r.added] == ["4"]
    assert [row["id"] for row in r.removed] == ["3"]
    assert r.changed[0].key == ("1",)
    assert (r.changed[0].changes[0].old, r.changed[0].changes[0].new) == ("5", "6")
    assert r.unchanged == 1


def test_composite_key():
    old = table(["a", "b", "v"], ("1", "x", "1"), ("1", "y", "2"))
    new = table(["a", "b", "v"], ("1", "x", "1"), ("1", "y", "3"))
    r = run(old, new, keys=["a", "b"])
    assert r.changed[0].key == ("1", "y")


def test_schema_changes():
    new = table(["id", "name", "price"], ("1", "apple", "9"), ("2", "pear", "9"), ("3", "fig", "9"))
    r = run(OLD, new)
    assert r.columns_added == ["price"]
    assert r.columns_removed == ["qty"]
    assert r.has_differences


def test_duplicate_key_rejected():
    dup = table(["id", "name", "qty"], ("1", "a", "1"), ("1", "b", "2"))
    with pytest.raises(DiffError, match="duplicate"):
        run(dup, OLD)


def test_missing_key_column():
    with pytest.raises(DiffError, match="missing"):
        run(OLD, OLD, keys=["nope"])


def test_numeric_strings_compare_as_numbers():
    new = table(["id", "name", "qty"], ("1", "apple", "5.0"), ("2", "pear", "7"), ("3", "fig", "1"))
    assert not run(OLD, new).has_differences


def test_json_int_equals_csv_string():
    old = table(["id", "v"], (1, 2))
    new = table(["id", "v"], (1, "2"))
    assert not run(old, new).has_differences


def test_tolerance():
    old = table(["id", "v"], ("1", "1.000"))
    new = table(["id", "v"], ("1", "1.004"))
    assert run(old, new).has_differences
    assert not run(old, new, tolerance=0.01).has_differences


def test_trim_and_case():
    old = table(["id", "v"], ("1", "Hello "))
    new = table(["id", "v"], ("1", "hello"))
    assert run(old, new).has_differences
    assert run(old, new, trim=True).has_differences
    assert not run(old, new, trim=True, ignore_case=True).has_differences


def test_ignore_column():
    new = table(["id", "name", "qty"], ("1", "apple", "9"), ("2", "pear", "7"), ("3", "fig", "1"))
    assert not run(OLD, new, ignore={"qty"}).has_differences


def test_none_vs_empty_string_differs():
    old = table(["id", "v"], ("1", None))
    new = table(["id", "v"], ("1", ""))
    assert run(old, new).has_differences
