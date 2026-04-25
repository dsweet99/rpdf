from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parent.parent
OPS = ROOT / "ops"
SCRIPTS = ROOT / "scripts"
if str(OPS) not in sys.path:
    sys.path.insert(0, str(OPS))
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))


@pytest.fixture(autouse=True)
def _restore_sys_path() -> None:
    before = list(sys.path)
    yield
    sys.path[:] = before


def test_evalparams_symbol_coverage() -> None:
    import evalparams

    assert evalparams.EvalParams.__name__ == "EvalParams"


def test_evaluate_symbol_coverage() -> None:
    import evaluate

    assert callable(evaluate.cli)
    assert callable(evaluate.all_cmd)
    assert callable(evaluate.rpdf_cmd)


def test_kpop_exec_symbol_coverage() -> None:
    import kpop_exec

    assert callable(kpop_exec.run_eval)


def test_kpop_loader_symbol_coverage() -> None:
    import kpop_loader

    assert callable(kpop_loader.add_bench_path)
    assert callable(kpop_loader.resolve_parsers)
    assert callable(kpop_loader.inner_workers)
    assert callable(kpop_loader.par_count)
    assert callable(kpop_loader.run_from_pack)


def test_kpop_metric_ref_symbol_coverage() -> None:
    import kpop_metric_ref

    assert callable(kpop_metric_ref.applicable_gates)


def test_kpop_serialize_symbol_coverage() -> None:
    import kpop_serialize

    assert callable(kpop_serialize.serialize_bench)
    assert callable(kpop_serialize.to_stdout)


def test_kpop_write_symbol_coverage() -> None:
    import kpop_write

    assert callable(kpop_write.write_suite_cfg)


def test_register_bench_corpora_symbol_coverage() -> None:
    import register_bench_corpora

    assert callable(register_bench_corpora.main)
