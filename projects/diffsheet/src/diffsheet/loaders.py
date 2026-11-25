"""Load CSV and JSON files into a uniform Table."""

from __future__ import annotations

import csv
import io
import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


class LoadError(ValueError):
    pass


@dataclass
class Table:
    columns: list[str]
    rows: list[dict[str, Any]] = field(default_factory=list)


def load_table(path: str | Path, fmt: str | None = None) -> Table:
    """Load a table from a file; format is guessed from the extension."""
    path = Path(path)
    text = path.read_text(encoding="utf-8-sig")
    return parse_table(text, fmt or _guess_format(path))


def parse_table(text: str, fmt: str) -> Table:
    if fmt == "csv":
        return _parse_csv(text, ",")
    if fmt == "tsv":
        return _parse_csv(text, "\t")
    if fmt == "json":
        return _parse_json(text)
    raise LoadError(f"unsupported format: {fmt!r}")


def _guess_format(path: Path) -> str:
    suffix = path.suffix.lower().lstrip(".")
    if suffix in ("csv", "tsv", "json"):
        return suffix
    raise LoadError(f"cannot guess format of {path.name}; pass --format")


def _parse_csv(text: str, delimiter: str) -> Table:
    reader = csv.DictReader(io.StringIO(text), delimiter=delimiter)
    columns = list(reader.fieldnames or [])
    if len(set(columns)) != len(columns):
        raise LoadError("duplicate column names in header")
    rows = [{c: row.get(c) for c in columns} for row in reader]
    return Table(columns, rows)


def _parse_json(text: str) -> Table:
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        raise LoadError(f"invalid JSON: {exc}") from exc
    if isinstance(data, dict):
        # {"id1": {...}, "id2": {...}} -> rows with the outer key as "_key"
        if not all(isinstance(v, dict) for v in data.values()):
            raise LoadError("JSON object must map keys to objects")
        data = [{"_key": k, **v} for k, v in data.items()]
    if not isinstance(data, list) or not all(isinstance(r, dict) for r in data):
        raise LoadError("JSON must be an array of objects or an object of objects")
    columns: list[str] = []
    for row in data:
        for name in row:
            if name not in columns:
                columns.append(name)
    return Table(columns, [{c: row.get(c) for c in columns} for row in data])
