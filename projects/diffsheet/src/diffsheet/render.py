"""Render a DiffResult as text, markdown, JSON or HTML."""

from __future__ import annotations

import html
import json
from typing import Any

from .diff import DiffResult, RowChange


def _fmt(value: Any) -> str:
    return "<empty>" if value is None else str(value)


def _key_label(keys: list[str], key: tuple) -> str:
    return ", ".join(f"{k}={v}" for k, v in zip(keys, key))


def _row_key(result: DiffResult, row: dict) -> tuple:
    return tuple(row[k] for k in result.keys)


def summary_line(r: DiffResult) -> str:
    return (
        f"{len(r.added)} added, {len(r.removed)} removed, "
        f"{len(r.changed)} changed, {r.unchanged} unchanged"
    )


def render_text(r: DiffResult) -> str:
    lines: list[str] = []
    for c in r.columns_removed:
        lines.append(f"- column {c}")
    for c in r.columns_added:
        lines.append(f"+ column {c}")
    for row in r.removed:
        lines.append(f"- {_key_label(r.keys, _row_key(r, row))}")
    for row in r.added:
        lines.append(f"+ {_key_label(r.keys, _row_key(r, row))}")
    for rc in r.changed:
        lines.append(f"~ {_key_label(r.keys, rc.key)}")
        for ch in rc.changes:
            lines.append(f"    {ch.column}: {_fmt(ch.old)} -> {_fmt(ch.new)}")
    lines.append(summary_line(r))
    return "\n".join(lines)


def render_markdown(r: DiffResult) -> str:
    out = ["## diffsheet", "", summary_line(r), ""]
    if r.columns_added or r.columns_removed:
        out.append("### Columns")
        out += [f"- removed `{c}`" for c in r.columns_removed]
        out += [f"- added `{c}`" for c in r.columns_added]
        out.append("")
    if r.removed:
        out.append("### Removed")
        out += [f"- `{_key_label(r.keys, _row_key(r, row))}`" for row in r.removed]
        out.append("")
    if r.added:
        out.append("### Added")
        out += [f"- `{_key_label(r.keys, _row_key(r, row))}`" for row in r.added]
        out.append("")
    if r.changed:
        out += ["### Changed", "", "| Key | Column | Old | New |", "|---|---|---|---|"]
        for rc in r.changed:
            for ch in rc.changes:
                label = _key_label(r.keys, rc.key)
                out.append(f"| {_md(label)} | {_md(ch.column)} | {_md(_fmt(ch.old))} | {_md(_fmt(ch.new))} |")
    return "\n".join(out).rstrip() + "\n"


def _md(text: str) -> str:
    return text.replace("|", "\\|")


def to_dict(r: DiffResult) -> dict:
    def changed(rc: RowChange) -> dict:
        return {
            "key": dict(zip(r.keys, rc.key)),
            "changes": [{"column": c.column, "old": c.old, "new": c.new} for c in rc.changes],
        }

    return {
        "summary": {
            "added": len(r.added),
            "removed": len(r.removed),
            "changed": len(r.changed),
            "unchanged": r.unchanged,
        },
        "columns_added": r.columns_added,
        "columns_removed": r.columns_removed,
        "added": r.added,
        "removed": r.removed,
        "changed": [changed(rc) for rc in r.changed],
    }


def render_json(r: DiffResult) -> str:
    return json.dumps(to_dict(r), indent=2, ensure_ascii=False)


_CSS = (
    "body{font:14px system-ui,sans-serif;margin:2rem;color:#222}"
    "table{border-collapse:collapse;margin:1rem 0}td,th{border:1px solid #ccc;padding:.3rem .6rem;text-align:left}"
    ".add{background:#e6ffed}.del{background:#ffeef0}.chg{background:#fff8c5}"
)


def render_html(r: DiffResult) -> str:
    e = html.escape
    parts = [
        "<!doctype html><meta charset=utf-8><title>diffsheet report</title>",
        f"<style>{_CSS}</style>",
        f"<h1>diffsheet report</h1><p>{e(summary_line(r))}</p>",
    ]
    if r.columns_added or r.columns_removed:
        parts.append("<h2>Columns</h2><ul>")
        parts += [f"<li class=del>removed {e(c)}</li>" for c in r.columns_removed]
        parts += [f"<li class=add>added {e(c)}</li>" for c in r.columns_added]
        parts.append("</ul>")
    for title, css, rows in (("Removed", "del", r.removed), ("Added", "add", r.added)):
        if rows:
            parts.append(f"<h2>{title}</h2><ul>")
            parts += [f"<li class={css}>{e(_key_label(r.keys, _row_key(r, row)))}</li>" for row in rows]
            parts.append("</ul>")
    if r.changed:
        parts.append("<h2>Changed</h2><table><tr><th>Key<th>Column<th>Old<th>New")
        for rc in r.changed:
            for ch in rc.changes:
                parts.append(
                    f"<tr class=chg><td>{e(_key_label(r.keys, rc.key))}<td>{e(ch.column)}"
                    f"<td>{e(_fmt(ch.old))}<td>{e(_fmt(ch.new))}"
                )
        parts.append("</table>")
    return "\n".join(parts) + "\n"


RENDERERS = {
    "text": render_text,
    "markdown": render_markdown,
    "json": render_json,
    "html": render_html,
}
