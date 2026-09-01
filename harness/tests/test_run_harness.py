import json
import subprocess
import sys
from pathlib import Path

import pytest
from harness.scripts.run_harness import _legacy_module

SCRIPT = Path("harness/scripts/run-harness.py").resolve()
# Alternative path for running from harness directory
if not SCRIPT.exists():
    SCRIPT = Path("scripts/run-harness.py").resolve()


def _run(cmd, cwd: Path) -> str:
    proc = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout


def _initialize_git_repo(repo: Path) -> None:
    subprocess.run(["git", "init", "-q", str(repo)], check=True)
    subprocess.run(
        ["git", "-C", str(repo), "config", "user.email", "tests@example.invalid"],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(repo), "config", "user.name", "Harness tests"],
        check=True,
    )
    subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "test fixture"], check=True)


def test_validated_output_path_rejects_workspace_escape(tmp_path, monkeypatch):
    # Traces to: FR-HELIOS-IO-006 (bounded evidence output).
    monkeypatch.chdir(tmp_path)
    write_output = _legacy_module()._write_output
    nested = tmp_path / "artifacts"
    nested.mkdir()

    write_output("artifacts/run.json", "{}")
    assert (nested / "run.json").read_text() == "{}"
    with pytest.raises(ValueError, match="inside the invoking workspace"):
        write_output("../outside.json", "{}")

    (tmp_path / "link").symlink_to(Path("/tmp"), target_is_directory=True)
    with pytest.raises(ValueError, match="inside the invoking workspace"):
        write_output("link/run.json", "{}")


def test_harness_dry_run_and_plan_hash(tmp_path):
    # Traces to: FR-HELIOS-IO-006 (strict dry-run envelope).
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "package.json").write_text(
        '{"scripts":{"lint":"echo lint","test":"echo test","build":"echo build"}}'
    )
    _initialize_git_repo(repo)
    out_discover = tmp_path / "discover.json"
    out_run = tmp_path / "run.json"

    _run(
        [
            sys.executable,
            str(SCRIPT),
            "discover",
            "--root",
            str(repo),
            "--out",
            str(out_discover),
            "--max-scan-depth",
            "2",
        ],
        cwd=tmp_path,
    )
    payload = json.loads(out_discover.read_text())
    assert payload["buckets"].get("static")

    _run(
        [
            sys.executable,
            str(SCRIPT),
            "run",
            "--repo",
            str(repo),
            "--out",
            str(out_run),
            "--dry-run",
            "--timeout",
            "2",
            "--retries",
            "1",
        ],
        cwd=tmp_path,
    )
    output = json.loads(out_run.read_text())
    assert output["result_code"] == "PASS"
    assert output["plan_hash"]


def test_harness_replay_and_validate(tmp_path):
    # Traces to: FR-HELIOS-IO-006 (deterministic replay evidence).
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Makefile").write_text("check:\n\t@echo check\n")
    _initialize_git_repo(repo)
    out_first = tmp_path / "first.json"
    out_second = tmp_path / "second.json"

    _run(
        [
            sys.executable,
            str(SCRIPT),
            "run",
            "--repo",
            str(repo),
            "--out",
            str(out_first),
            "--timeout",
            "2",
        ],
        cwd=tmp_path,
    )

    _run(
        [
            sys.executable,
            str(SCRIPT),
            "run",
            "--repo",
            str(repo),
            "--out",
            str(out_second),
            "--replay",
            str(out_first),
            "--timeout",
            "2",
        ],
        cwd=tmp_path,
    )

    second_payload = json.loads(out_second.read_text())
    assert second_payload["replay"]["same_plan"] is True
    assert "prior_plan_hash" in second_payload["replay"]
    assert second_payload["subject"]["commit"]
    assert second_payload["provenance"]["source_ref"] == second_payload["subject"]["ref"]

    schema = Path("harness/schemas/benchmark_run.schema.json").resolve()
    if not schema.exists():
        schema = Path("schemas/benchmark_run.schema.json").resolve()
    _run(
        [
            sys.executable,
            str(SCRIPT),
            "validate",
            "--schema",
            str(schema),
            "--file",
            str(out_second),
        ],
        cwd=tmp_path,
    )
