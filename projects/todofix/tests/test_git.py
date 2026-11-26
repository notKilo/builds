import subprocess

from conftest import commit
from todofix.blame import blame_file
from todofix.discovery import discover, is_ignored


def test_discover_skips_binary_and_ignored(repo):
    commit(repo, {"a.py": "# TODO x\n", "vendor/b.py": "# TODO y\n"}, "2024-01-01T00:00:00")
    (repo / "img.png").write_bytes(b"\x89PNG\0\0")
    subprocess.run(["git", "-C", str(repo), "add", "img.png"], check=True)
    found = discover(repo, ignore=["vendor/*"])
    assert found == ["a.py"]


def test_is_ignored_matches_basename():
    assert is_ignored("src/x.min.js", ["*.min.js"])
    assert not is_ignored("src/x.js", ["*.min.js"])


def test_blame_picks_author_and_date_per_line(repo):
    commit(repo, {"a.py": "x = 1\n# TODO old\n"}, "2020-03-04T10:00:00", author="Ann")
    commit(repo, {"a.py": "x = 1\n# TODO old\ny = 2\n# FIXME new\n"}, "2023-06-01T10:00:00", author="Bob")
    b = blame_file(repo, "a.py", [2, 4])
    assert b[2].author == "Ann" and b[2].when.year == 2020
    assert b[4].author == "Bob" and b[4].when.year == 2023
