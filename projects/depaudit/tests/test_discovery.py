from depaudit.discovery import collect_packages, find_manifests


def test_finds_manifests_and_skips_vendor_dirs(tmp_path):
    (tmp_path / "requirements.txt").write_text("a==1.0\n")
    (tmp_path / "web").mkdir()
    (tmp_path / "web" / "package-lock.json").write_text('{"packages": {}}')
    (tmp_path / "node_modules" / "x").mkdir(parents=True)
    (tmp_path / "node_modules" / "x" / "package-lock.json").write_text("{}")
    found = [p.relative_to(tmp_path).as_posix() for p in find_manifests(tmp_path)]
    assert found == ["requirements.txt", "web/package-lock.json"]


def test_collect_deduplicates_across_manifests(tmp_path):
    (tmp_path / "requirements.txt").write_text("a==1.0\n")
    (tmp_path / "sub").mkdir()
    (tmp_path / "sub" / "requirements.txt").write_text("a==1.0\nb==2.0\n")
    assert [(p.name, p.version) for p in collect_packages(tmp_path)] == [("a", "1.0"), ("b", "2.0")]
