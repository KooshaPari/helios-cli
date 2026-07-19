"""Configuration source-provenance regressions."""

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
