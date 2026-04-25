from pathlib import Path

import pytest

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11
    tomllib = None


def test_cargo_toml_exists() -> None:
    root = Path(__file__).resolve().parents[1]
    assert (root / "Cargo.toml").is_file()


def test_python_ops_and_packaging() -> None:
    root = Path(__file__).resolve().parents[1]
    pyproject = (root / "pyproject.toml").read_text(encoding="utf-8")
    if tomllib is not None:
        toml = tomllib.loads(pyproject)
    else:
        tomli = pytest.importorskip("tomli")
        toml = tomli.loads(pyproject)
    assert (root / "ops" / "evaluate.py").is_file()
    assert toml["project"]["scripts"]["rpdf-ops"] == "evaluate:cli"
    assert "pytest>=8.0" in toml["project"]["optional-dependencies"]["dev"]
    assert "pdf-bench" in toml["project"]["optional-dependencies"]["benchmark"]
