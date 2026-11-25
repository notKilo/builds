"""Key-based diff of two tables."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .loaders import Table

Key = tuple[Any, ...]


class DiffError(ValueError):
    pass


@dataclass
class DiffOptions:
    keys: list[str]
    ignore: set[str] = field(default_factory=set)
    tolerance: float = 0.0
    trim: bool = False
    ignore_case: bool = False


@dataclass
class Change:
    column: str
    old: Any
    new: Any


@dataclass
class RowChange:
    key: Key
    changes: list[Change]


@dataclass
class DiffResult:
    keys: list[str]
    added: list[dict[str, Any]] = field(default_factory=list)
    removed: list[dict[str, Any]] = field(default_factory=list)
    changed: list[RowChange] = field(default_factory=list)
    columns_added: list[str] = field(default_factory=list)
    columns_removed: list[str] = field(default_factory=list)
    unchanged: int = 0

    @property
    def has_differences(self) -> bool:
        return bool(
            self.added or self.removed or self.changed
            or self.columns_added or self.columns_removed
        )


def diff_tables(old: Table, new: Table, options: DiffOptions) -> DiffResult:
    for table, label in ((old, "old"), (new, "new")):
        missing = [k for k in options.keys if k not in table.columns]
        if missing:
            raise DiffError(f"key column(s) {missing} missing from {label} table")

    old_rows = _index(old, options.keys, "old")
    new_rows = _index(new, options.keys, "new")

    skip = set(options.ignore) | set(options.keys)
    result = DiffResult(keys=list(options.keys))
    result.columns_added = [c for c in new.columns if c not in old.columns and c not in options.ignore]
    result.columns_removed = [c for c in old.columns if c not in new.columns and c not in options.ignore]
    shared = [c for c in old.columns if c in new.columns and c not in skip]

    for key, row in old_rows.items():
        if key not in new_rows:
            result.removed.append(row)
            continue
        changes = [
            Change(c, row[c], new_rows[key][c])
            for c in shared
            if not values_equal(row[c], new_rows[key][c], options)
        ]
        if changes:
            result.changed.append(RowChange(tuple(row[k] for k in options.keys), changes))
        else:
            result.unchanged += 1
    result.added = [row for key, row in new_rows.items() if key not in old_rows]
    return result


def _index(table: Table, keys: list[str], label: str) -> dict[Key, dict[str, Any]]:
    index: dict[Key, dict[str, Any]] = {}
    for row in table.rows:
        key = tuple(_norm_key(row[k]) for k in keys)
        if key in index:
            raise DiffError(f"duplicate key {key!r} in {label} table")
        index[key] = row
    return index


def values_equal(a: Any, b: Any, options: DiffOptions) -> bool:
    if a == b:
        return True
    if a is None or b is None:
        return False
    num_a, num_b = _as_number(a), _as_number(b)
    if num_a is not None and num_b is not None:
        return abs(num_a - num_b) <= options.tolerance
    if isinstance(a, str) and isinstance(b, str):
        if options.trim:
            a, b = a.strip(), b.strip()
        if options.ignore_case:
            a, b = a.lower(), b.lower()
        return a == b
    return False


def _as_number(value: Any) -> float | None:
    """Numbers and numeric strings compare as numbers, so CSV '1.0' equals JSON 1."""
    if isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        try:
            return float(value.strip())
        except ValueError:
            return None
    return None


def _norm_key(value: Any) -> Any:
    """Make CSV '1', JSON 1 and '1.0' land on the same key."""
    number = _as_number(value)
    if number is None:
        return value
    return int(number) if number.is_integer() else number
