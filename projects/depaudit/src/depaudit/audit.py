"""Combine packages, advisories and license policy into findings."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .advisories import SEVERITIES, AdvisoryIndex
from .discovery import collect_packages
from .licenses import Policy, resolve_license
from .models import Package


@dataclass(frozen=True)
class Finding:
    kind: str  # "vulnerability" or "license"
    package: Package
    detail: str  # advisory id or license expression
    severity: str  # advisory severity; "license" findings use "high"
    message: str


def audit_packages(
    packages: list[Package],
    index: AdvisoryIndex,
    policy: Policy,
    license_map: dict[str, str] | None = None,
) -> list[Finding]:
    findings = []
    for pkg in packages:
        for adv in index.matches(pkg):
            fixed = ", ".join(adv.fixed_versions)
            msg = adv.summary + (f" (fixed in {fixed})" if fixed else "")
            findings.append(Finding("vulnerability", pkg, adv.id, adv.severity, msg))
        lic = resolve_license(pkg, license_map)
        if not policy.permits(lic):
            findings.append(Finding("license", pkg, lic, "high", f"license {lic} violates policy"))
    return findings


def audit_repo(root: Path, index: AdvisoryIndex, policy: Policy, license_map=None) -> list[Finding]:
    return audit_packages(collect_packages(root), index, policy, license_map)


def filter_severity(findings: list[Finding], minimum: str) -> list[Finding]:
    floor = SEVERITIES.index(minimum)
    # License violations are policy failures, not graded, so they always pass.
    return [f for f in findings if f.kind == "license" or SEVERITIES.index(f.severity) >= floor]
