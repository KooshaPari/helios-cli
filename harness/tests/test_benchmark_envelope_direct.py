import json
import subprocess
import tempfile
from types import SimpleNamespace

import pytest

from harness.scripts.run_harness import run_runner
from harness.benchmark_envelope import add_envelope


def _initialize_git_repo(repo):
    subprocess.run(["git", "init", "-q", str(repo)], check=True)
    subprocess.run(["git", "-C", str(repo), "config", "user.email", "tests@example.invalid"], check=True)
    subprocess.run(["git", "-C", str(repo), "config", "user.name", "Harness tests"], check=True)
    subprocess.run(["git", "-C", str(repo), "add", "Makefile"], check=True)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "test fixture"], check=True)


def test_run_runner_emits_benchmark_envelope(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Makefile").write_text("check:\n\t@echo check\n")
    _initialize_git_repo(repo)
    out = tmp_path / "run.json"
    run_runner(str(repo), "strict-full", str(out), SimpleNamespace(
        replay=None, dry_run=True, max_parallel=2, timeout=2, retries=0,
        retry_delay=1.0, budget=None, continue_on_fail=False,
    ))
    payload = json.loads(out.read_text())
    try:
        import jsonschema
        from pathlib import Path
        schema_path = Path(__file__).parents[5] / "docs/sessions/20260722-agent-harness-portfolio/artifacts/benchmark_run.schema.json"
        jsonschema.Draft202012Validator(json.loads(schema_path.read_text())).validate(payload)
    except ModuleNotFoundError:
        pass
    assert payload["tenant_id"] == "phenotype"
    assert payload["session_id"].startswith("ses_")
    assert payload["run_id"].startswith("run_")
    assert payload["attempt_id"].startswith("att_")
    assert payload["subject"]["harness"] == "helios-harness"
    assert payload["subject"]["commit"] != "unknown"
    assert payload["deterministic_identity"]["inputs"]["commit"] == payload["subject"]["commit"]
    assert payload["provenance"]["collector"] == "helios-harness"
    assert payload["signature"]["algorithm"] == "placeholder"
    assert {event["type"] for event in payload["events"]} >= {"checkpoint", "compaction"}


def test_real_runs_promote_populated_tasks_and_runs():
    payload = add_envelope(
        {
            "commands": [{"command": "pytest -q", "bucket": "test", "required": True, "cwd": ".", "source": "Makefile"}],
            "runs": [{"command": "pytest -q", "returncode": 0, "timed_out": False, "duration_ms": 12}],
            "result_code": "PASS",
        },
        repo="triangle",
        profile="strict-full",
        plan_hash="a" * 64,
    )
    assert len(payload["tasks"]) == 1
    assert len(payload["runs"]) == 1
    assert payload["runs"][0]["task_id"] == payload["tasks"][0]["task_id"]
    assert payload["runs"][0]["status"] == "passed"


def test_add_envelope_rejects_missing_or_non_sha_subject_commit():
    with pytest.raises(ValueError, match="resolved Git SHA"):
        add_envelope(
            {"result_code": "PASS"},
            repo="triangle",
            profile="strict-full",
            plan_hash="a" * 64,
            subject_commit="unknown",
        )


def test_run_runner_rejects_non_git_subject(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Makefile").write_text("check:\n\t@echo check\n")
    with pytest.raises(ValueError, match="resolved Git SHA"):
        run_runner(
            str(repo),
            "strict-full",
            str(tmp_path / "run.json"),
            SimpleNamespace(
                replay=None, dry_run=True, max_parallel=2, timeout=2, retries=0,
                retry_delay=1.0, budget=None, continue_on_fail=False,
            ),
        )


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as directory:
        from pathlib import Path
        test_run_runner_emits_benchmark_envelope(Path(directory))
    print("direct_envelope_test_pass")
