"""Repository-root compatibility package for the installable harness."""

from pathlib import Path


_SOURCE_PACKAGE = Path(__file__).resolve().parent / "src" / "harness"
if _SOURCE_PACKAGE.is_dir() and str(_SOURCE_PACKAGE) not in __path__:
    __path__.append(str(_SOURCE_PACKAGE))
