import json
import tempfile
from types import SimpleNamespace

from harness.scripts.run_harness import run_runner


def test_run_runner_emits_benchmark_envelope(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / "Makefile").write_text("check:\n\t@echo check\n")
    out = tmp_path / "run.json"
    run_runner(
        str(repo),
        "strict-full",
        str(out),
        SimpleNamespace(
            replay=None,
            dry_run=True,
            max_parallel=2,
            timeout=2,
            retries=0,
            retry_delay=1.0,
            budget=None,
            continue_on_fail=False,
        ),
    )
    payload = json.loads(out.read_text())
    try:
        from pathlib import Path

        import jsonschema

        schema_path = (
            Path(__file__).parents[3]
            / "docs/sessions/20260722-agent-harness-portfolio/artifacts/benchmark_run.schema.json"
        )
        jsonschema.Draft202012Validator(json.loads(schema_path.read_text())).validate(payload)
    except ModuleNotFoundError:
        pass
    assert payload["tenant_id"] == "phenotype"
    assert payload["session_id"].startswith("ses_")
    assert payload["run_id"].startswith("run_")
    assert payload["attempt_id"].startswith("att_")
    assert payload["subject"]["harness"] == "helios-harness"
    assert payload["provenance"]["collector"] == "helios-harness"
    assert payload["signature"]["algorithm"] == "placeholder"
    assert {event["type"] for event in payload["events"]} >= {"checkpoint", "compaction"}


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as directory:
        from pathlib import Path

        test_run_runner_emits_benchmark_envelope(Path(directory))
    print("direct_envelope_test_pass")
