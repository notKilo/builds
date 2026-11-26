import os
import subprocess
from pathlib import Path

import pytest


def git(repo: Path, *args: str, date: str | None = None) -> None:
    env = dict(os.environ, GIT_AUTHOR_NAME="Ann", GIT_AUTHOR_EMAIL="ann@x.io",
               GIT_COMMITTER_NAME="Ann", GIT_COMMITTER_EMAIL="ann@x.io")
    if date:
        env["GIT_AUTHOR_DATE"] = env["GIT_COMMITTER_DATE"] = date
    subprocess.run(["git", "-C", str(repo), *args], check=True, env=env,
                   capture_output=True)


@pytest.fixture
def repo(tmp_path):
    git(tmp_path, "init", "-q")
    return tmp_path


def commit(repo: Path, files: dict[str, str], date: str, author: str | None = None):
    for name, body in files.items():
        p = repo / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(body)
    git(repo, "add", "-A")
    args = ["commit", "-q", "-m", "c"]
    if author:
        args += ["--author", f"{author} <{author.lower()}@x.io>"]
    git(repo, *args, date=date)
