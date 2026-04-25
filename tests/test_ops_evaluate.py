from __future__ import annotations

import json
import os
import subprocess
import sys
import types
from pathlib import Path

import click
import pytest

ROOT = Path(__file__).resolve().parent.parent
EVAL = ROOT / "ops" / "evaluate.py"

_SUB_TIMEOUT = 30


@pytest.fixture(autouse=True)
def _restore_sys_path() -> None:
    before = list(sys.path)
    yield
    sys.path[:] = before


def _stub_eval_result(synthetic_five: list[str]) -> dict[str, object]:
    m_ok = {
        "edit_similarity": 0.99,
        "chrf++": 99.0,
        "character_error_rate": 0.0,
        "tree_similarity": 0.9,
        "element_f1": 0.9,
    }
    return {
        "parsers": ["rpdf"],
        "per_suite": {
            "synthetic": {
                "rows": [
                    {
                        "document_id": "a",
                        "parser_name": "rpdf",
                        "success": True,
                        "metrics": m_ok,
                    }
                ]
            }
        },
        "suite_computed_metrics": {"synthetic": synthetic_five},
    }


def test_evaluate_root_help() -> None:
    r = subprocess.run(
        [sys.executable, str(EVAL), "--help"],
        check=False,
        capture_output=True,
        text=True,
        timeout=_SUB_TIMEOUT,
    )
    assert r.returncode == 0


def test_evaluate_root_help_via_shebang_executable() -> None:
    r = subprocess.run(
        [str(EVAL), "--help"],
        check=False,
        capture_output=True,
        text=True,
        timeout=_SUB_TIMEOUT,
    )
    assert r.returncode == 0


def test_evaluate_all_subhelp() -> None:
    r = subprocess.run(
        [sys.executable, str(EVAL), "all", "--help"],
        check=False,
        capture_output=True,
        text=True,
        timeout=_SUB_TIMEOUT,
    )
    assert r.returncode == 0
    assert "all" in r.stdout.lower() or "bench" in r.stdout.lower()
    assert "RPDF_OPS_EXCLUDE_CLOUD" in r.stdout or "cloud" in r.stdout.lower()
    assert "--no-suite-mp" in r.stdout


def test_evaluate_rpdf_subhelp() -> None:
    r = subprocess.run(
        [sys.executable, str(EVAL), "rpdf", "--help"],
        check=False,
        capture_output=True,
        text=True,
        timeout=_SUB_TIMEOUT,
    )
    assert r.returncode == 0
    assert "rpdf" in r.stdout.lower()
    assert "--no-suite-mp" in r.stdout


def test_max_documents_must_be_positive() -> None:
    from click.testing import CliRunner

    sys.path.insert(0, str(ROOT / "ops"))
    from evaluate import cli

    r = CliRunner().invoke(cli, ["rpdf", "--max-documents", "-1"])
    assert r.exit_code != 0
    assert "Invalid value" in r.output


def test_evaluate_prefers_local_ops_path() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import evaluate

    assert sys.path[0] == str(ROOT / "ops")
    assert evaluate.__file__ is not None


def test_kpop_suites_five() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_suites import FIVE
    assert len(FIVE) == 5
    assert "edit_similarity" in FIVE


def test_resolve_parsers_excludes_cloud_for_uppercase_truthy(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = {}
    mod.PARSER_REGISTRY = {"rpdf": object(), "pdfsmith-openai": object()}
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.setenv("RPDF_OPS_EXCLUDE_CLOUD", "TRUE")
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False, all_registry=True)
    assert "pdfsmith-openai" not in out


def test_bench_list_raises_without_repo() -> None:
    import tempfile
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_suites import list_suite_rows
    with tempfile.TemporaryDirectory() as d:
        p = Path(d)
        with pytest.raises(FileNotFoundError):
            list_suite_rows(p, max_doc=1)


def _write_min_bench(p: Path) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_metric_ref import CUAD_FOUR
    m5 = [
        "edit_similarity",
        "chrf++",
        "character_error_rate",
        "tree_similarity",
        "element_f1",
    ]
    m3 = m5[:3]
    bdir = p / "benchmarks"
    bdir.mkdir(parents=True)

    def _one(rel: str, mlist: list[str]) -> None:
        lines = "\n".join(f"    - {x}" for x in mlist)
        body = f"""name: x
description: x
corpus_dir: ../corpus
document_filter:
  corpora: []
  domains: []
  features: []
  exclude_doc_ids: []
  include_doc_ids: []
  max_documents: 1
metric_config:
  metrics:
{lines}
"""
        (bdir / rel).write_text(body, encoding="utf-8")
    _one("legal_108docs.yaml", m5)
    _one("invoices_100docs.yaml", m5 + ["teds"])
    _one("hr_34docs.yaml", m5)
    _one("cuad_75docs.yaml", CUAD_FOUR)
    _one("arxiv_10docs.yaml", m3)


def test_list_suite_rows_metrics_match_benchmark_yaml() -> None:
    import tempfile
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_metric_ref import CUAD_FOUR
    from kpop_suites import FIVE, list_suite_rows
    with tempfile.TemporaryDirectory() as d:
        p = Path(d)
        _write_min_bench(p)
        rows = list_suite_rows(p, max_doc=1)
    assert {x[0] for x in rows} == {
        "synthetic",
        "legal",
        "invoices",
        "hr",
        "cuad",
        "arxiv",
    }
    by_id = {r[0]: r[4] for r in rows}
    assert by_id["synthetic"] == FIVE
    assert by_id["arxiv"] == [
        "edit_similarity",
        "chrf++",
        "character_error_rate",
    ]
    assert "teds" in by_id["invoices"] and "teds" not in by_id["cuad"]
    assert "tree_similarity" not in by_id["cuad"]
    assert by_id["cuad"] == CUAD_FOUR


def test_synthetic_suite_max_documents_is_capped_at_32() -> None:
    import tempfile

    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_suites import list_suite_rows

    with tempfile.TemporaryDirectory() as d:
        p = Path(d)
        _write_min_bench(p)
        rows = list_suite_rows(p, max_doc=1000)

    by_id = {row[0]: row for row in rows}
    assert by_id["synthetic"][3]["max_documents"] == 32


def test_evaluate_run_missing_corpus_is_click_error() -> None:
    import tempfile
    with tempfile.TemporaryDirectory() as d:
        p = Path(d)
        _write_min_bench(p)
        r = subprocess.run(
            [sys.executable, str(EVAL), "all", "--bench-dir", str(p)],
            check=False,
            capture_output=True,
            text=True,
            timeout=_SUB_TIMEOUT,
        )
    assert r.returncode != 0
    out = r.stdout + r.stderr
    assert "Traceback" not in out
    assert "Error:" in out
    assert "bench corpus missing" in out


def test_kpop_gates_accepts_passing_synthetic() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop
    from kpop_metric_ref import CUAD_FOUR, FIVE, gating_metrics_for_suite
    assert gating_metrics_for_suite("cuad") == CUAD_FOUR
    o = {
        "suite_computed_metrics": {"synthetic": FIVE},
        "per_suite": {
            "synthetic": {
                "rows": [
                    {
                        "document_id": "a",
                        "parser_name": "p",
                        "success": True,
                        "metrics": {
                            "edit_similarity": 0.99,
                            "chrf++": 99.0,
                            "character_error_rate": 0.0,
                            "tree_similarity": 0.9,
                            "element_f1": 0.9,
                        },
                    }
                ],
            }
        },
    }
    a = attach_kpop(o)
    assert a["kpop_gates"]["per_suite"]["synthetic"]["kpop_all_rows_pass"] is True


def test_kpop_has_failure_detects_failing_row() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import kpop_has_failure
    out = {
        "kpop_gates": {
            "per_suite": {
                "synthetic": {
                    "row_count": 1,
                    "kpop_all_rows_pass": False,
                }
            }
        }
    }
    assert kpop_has_failure(out) is True
    out["kpop_gates"]["per_suite"]["synthetic"]["kpop_all_rows_pass"] = True
    assert kpop_has_failure(out) is False
    out["kpop_gates"]["per_suite"]["synthetic"]["row_count"] = 0
    assert kpop_has_failure(out) is True


def _kpop_ok_metrics() -> dict[str, float]:
    return {
        "edit_similarity": 0.99,
        "chrf++": 99.0,
        "character_error_rate": 0.0,
        "tree_similarity": 0.9,
        "element_f1": 0.9,
    }


def _synthetic_suite_out(rows: list[dict[str, object]]) -> dict[str, object]:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_metric_ref import FIVE

    return {
        "suite_computed_metrics": {"synthetic": FIVE},
        "suite_expected_documents": {"synthetic": 2},
        "per_suite": {"synthetic": {"rows": rows}},
    }


def test_attach_kpop_flags_incomplete_document_coverage() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    out = _synthetic_suite_out(
        [
            {
                "document_id": "doc-a",
                "parser_name": "rpdf",
                "success": True,
                "metrics": _kpop_ok_metrics(),
            }
        ]
    )
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["kpop_documents_complete"] is False
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_attach_kpop_requires_full_docs_for_each_parser() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    ok_metrics = _kpop_ok_metrics()
    out = _synthetic_suite_out(
        [
            {
                "document_id": "doc-a",
                "parser_name": "rpdf",
                "success": True,
                "metrics": ok_metrics,
            },
            {
                "document_id": "doc-b",
                "parser_name": "rpdf",
                "success": True,
                "metrics": ok_metrics,
            },
            {
                "document_id": "doc-a",
                "parser_name": "other",
                "success": True,
                "metrics": ok_metrics,
            },
        ]
    )
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["kpop_documents_complete"] is False
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_gating_metrics_unknown_suite_defaults_to_five() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_metric_ref import FIVE, gating_metrics_for_suite
    assert gating_metrics_for_suite("not_registered_yet") == FIVE


def test_kpop_check_rejects_unknown_metric_name() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import _check
    assert _check("typo_or_future_metric", 0.99) is False


def test_go_includes_elapsed_and_schema_with_stubbed_eval(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    from kpop_metric_ref import FIVE
    b = tmp_path
    rpdf = b / "rpdf"
    rpdf.write_text("x")

    def fe(_p: object) -> dict[str, object]:
        return _stub_eval_result(FIVE)

    monkeypatch.setattr(kpop_exec, "run_eval", fe)
    from evaluate import GoIn, go

    go(
        GoIn(
            rpdf_only=False,
            bench_dir=b,
            rpdf_bin=rpdf,
            max_doc=None,
            no_suite_mp=False,
            all_registry_parsers=False,
        )
    )
    j = json.loads(capsys.readouterr().out)
    assert j["output_schema"] == "kpop_ops_eval_v1_json"
    assert "elapsed_seconds" in j
    assert isinstance(j["elapsed_seconds"], (int, float))


def test_go_disables_process_pool_with_no_suite_mp(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    from kpop_metric_ref import FIVE
    monkeypatch.setattr(click, "echo", lambda *_a, **_k: None)
    b = tmp_path
    rpdf = b / "rpdf"
    rpdf.write_text("x")
    cap: list[object] = []

    def fe(p: object) -> dict[str, object]:
        cap.append(p)
        return _stub_eval_result(FIVE)

    monkeypatch.setattr(kpop_exec, "run_eval", fe)
    from evaluate import GoIn, go

    go(
        GoIn(
            rpdf_only=False,
            bench_dir=b,
            rpdf_bin=rpdf,
            max_doc=None,
            no_suite_mp=True,
            all_registry_parsers=False,
        )
    )
    assert not cap[0].use_process_pool


def test_go_wraps_bench_failures_in_click_exception(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    b = tmp_path
    rpdf = b / "rpdf"
    rpdf.write_text("x")

    def boom(_p: object) -> None:
        msg = "simulated config failure"
        raise ValueError(msg)

    monkeypatch.setattr(kpop_exec, "run_eval", boom)
    from evaluate import GoIn, go

    with pytest.raises(click.ClickException) as excinfo:
        go(
            GoIn(
                rpdf_only=False,
                bench_dir=b,
                rpdf_bin=rpdf,
                max_doc=None,
                no_suite_mp=True,
                all_registry_parsers=False,
            )
        )
    assert "simulated config failure" in str(excinfo.value)


def test_go_wraps_attach_kpop_failures_in_click_exception(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    import kpop_gates
    from kpop_metric_ref import FIVE

    b = tmp_path
    rpdf = b / "rpdf"
    rpdf.write_text("x")

    def ok_eval(_p: object) -> dict[str, object]:
        return _stub_eval_result(FIVE)

    def boom(_o: object) -> None:
        msg = "attach failed"
        raise ValueError(msg)

    monkeypatch.setattr(kpop_exec, "run_eval", ok_eval)
    monkeypatch.setattr(kpop_gates, "attach_kpop", boom)
    from evaluate import GoIn, go

    with pytest.raises(click.ClickException) as excinfo:
        go(
            GoIn(
                rpdf_only=False,
                bench_dir=b,
                rpdf_bin=rpdf,
                max_doc=None,
                no_suite_mp=True,
                all_registry_parsers=False,
            )
        )
    assert "attach failed" in str(excinfo.value)


def test_go_reraises_unexpected_error_when_traceback_env(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    b = tmp_path
    rpdf = b / "rpdf"
    rpdf.write_text("x")
    monkeypatch.setenv("RPDF_OPS_TRACEBACK", "1")

    def boom(_p: object) -> None:
        msg = "simulated with traceback"
        raise ValueError(msg)

    monkeypatch.setattr(kpop_exec, "run_eval", boom)
    from evaluate import GoIn, go

    with pytest.raises(ValueError) as excinfo:
        go(
            GoIn(
                rpdf_only=False,
                bench_dir=b,
                rpdf_bin=rpdf,
                max_doc=None,
                no_suite_mp=True,
                all_registry_parsers=False,
            )
        )
    assert "simulated with traceback" in str(excinfo.value)


def test_write_suite_cfg_adds_bench_path_before_pdf_bench_import(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_write

    for name in ("pdf_bench", "pdf_bench.config"):
        monkeypatch.delitem(sys.modules, name, raising=False)

    def _add_bench_path(_bench: Path) -> None:
        pkg = types.ModuleType("pdf_bench")
        cfg = types.ModuleType("pdf_bench.config")

        class _P:
            def __init__(self, **kwargs: object) -> None:
                for k, v in kwargs.items():
                    setattr(self, k, v)

        def _save_config(_bcfg: object, path: Path) -> None:
            path.write_text("ok\n", encoding="utf-8")

        cfg.BenchmarkConfig = _P
        cfg.DocumentFilter = _P
        cfg.MetricConfig = _P
        cfg.OutputConfig = _P
        cfg.ParserConfig = _P
        cfg.save_config = _save_config
        pkg.config = cfg
        sys.modules["pdf_bench"] = pkg
        sys.modules["pdf_bench.config"] = cfg

    monkeypatch.setattr(kpop_write, "add_bench_path", _add_bench_path)

    cfg = {
        "bench": tmp_path,
        "tmp": tmp_path,
        "suite_id": "synthetic",
        "desc": "d",
        "filt": {
            "corpora": ["synthetic"],
            "domains": [],
            "features": [],
            "exclude_doc_ids": [],
            "include_doc_ids": [],
            "max_documents": 1,
        },
        "parsers": ["rpdf"],
        "pworkers": 0,
        "metrics": ["edit_similarity"],
    }
    out = kpop_write.write_suite_cfg(cfg)
    assert out.is_file()


def test_run_eval_allows_non_rpdf_parser_override_without_rpdf_binary(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    from evalparams import EvalParams

    bench = tmp_path / "bench"
    (bench / "corpus").mkdir(parents=True)
    missing_rpdf = tmp_path / "target" / "release" / "rpdf"

    monkeypatch.setattr(kpop_exec, "resolve_parsers", lambda *_a, **_k: ["other"])
    monkeypatch.setattr(
        kpop_exec,
        "list_suite_rows",
        lambda *_a, **_k: [
            ("synthetic", "syn", "d", {"max_documents": 1}, ["edit_similarity"])
        ],
    )
    monkeypatch.setattr(kpop_exec, "par_count", lambda *_a, **_k: 1)
    monkeypatch.setattr(kpop_exec, "inner_workers", lambda *_a, **_k: 0)
    monkeypatch.setattr(
        kpop_exec, "_build_yamls", lambda *_a, **_k: [("synthetic", "suite.yaml")]
    )
    monkeypatch.setattr(
        kpop_exec,
        "_run_dispatch",
        lambda *_a, **_k: {"synthetic": {"rows": []}},
    )

    out = kpop_exec.run_eval(
        EvalParams(
            bench=bench,
            rpdf_bin=missing_rpdf,
            rpdf_only=False,
            max_doc=None,
            use_process_pool=False,
            all_registry_parsers=True,
        )
    )
    assert out["parsers"] == ["other"]


def test_run_eval_restores_rpdf_bin_after_success(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    from evalparams import EvalParams

    bench = tmp_path / "bench"
    (bench / "corpus").mkdir(parents=True)
    rpdf = tmp_path / "target" / "release" / "rpdf"
    rpdf.parent.mkdir(parents=True, exist_ok=True)
    rpdf.write_text("x", encoding="utf-8")
    monkeypatch.setenv("RPDF_BIN", "/tmp/original-rpdf")
    monkeypatch.setattr(kpop_exec, "resolve_parsers", lambda *_a, **_k: ["rpdf"])
    monkeypatch.setattr(
        kpop_exec,
        "list_suite_rows",
        lambda *_a, **_k: [
            ("synthetic", "syn", "d", {"max_documents": 1}, ["edit_similarity"])
        ],
    )
    monkeypatch.setattr(kpop_exec, "par_count", lambda *_a, **_k: 1)
    monkeypatch.setattr(kpop_exec, "inner_workers", lambda *_a, **_k: 0)
    monkeypatch.setattr(
        kpop_exec, "_build_yamls", lambda *_a, **_k: [("synthetic", "suite.yaml")]
    )
    monkeypatch.setattr(
        kpop_exec,
        "_run_dispatch",
        lambda *_a, **_k: {"synthetic": {"rows": []}},
    )

    _ = kpop_exec.run_eval(
        EvalParams(
            bench=bench,
            rpdf_bin=rpdf,
            rpdf_only=False,
            max_doc=None,
            use_process_pool=False,
            all_registry_parsers=True,
        )
    )
    assert os.environ.get("RPDF_BIN") == "/tmp/original-rpdf"


def test_run_eval_restores_rpdf_bin_after_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    import kpop_exec
    from evalparams import EvalParams

    bench = tmp_path / "bench"
    (bench / "corpus").mkdir(parents=True)
    rpdf = tmp_path / "target" / "release" / "rpdf"
    rpdf.parent.mkdir(parents=True, exist_ok=True)
    rpdf.write_text("x", encoding="utf-8")
    monkeypatch.setenv("RPDF_BIN", "/tmp/original-rpdf")
    monkeypatch.setattr(kpop_exec, "resolve_parsers", lambda *_a, **_k: ["rpdf"])

    def _boom(*_a: object, **_k: object) -> object:
        raise RuntimeError("boom")

    monkeypatch.setattr(kpop_exec, "list_suite_rows", _boom)

    with pytest.raises(RuntimeError):
        _ = kpop_exec.run_eval(
            EvalParams(
                bench=bench,
                rpdf_bin=rpdf,
                rpdf_only=False,
                max_doc=None,
                use_process_pool=False,
                all_registry_parsers=True,
            )
        )
    assert os.environ.get("RPDF_BIN") == "/tmp/original-rpdf"
