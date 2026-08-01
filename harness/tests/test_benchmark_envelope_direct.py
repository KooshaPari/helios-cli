import json
import subprocess
import tempfile
from pathlib import Path
from types import SimpleNamespace

import pytest
from harness.benchmark_envelope import add_envelope
from harness.scripts.run_harness import run_runner


def _initialize_git_repo(repo):
    subprocess.run(["git", "init", "-q", str(repo)], check=True)
    subprocess.run(["git", "-C", str(repo), "config", "user.email", "tests@example.invalid"], check=True)
    subprocess.run(["git", "-C", str(repo), "config", "user.name", "Harness tests"], check=True)
    subprocess.run(["git", "-C", str(repo), "add", "Makefile"], check=True)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "test fixture"], check=True)


def test_run_runner_emits_benchmark_envelope(tmp_path):
    # Traces to: FR-HELIOS-IO-006 (resolved source provenance).
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

        package_schema = Path(__file__).parents[1] / "schemas/benchmark_run.schema.json"
        jsonschema.Draft202012Validator(json.loads(package_schema.read_text())).validate(payload)

        session_schema = Path(
            "/Users/kooshapari/CodeProjects/Phenotype/repos/docs/sessions/"
            "20260722-agent-harness-portfolio/artifacts/benchmark_run.schema.json"
        )
        if session_schema.exists():
            parity_payload = json.loads(json.dumps(payload))
            parity_payload["subject"].pop("ref")
            parity_payload["provenance"].pop("source_ref")
            parity_payload["provenance"].pop("source_sha")
            jsonschema.Draft202012Validator(
                json.loads(session_schema.read_text())
            ).validate(parity_payload)
    except ModuleNotFoundError:
        pass
    assert payload["tenant_id"] == "phenotype"
    assert payload["session_id"].startswith("ses_")
    assert payload["run_id"].startswith("run_")
    assert payload["attempt_id"].startswith("att_")
    assert payload["subject"]["harness"] == "helios-harness"
    assert payload["subject"]["ref"]
    assert payload["subject"]["commit"] != "unknown"
    assert len(payload["subject"]["commit"]) == 40
    assert payload["deterministic_identity"]["inputs"]["commit"] == payload["subject"]["commit"]
    assert payload["provenance"]["source_ref"] == payload["subject"]["ref"]
    assert payload["provenance"]["source_sha"] == payload["subject"]["commit"]
    assert "ref" not in payload["deterministic_identity"]["inputs"]
    assert payload["provenance"]["collector"] == "helios-harness"
    assert payload["signature"]["algorithm"] == "placeholder"
    assert {event["type"] for event in payload["events"]} >= {"checkpoint", "compaction"}


def test_real_runs_promote_populated_tasks_and_runs():
    # Traces to: FR-HELIOS-IO-006 (run evidence projection).
    payload = add_envelope(
        {
            "commands": [{"command": "pytest -q", "bucket": "test", "required": True, "cwd": ".", "source": "Makefile"}],
            "runs": [{"command": "pytest -q", "returncode": 0, "timed_out": False, "duration_ms": 12}],
            "result_code": "PASS",
        },
        repo="triangle",
        profile="strict-full",
        plan_hash="a" * 64,
        subject_commit="a" * 40,
        subject_ref="main",
    )
    assert len(payload["tasks"]) == 1
    assert len(payload["runs"]) == 1
    assert payload["runs"][0]["task_id"] == payload["tasks"][0]["task_id"]
    assert payload["runs"][0]["status"] == "passed"


def test_warn_result_preserves_code_and_content_addressed_metadata():
    # Traces to: FR-HELIOS-IO-006 (warning and replay evidence fidelity).
    payload = add_envelope(
        {
            "commands": [{"command": "make check", "bucket": "test", "required": True}],
            "runs": [
                {
                    "command": "make check",
                    "returncode": None,
                    "timed_out": False,
                    "duration_ms": 12,
                    "stdout_file": "artifacts/stdout.log",
                    "stderr_file": "artifacts/stderr.log",
                    "artifact_dir": "artifacts/task-1",
                    "started_at": "2026-01-01T00:00:00+00:00",
                    "finished_at": "2026-01-01T00:00:01+00:00",
                    "attempts": 1,
                    "error": None,
                    "skipped": True,
                }
            ],
            "result_code": "WARN",
        },
        repo="triangle",
        profile="strict-full",
        plan_hash="a" * 64,
        subject_commit="a" * 40,
        subject_ref="main",
    )
    assert payload["result_code"] == "WARN"
    assert payload["result"]["status"] == "failed"
    assert len(payload["result"]["outcome_sha256"]) == 64
    run = payload["runs"][0]
    assert set(run) == {"run_id", "task_id", "command", "status", "returncode", "duration_ms"}
    assert run["status"] == "passed"


def test_add_envelope_accepts_sha256_subject_commit():
    # Traces to: FR-HELIOS-IO-006 (full Git object identity).
    payload = add_envelope(
        {"result_code": "PASS"},
        repo="triangle",
        profile="strict-full",
        plan_hash="a" * 64,
        subject_commit="a" * 64,
        subject_ref="main",
    )
    assert payload["subject"]["commit"] == "a" * 64


def test_add_envelope_rejects_missing_or_non_sha_subject_commit():
    with pytest.raises(ValueError, match="resolved Git SHA"):
        add_envelope(
            {"result_code": "PASS"},
            repo="triangle",
            profile="strict-full",
            plan_hash="a" * 64,
            subject_commit="unknown",
            subject_ref="main",
        )


def test_run_runner_rejects_non_git_subject(tmp_path):
    # Traces to: FR-HELIOS-IO-006 (explicit unresolved provenance).
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Makefile").write_text("check:\n\t@echo check\n")
    out = tmp_path / "run.json"
    run_runner(
        str(repo),
        "strict-full",
        str(out),
        SimpleNamespace(
            replay=None, dry_run=True, max_parallel=2, timeout=2, retries=0,
            retry_delay=1.0, budget=None, continue_on_fail=False,
        ),
    )
    payload = json.loads(out.read_text())
    assert payload["result_code"] == "WARN"
    assert payload["provenance"]["status"] == "unresolved"
    assert payload["provenance"]["reason"] == "non_git_repository"


def test_add_envelope_rejects_missing_source_ref():
    with pytest.raises(ValueError, match="resolved source ref"):
        add_envelope(
            {"result_code": "PASS"},
            repo="triangle",
            profile="strict-full",
            plan_hash="a" * 64,
            subject_commit="a" * 40,
            subject_ref="",
        )


def test_replay_hash_is_stable_across_collection_timestamps():
    kwargs = {
        "repo": "triangle",
        "profile": "strict-full",
        "plan_hash": "a" * 64,
        "subject_commit": "a" * 40,
        "subject_ref": "main",
    }
    first = add_envelope({"result_code": "PASS"}, **kwargs)
    second = add_envelope({"result_code": "PASS"}, **kwargs)

    assert first["result"]["replay_hash"] == second["result"]["replay_hash"]


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as directory:
        from pathlib import Path
        test_run_runner_emits_benchmark_envelope(Path(directory))
    print("direct_envelope_test_pass")
