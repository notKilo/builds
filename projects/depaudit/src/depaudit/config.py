"""Load .depaudit.toml from the audited directory."""
from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from pathlib import Path

CONFIG_NAME = ".depaudit.toml"


@dataclass
class Config:
    allow: list[str] = field(default_factory=list)
    deny: list[str] = field(default_factory=list)
    allow_unknown: bool = True
    min_severity: str = "low"
    advisories: list[str] = field(default_factory=list)
    license_map: str | None = None


def load_config(root: Path) -> Config:
    path = root / CONFIG_NAME
    if not path.is_file():
        return Config()
    data = tomllib.loads(path.read_text())
    return Config(
        allow=data.get("licenses", {}).get("allow", []),
        deny=data.get("licenses", {}).get("deny", []),
        allow_unknown=data.get("licenses", {}).get("allow_unknown", True),
        min_severity=data.get("severity", {}).get("minimum", "low"),
        advisories=data.get("advisories", {}).get("files", []),
        license_map=data.get("licenses", {}).get("map"),
    )
