#!/usr/bin/env python3
"""
Bump versions across the repo to match the Python wrapper.

Usage examples:
  - Set explicit version:  scripts/bump_versions.py --set 1.2.3
  - Bump patch version:    scripts/bump_versions.py --patch (0.0.1)
  - Bump minor version:    scripts/bump_versions.py --minor (0.1.0)
  - Bump major version:    scripts/bump_versions.py --major (1.0.0)

Optional:
  --dry-run        Show changes without writing files
  --git-commit     Create a release commit
  --git-tag        Create a git tag vX.Y.Z
  --git-push       Push commit and tag to origin

Notes:
  - Source of truth is crates/py/pyproject.toml [project].version.
  - Also updates crates/py/Cargo.toml to the same version.
  - Updates all workspace crate versions, Tauri config version, and UI package.json.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path
from typing import Iterable, Tuple


def read_text(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def write_text(p: Path, s: str, dry_run: bool) -> None:
    if dry_run:
        return
    p.write_text(s, encoding="utf-8")


def parse_semver(v: str) -> Tuple[int, int, int, str]:
    # Accept forms like 1.2.3, 1.2.3-alpha.1, 1.2.3+meta
    main, *rest = v.split("-", 1)
    pre = f"-{rest[0]}" if rest else ""
    main = main.split("+", 1)[0]
    parts = main.split(".")
    if len(parts) != 3 or not all(p.isdigit() for p in parts):
        raise ValueError(f"Unsupported version format: {v}")
    return int(parts[0]), int(parts[1]), int(parts[2]), pre


def bump_version_str(v: str, which: str) -> str:
    major, minor, patch, _pre = parse_semver(v)
    if which == "major":
        major, minor, patch = major + 1, 0, 0
    elif which == "minor":
        minor, patch = minor + 1, 0
    elif which == "patch":
        patch += 1
    else:
        raise ValueError(f"Unknown bump type: {which}")
    return f"{major}.{minor}.{patch}"


def get_repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def get_python_wrapper_version(root: Path) -> str:
    # Prefer pyproject.toml [project].version
    pyproj = root / "crates/py/pyproject.toml"
    cargo = root / "crates/py/Cargo.toml"
    v = None
    if pyproj.exists():
        text = read_text(pyproj)
        in_proj = False
        for line in text.splitlines():
            if line.strip().startswith("[project]"):
                in_proj = True
                continue
            if in_proj and line.strip().startswith("["):
                break
            m = re.match(r"\s*version\s*=\s*\"([^\"]+)\"", line)
            if in_proj and m:
                v = m.group(1)
                break
    if not v and cargo.exists():
        text = read_text(cargo)
        in_pkg = False
        for line in text.splitlines():
            if line.strip() == "[package]":
                in_pkg = True
                continue
            if in_pkg and line.strip().startswith("["):
                break
            m = re.match(r"\s*version\s*=\s*\"([^\"]+)\"", line)
            if in_pkg and m:
                v = m.group(1)
                break
    if not v:
        raise SystemExit("Could not determine Python wrapper version")
    return v


def set_python_wrapper_version(root: Path, new_version: str, dry_run: bool) -> Iterable[Path]:
    changed = []
    # Update pyproject.toml [project].version
    pyproj = root / "crates/py/pyproject.toml"
    if pyproj.exists():
        text = read_text(pyproj)
        lines = text.splitlines()
        out = []
        in_proj = False
        changed_pyproj = False
        for line in lines:
            if line.strip().startswith("[project]"):
                in_proj = True
                out.append(line)
                continue
            if in_proj and line.strip().startswith("["):
                in_proj = False
            if in_proj:
                m = re.match(r"(\s*version\s*=\s*\")([^\"]+)(\"\s*)$", line)
                if m:
                    line = f"{m.group(1)}{new_version}{m.group(3)}"
                    changed_pyproj = True
            out.append(line)
        if changed_pyproj:
            write_text(pyproj, "\n".join(out) + ("\n" if text.endswith("\n") else ""), dry_run)
            changed.append(pyproj)

    # Update crates/py/Cargo.toml [package].version
    cargo = root / "crates/py/Cargo.toml"
    if cargo.exists():
        updated = update_cargo_package_version(cargo, new_version, dry_run)
        if updated:
            changed.append(cargo)

    return changed


def update_cargo_package_version(path: Path, new_version: str, dry_run: bool) -> bool:
    text = read_text(path)
    lines = text.splitlines()
    out = []
    in_pkg = False
    changed = False
    for line in lines:
        if line.strip() == "[package]":
            in_pkg = True
            out.append(line)
            continue
        if in_pkg and line.strip().startswith("["):
            in_pkg = False
        if in_pkg:
            m = re.match(r"(\s*version\s*=\s*\")([^\"]+)(\"\s*)$", line)
            if m:
                line = f"{m.group(1)}{new_version}{m.group(3)}"
                changed = True
        out.append(line)
    if changed:
        write_text(path, "\n".join(out) + ("\n" if text.endswith("\n") else ""), dry_run)
    return changed


def update_all_cargo_versions(root: Path, new_version: str, dry_run: bool) -> Iterable[Path]:
    changed: list[Path] = []
    cargo_files = [
        *sorted((root / "crates").glob("*/Cargo.toml")),
        root / "apps/tauri/src-tauri/Cargo.toml",
    ]
    for cf in cargo_files:
        if not cf.exists():
            continue
        if update_cargo_package_version(cf, new_version, dry_run):
            changed.append(cf)
    return changed


def update_tauri_conf(root: Path, new_version: str, dry_run: bool) -> Iterable[Path]:
    p = root / "apps/tauri/src-tauri/tauri.conf.json"
    if not p.exists():
        return []
    data = json.loads(read_text(p))
    if data.get("version") == new_version:
        return []
    data["version"] = new_version
    if not dry_run:
        p.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return [p]


def update_ui_package_json(root: Path, new_version: str, dry_run: bool) -> Iterable[Path]:
    p = root / "apps/tauri/ui/package.json"
    if not p.exists():
        return []
    data = json.loads(read_text(p))
    if data.get("version") == new_version:
        return []
    data["version"] = new_version
    if not dry_run:
        p.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return [p]


def git(cmd: list[str]) -> None:
    subprocess.check_call(["git", *cmd])


def main() -> None:
    ap = argparse.ArgumentParser(description="Bump repo versions to match Python wrapper")
    g = ap.add_mutually_exclusive_group()
    g.add_argument("--set", dest="set_version", help="Set explicit version (e.g. 1.2.3)")
    g.add_argument("--patch", action="store_true", help="Bump patch version")
    g.add_argument("--minor", action="store_true", help="Bump minor version")
    g.add_argument("--major", action="store_true", help="Bump major version")
    ap.add_argument("--dry-run", action="store_true", help="Show changes without writing files")
    ap.add_argument("--git-commit", action="store_true", help="Create a release commit")
    ap.add_argument("--git-tag", action="store_true", help="Create a git tag vX.Y.Z")
    ap.add_argument("--git-push", action="store_true", help="Push commit and tag to origin")
    args = ap.parse_args()

    root = get_repo_root()
    current = get_python_wrapper_version(root)

    if args.set_version:
        new_version = args.set_version
    else:
        which = "patch" if (args.patch or args.minor or args.major) is False else (
            "major" if args.major else "minor" if args.minor else "patch"
        )
        new_version = bump_version_str(current, which)

    print(f"Current Python wrapper version: {current}")
    print(f"Target version: {new_version}")

    changed_paths = []
    changed_paths += set_python_wrapper_version(root, new_version, args.dry_run)
    changed_paths += update_all_cargo_versions(root, new_version, args.dry_run)
    changed_paths += update_tauri_conf(root, new_version, args.dry_run)
    changed_paths += update_ui_package_json(root, new_version, args.dry_run)

    # De-duplicate while preserving order
    seen = set()
    ordered = []
    for p in changed_paths:
        if p not in seen:
            seen.add(p)
            ordered.append(p)

    if ordered:
        print("Updated:")
        for p in ordered:
            print(f" - {p.relative_to(root)}")
    else:
        print("No files needed updating.")

    if args.dry_run:
        print("Dry run complete — no files written.")
        return

    if args.git_commit:
        git(["add", "."])
        git(["commit", "-m", f"chore(release): v{new_version}"])

    if args.git_tag:
        # Create annotated tag
        git(["tag", "-a", f"v{new_version}", "-m", f"v{new_version}"])

    if args.git_push:
        # Push current branch and tags
        git(["push"])
        git(["push", "--tags"])


if __name__ == "__main__":
    main()
