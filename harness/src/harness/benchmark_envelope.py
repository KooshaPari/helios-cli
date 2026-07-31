"""Deterministic umbrella evidence envelope for harness runs."""

from __future__ import annotations

import hashlib
import json
import re
from datetime import UTC, datetime


def _digest(value: object) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


_GIT_SHA_PATTERN = re.compile(r"[0-9a-f]{7,64}")


def _require_resolved_git_sha(subject_commit: str) -> str:
    if not _GIT_SHA_PATTERN.fullmatch(subject_commit):
        raise ValueError("benchmark envelope requires a resolved Git SHA for the subject commit")
    return subject_commit


def add_envelope(
    payload: dict,
    *,
    repo: str,
    profile: str,
    plan_hash: str,
    subject_commit: str,
) -> dict:
    subject_commit = _require_resolved_git_sha(subject_commit)
    identity = {"repo": repo, "commit": subject_commit, "harness": "helios-harness", "model": "unknown", "task_id": f"plan:{plan_hash}", "hardware": "unknown"}
    digest = _digest(identity)
    run_id = f"run_{digest}"
    session_id = f"ses_{_digest({'run_id': run_id, 'session': 'default'})}"
    attempt_id = f"att_{_digest({'run_id': run_id, 'attempt': 0})}"
    causality = {"tenant_id": "phenotype", "session_id": session_id, "run_id": run_id, "attempt_id": attempt_id}
    now = datetime.now(UTC).isoformat()
    checkpoint_id = f"cp_{_digest({'run_id': run_id, 'checkpoint': 0})[:16]}"
    events = []
    for seq, event_type in enumerate(("run_started", "checkpoint", "compaction", "run_finished")):
        event = {"event_id": f"evt_{_digest({'run_id': run_id, 'seq': seq, 'type': event_type})}", "seq": seq, "ts": now, "type": event_type, "payload_sha256": _digest({"type": event_type, "seq": seq}), "causality": causality}
        if event_type in ("checkpoint", "compaction"):
            event["checkpoint_id"] = checkpoint_id
        if event_type == "compaction":
            event["details"] = {"tokens_before": 0, "tokens_after": 0, "retained_event_ids": [], "dropped_event_ids": []}
        events.append(event)
    passed = payload.get("result_code") == "PASS"
    legacy_digest = _digest(payload)
    commands = payload.get("commands", payload.get("plan", []))
    raw_runs = payload.get("runs", [])
    tasks = [
        {
            "task_id": f"task_{_digest({'plan': plan_hash, 'index': index, 'command': command.get('command', '')})}",
            "command": command.get("command", ""),
            "bucket": command.get("bucket", "runtime"),
            "required": bool(command.get("required", True)),
            "cwd": command.get("cwd", repo),
            "source": command.get("source", ""),
        }
        for index, command in enumerate(commands)
        if isinstance(command, dict)
    ]
    plan_commands = [
        {
            **command,
            "task_id": tasks[index]["task_id"],
        }
        for index, command in enumerate(commands)
        if isinstance(command, dict)
    ]
    runs = [
        {
            "run_id": f"taskrun_{_digest({'run': run, 'index': index})}",
            "task_id": tasks[index]["task_id"] if index < len(tasks) else f"task_{_digest({'plan': plan_hash, 'index': index})}",
            "command": run.get("command", ""),
            "status": "passed" if run.get("returncode") in (0, None) and not run.get("timed_out") else ("timeout" if run.get("timed_out") else "failed"),
            "returncode": run.get("returncode"),
            "duration_ms": run.get("duration_ms", 0),
        }
        for index, run in enumerate(raw_runs)
        if isinstance(run, dict)
    ]
    return {
        "plan": plan_commands,
        "commands": plan_commands,
        "plan_hash": plan_hash,
        "schema_version": "1.0.0",
        "tenant_id": "phenotype", "session_id": session_id, "run_id": run_id, "attempt_id": attempt_id,
        "deterministic_identity": {"algorithm": "sha256(canonical-json(inputs))", "canonical_json_sha256": digest, "inputs": identity},
        "subject": {"repo": repo, "commit": subject_commit, "harness": "helios-harness", "runtime": "python", "model": "unknown", "hardware": "unknown"},
        "lease": {"lease_id": f"lease_{attempt_id[4:20]}", "owner": "helios-harness", "ttl_seconds": 120, "heartbeat_interval_seconds": 20},
        "task_manifest": {"task_id": f"plan:{plan_hash}", "input_sha256": plan_hash, "timeout_seconds": 1, "assertions": [{"id": "plan_discovered", "kind": "command_plan", "expected": True}], "judge": {"name": "helios-harness", "version": "0.1.0"}},
        "tasks": tasks,
        "runs": runs,
        "events": events,
        "result": {"status": "passed" if passed else "failed", "outcome_sha256": legacy_digest, "replay_hash": _digest(events), "failure_class": "none" if passed else "unknown", "artifacts": [{"kind": "report", "uri": f"urn:helios:legacy-evidence:{legacy_digest}", "sha256": legacy_digest}]},
        "provenance": {"collector": "helios-harness", "collected_at": now, "source_hashes": {"plan": plan_hash}},
        "signature": {"algorithm": "placeholder", "key_id": "unconfigured", "signature_b64": ""},
        "result_code": "PASS" if passed else "FAIL",
    }
