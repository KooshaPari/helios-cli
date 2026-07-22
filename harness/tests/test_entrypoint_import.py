from importlib import import_module


def test_harness_entrypoint_is_importable():
    module = import_module("harness.scripts.run_harness")
    assert callable(module.main)
    assert callable(module.run_runner)
