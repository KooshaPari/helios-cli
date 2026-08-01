# Helios Harness Benchmarks

from pathlib import Path


# Keep the legacy benchmark package importable from the repository root while
# exposing the installable ``src/harness`` package to in-repo entrypoint users.
_SOURCE_PACKAGE = Path(__file__).resolve().parent / "src" / "harness"
if _SOURCE_PACKAGE.is_dir() and str(_SOURCE_PACKAGE) not in __path__:
    __path__.append(str(_SOURCE_PACKAGE))
