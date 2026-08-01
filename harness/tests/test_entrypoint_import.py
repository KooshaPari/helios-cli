from importlib import import_module
from pathlib import Path
import tomllib


def test_harness_entrypoint_is_importable():
    module = import_module("harness.scripts.run_harness")
    assert callable(module.main)
    assert callable(module.run_runner)


def test_harness_declares_schema_validation_dependencies():
    # Traces to: FR-HELIOS-SCHEMA-001 (schema validation dependencies).
    project = tomllib.loads(
        (Path(__file__).resolve().parents[1] / "pyproject.toml").read_text()
    )
    dependencies = set(project["project"]["dependencies"])
    assert any(dependency.startswith("jsonschema") for dependency in dependencies)
    assert any(dependency.startswith("fastjsonschema") for dependency in dependencies)
