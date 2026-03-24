#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT / "config" / "patch_superset.json"
MODULE_BAZEL_PATH = ROOT / "MODULE.bazel"
SHELL_README_PATH = ROOT / "codex-rs" / "shell-escalation" / "README.md"


def load_manifest() -> dict:
    return json.loads(MANIFEST_PATH.read_text())


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resolve_secondary_root(manifest: dict, secondary_root: str | None) -> Path:
    if secondary_root:
        return (ROOT / secondary_root).resolve() if not Path(secondary_root).is_absolute() else Path(secondary_root)
    return (ROOT / manifest["allowed_secondary_root"]).resolve()


def inventory(manifest: dict) -> int:
    print("Patch superset inventory:")
    for patch in manifest["patches"]:
        path = ROOT / patch["path"]
        digest = sha256(path)[:12] if path.exists() else "missing"
        print(
            f"- {patch['id']} | category={patch['category']} integration={patch['integration']} "
            f"policy={patch['secondary_policy']} path={patch['path']} sha256={digest}"
        )
    return 0


def check(manifest: dict) -> int:
    errors: list[str] = []
    module_bazel = MODULE_BAZEL_PATH.read_text()
    shell_readme = SHELL_README_PATH.read_text()

    for patch in manifest["patches"]:
        path = ROOT / patch["path"]
        if not path.exists():
            errors.append(f"missing patch file: {patch['path']}")
            continue

        module_reference = patch.get("module_reference")
        if module_reference and module_reference not in module_bazel:
            errors.append(f"missing MODULE.bazel reference for {patch['id']}: {module_reference}")

        readme_reference = patch.get("readme_reference")
        if readme_reference and readme_reference not in shell_readme:
            errors.append(f"missing shell escalation README reference for {patch['id']}: {readme_reference}")

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print(f"Verified {len(manifest['patches'])} patch entries.")
    return 0


def compare_secondary(manifest: dict, secondary_root: Path) -> int:
    print(f"Comparing against secondary root: {secondary_root}")
    mismatches = 0

    for patch in manifest["patches"]:
        primary_path = ROOT / patch["path"]
        secondary_path = secondary_root / patch["path"]
        if not secondary_path.exists():
            print(f"- {patch['id']} | secondary=missing")
            mismatches += 1
            continue

        primary_hash = sha256(primary_path)
        secondary_hash = sha256(secondary_path)
        if primary_hash == secondary_hash:
            status = "match"
        elif patch["secondary_policy"] == "prefer_primary":
            status = "prefer_primary"
        else:
            status = "mismatch"
        print(
            f"- {patch['id']} | secondary={status} primary={primary_hash[:12]} "
            f"secondary={secondary_hash[:12]}"
        )
        if status not in {"match", "prefer_primary"}:
            mismatches += 1

    return 1 if mismatches else 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Inventory and verify the heliosCLI patch superset.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("inventory", help="List the compiled patch superset.")
    subparsers.add_parser("check", help="Verify manifest entries against live repo references.")

    compare_parser = subparsers.add_parser(
        "compare-secondary",
        help="Compare manifest patch files against the secondary rewrite repo.",
    )
    compare_parser.add_argument("--secondary-root", default=None)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    manifest = load_manifest()

    if args.command == "inventory":
        return inventory(manifest)
    if args.command == "check":
        return check(manifest)
    if args.command == "compare-secondary":
        return compare_secondary(manifest, resolve_secondary_root(manifest, args.secondary_root))

    parser.error(f"unsupported command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
