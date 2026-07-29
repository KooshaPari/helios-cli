"""Configuration source-provenance and file-shape regressions."""

import json

import pytest

from harness.config import ConfigManager, ConfigSource


def test_load_file_marks_each_loaded_key_as_file_source(tmp_path) -> None:
    config_path = tmp_path / "config.json"
    config_path.write_text('{"endpoint": "https://file.example", "retries": 3}')

    config = ConfigManager(base_path=str(tmp_path)).load_file("config.json", source="json")

    assert config.get("endpoint") == "https://file.example"
    assert config.get("retries") == 3
    assert config.source_of("endpoint") is ConfigSource.FILE
    assert config.source_of("retries") is ConfigSource.FILE


def test_later_env_and_manual_values_replace_file_provenance(tmp_path, monkeypatch) -> None:
    config_path = tmp_path / "config.json"
    config_path.write_text('{"endpoint": "https://file.example", "retries": 3}')
    monkeypatch.setenv("HELIOS_ENDPOINT", "https://env.example")

    config = ConfigManager(base_path=str(tmp_path)).load_file("config.json", source="json")
    config.load_env_prefix()
    config.set("retries", 4)

    assert config.get("endpoint") == "https://env.example"
    assert config.source_of("endpoint") is ConfigSource.ENV
    assert config.get("retries") == 4
    assert config.source_of("retries") is ConfigSource.DEFAULT


@pytest.mark.parametrize(
    ("filename", "contents", "source"),
    [
        ("empty.yaml", "", "yaml"),
        ("null.json", "null", "json"),
        ("array.json", "[\"not\", \"a mapping\"]", "json"),
        ("scalar.json", "3", "json"),
        ("pairs.yaml", "- [key, value]", "yaml"),
    ],
)
def test_load_file_rejects_non_mapping_documents_without_mutation(
    tmp_path, filename, contents, source
) -> None:
    (tmp_path / filename).write_text(contents)
    config = ConfigManager(base_path=str(tmp_path))
    config.set("preserved", "value", source=ConfigSource.ENV)

    with pytest.raises(ValueError, match=filename):
        config.load_file(filename, source=source)

    assert config.to_dict() == {"preserved": "value"}
    assert config.source_of("preserved") is ConfigSource.ENV


@pytest.mark.parametrize(
    ("filename", "contents", "source"),
    [
        ("empty-mapping.yaml", "{}", "yaml"),
        ("empty-mapping.json", "{}", "json"),
    ],
)
def test_load_file_accepts_empty_mapping_as_a_noop(tmp_path, filename, contents, source) -> None:
    (tmp_path / filename).write_text(contents)
    config = ConfigManager(base_path=str(tmp_path))
    config.set("preserved", "value", source=ConfigSource.ENV)

    assert config.load_file(filename, source=source) is config
    assert config.to_dict() == {"preserved": "value"}
    assert config.source_of("preserved") is ConfigSource.ENV


def test_load_file_keeps_json_parse_errors_unchanged(tmp_path) -> None:
    filename = "malformed.json"
    (tmp_path / filename).write_text("{")
    config = ConfigManager(base_path=str(tmp_path))

    with pytest.raises(json.JSONDecodeError):
        config.load_file(filename, source="json")

    assert config.to_dict() == {}
