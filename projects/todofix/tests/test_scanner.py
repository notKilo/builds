from todofix.scanner import scan_text


def one(path, line):
    res = scan_text(path, line)
    assert len(res) == 1, res
    return res[0]


def test_python_hash_comment():
    f = one("a.py", "x = 1  # TODO: handle negatives")
    assert (f.tag, f.text, f.line) == ("TODO", "handle negatives", 1)


def test_owner_and_ticket():
    f = one("a.go", "// FIXME(alice) [ABC-12]: leaks fd")
    assert f.owner == "alice" and f.ticket == "ABC-12" and f.text == "leaks fd"


def test_github_style_ticket():
    assert one("a.ts", "// TODO [#42] wire up").ticket == "#42"


def test_block_comment_closer_stripped():
    assert one("a.c", "/* HACK: temporary */").text == "temporary"


def test_star_continuation_line():
    assert one("a.java", " * TODO document this").tag == "TODO"


def test_tag_in_string_literal_ignored():
    assert scan_text("a.py", 'print("TODO list")') == []


def test_lowercase_prose_ignored():
    assert scan_text("a.py", "# we should todo this later") == []


def test_prose_before_tag_ignored():
    assert scan_text("a.py", "# see the TODO below") == []


def test_unknown_extension_skipped():
    assert scan_text("a.bin", "# TODO x") == []


def test_line_numbers():
    res = scan_text("a.py", "a\n# TODO one\nb\n# FIXME two\n")
    assert [(f.line, f.tag) for f in res] == [(2, "TODO"), (4, "FIXME")]
