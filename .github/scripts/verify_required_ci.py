#!/usr/bin/env python3
"""Verify that the required CI result cannot hide mandatory job failures."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


REQUIRED_JOBS = ("workspace", "deny", "ci_contract", "changed")
CONDITIONAL_JOBS = (
    "general",
    "cargo_shear",
    "argument_comment_lint_package",
    "argument_comment_lint_prebuilt",
)


def _job_block(workflow: str, job: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job)}:\s*\n(.*?)(?=^  [A-Za-z0-9_-]+:\s*\n|\Z)",
        workflow,
    )
    if match is None:
        raise ValueError(f"missing job: {job}")
    return match.group(1)


def contract_errors(workflow: str) -> list[str]:
    """Return violations of the required-check aggregation contract."""
    try:
        results = _job_block(workflow, "results")
    except ValueError as error:
        return [str(error)]

    needs_match = re.search(r"(?ms)^    needs:\s*(.*?)(?=^    [A-Za-z0-9_-]+:|\Z)", results)
    if needs_match is None:
        return ["results job has no needs dependency list"]

    needs = set(re.findall(r"\b[A-Za-z][A-Za-z0-9_]*\b", needs_match.group(1)))
    errors: list[str] = []
    for job in (*REQUIRED_JOBS, *CONDITIONAL_JOBS):
        if job not in needs:
            errors.append(f"results job does not depend on {job}")

    for job in REQUIRED_JOBS:
        assertion = (
            rf"\[\[\s*'\$\{{\{{\s*needs\.{re.escape(job)}\.result\s*\}}\}}'"
            rf"\s*==\s*'success'\s*\]\]"
        )
        if re.search(assertion, results) is None:
            errors.append(f"results job does not require {job} to succeed")
    return errors


def npm_contract_errors(workflow: str) -> list[str]:
    """Return violations that can hide an npm staging failure or lock drift."""
    errors: list[str] = []
    permissions_match = re.search(
        r"(?ms)^permissions:\s*\n((?:^[ \t]+.*\n?)+)", workflow
    )
    declared_permissions = (
        set(
            re.findall(
                r"(?m)^\s+([a-z-]+):\s*(read|write|none)\s*(?:#.*)?$",
                permissions_match.group(1),
            )
        )
        if permissions_match is not None
        else set()
    )
    if declared_permissions != {("contents", "read")}:
        errors.append("npm CI token permissions must be exactly contents: read")

    if "pnpm install --frozen-lockfile" not in workflow:
        errors.append("npm CI does not enforce the committed lockfile")

    stage_match = re.search(
        r"(?ms)^            - name: Stage npm package\s*\n(.*?)"
        r"(?=^            - name:|\Z)",
        workflow,
    )
    if stage_match is None:
        errors.append("npm CI has no staging step")
    elif re.search(r"(?m)^\s*continue-on-error:\s*true\s*$", stage_match.group(1)):
        errors.append("npm staging is allowed to fail")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "workflow",
        nargs="?",
        type=Path,
        default=Path(".github/workflows/rust-ci.yml"),
    )
    parser.add_argument(
        "--npm-workflow",
        type=Path,
        default=Path(".github/workflows/ci.yml"),
    )
    args = parser.parse_args()
    errors = contract_errors(args.workflow.read_text(encoding="utf-8"))
    errors.extend(npm_contract_errors(args.npm_workflow.read_text(encoding="utf-8")))
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    print("required CI contract is truthful")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
