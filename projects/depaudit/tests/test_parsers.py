import json

from depaudit.parsers import parse_cargo_lock, parse_package_lock, parse_requirements


def test_requirements_keeps_only_exact_pins(tmp_path):
    f = tmp_path / "requirements.txt"
    f.write_text(
        "# comment\nRequests==2.31.0\nflask>=2.0\nPyYAML[extra]==6.0.1 ; python_version>'3'\n"
        "-r other.txt\nurllib3==1.26.5  # pinned\n"
    )
    pkgs = parse_requirements(f)
    assert [(p.name, p.version) for p in pkgs] == [
        ("requests", "2.31.0"),
        ("pyyaml", "6.0.1"),
        ("urllib3", "1.26.5"),
    ]
    assert all(p.ecosystem == "PyPI" for p in pkgs)


def test_package_lock_v3_reads_nested_names_and_licenses(tmp_path):
    f = tmp_path / "package-lock.json"
    f.write_text(json.dumps({
        "lockfileVersion": 3,
        "packages": {
            "": {"name": "app", "version": "1.0.0"},
            "node_modules/lodash": {"version": "4.17.20", "license": "MIT"},
            "node_modules/a/node_modules/@scope/b": {"version": "2.0.0"},
        },
    }))
    pkgs = parse_package_lock(f)
    assert [(p.name, p.version, p.license) for p in pkgs] == [
        ("lodash", "4.17.20", "MIT"),
        ("@scope/b", "2.0.0", None),
    ]


def test_package_lock_v1_walks_nested_dependencies(tmp_path):
    f = tmp_path / "package-lock.json"
    f.write_text(json.dumps({
        "lockfileVersion": 1,
        "dependencies": {
            "a": {"version": "1.0.0", "dependencies": {"b": {"version": "0.1.0"}}},
        },
    }))
    assert [p.name for p in parse_package_lock(f)] == ["a", "b"]


def test_cargo_lock_skips_local_crates(tmp_path):
    f = tmp_path / "Cargo.lock"
    f.write_text(
        '[[package]]\nname = "me"\nversion = "0.1.0"\n\n'
        '[[package]]\nname = "serde"\nversion = "1.0.100"\n'
        'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
    )
    pkgs = parse_cargo_lock(f)
    assert [(p.ecosystem, p.name, p.version) for p in pkgs] == [("crates.io", "serde", "1.0.100")]
