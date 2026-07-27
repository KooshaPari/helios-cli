import json
import os
import sys
from pathlib import Path
from unittest.mock import patch

from harness.run_harness import main


def _run(args: list[str], cwd: Path) -> None:
    """Exercise the installed public CLI module without source-path injection."""
    previous_cwd = Path.cwd()
    try:
        os.chdir(cwd)
        with patch.object(sys, "argv", ["helios-harness", *args]):
            main()
    finally:
        os.chdir(previous_cwd)


def test_harness_dry_run_and_plan_hash(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "package.json").write_text('{"scripts":{"lint":"echo lint","test":"echo test","build":"echo build"}}')
    out_discover = tmp_path / "discover.json"
    out_run = tmp_path / "run.json"

    _run(
        [
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
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Makefile").write_text("check:\n\t@echo check\n")
    out_first = tmp_path / "first.json"
    out_second = tmp_path / "second.json"

    _run(
        [
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

    first_payload = json.loads(out_first.read_text())
    _run(
        [
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

    schema = Path("harness/schemas/harness-evidence.schema.json").resolve()
    if not schema.exists():
        schema = Path("schemas/harness-evidence.schema.json").resolve()
    _run(["validate", "--schema", str(schema), "--file", str(out_second)], cwd=tmp_path)
