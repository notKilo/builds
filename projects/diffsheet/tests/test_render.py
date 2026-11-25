import json

from diffsheet.diff import DiffOptions, diff_tables
from diffsheet.loaders import Table
from diffsheet.render import render_html, render_json, render_markdown, render_text


def sample():
    cols = ["id", "name", "qty"]
    old = Table(cols, [
        {"id": "1", "name": "apple", "qty": "5"},
        {"id": "2", "name": "pear", "qty": "7"},
    ])
    new = Table(cols, [
        {"id": "1", "name": "apple", "qty": "6"},
        {"id": "3", "name": "<b>kiwi</b>", "qty": None},
    ])
    return diff_tables(old, new, DiffOptions(keys=["id"]))


def test_text():
    out = render_text(sample())
    assert "- id=2" in out
    assert "+ id=3" in out
    assert "~ id=1" in out
    assert "qty: 5 -> 6" in out
    assert out.splitlines()[-1] == "1 added, 1 removed, 1 changed, 0 unchanged"


def test_markdown_escapes_pipes():
    r = sample()
    r.changed[0].changes[0].new = "a|b"
    assert "a\\|b" in render_markdown(r)


def test_json_roundtrip():
    data = json.loads(render_json(sample()))
    assert data["summary"] == {"added": 1, "removed": 1, "changed": 1, "unchanged": 0}
    assert data["changed"][0] == {
        "key": {"id": "1"},
        "changes": [{"column": "qty", "old": "5", "new": "6"}],
    }


def test_html_escapes_values():
    r = sample()
    r.added[0]["id"] = "<script>"
    out = render_html(r)
    assert "<script>" not in out
    assert "&lt;script&gt;" in out
