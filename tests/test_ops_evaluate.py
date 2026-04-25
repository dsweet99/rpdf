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
        "suite_expected_documents": {"synthetic": 1},
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
    assert "local" in r.stdout.lower() or "ollama" in r.stdout.lower()
    assert "RPDF_OPS_PARSERS" in r.stdout
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
    mod.PARSER_REGISTRY = {
        "rpdf": object(),
        "pdfsmith-openai": object(),
        "landing.ai": object(),
    }
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.setenv("RPDF_OPS_EXCLUDE_CLOUD", "TRUE")
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "pdfsmith-openai" not in out
    assert "landing.ai" not in out


def test_resolve_parsers_excludes_landing_registry_ids(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = {}
    mod.PARSER_REGISTRY = {"rpdf": object(), "landing.ai": object(), "other": object()}
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.delenv("RPDF_OPS_EXCLUDE_CLOUD", raising=False)
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "landing.ai" not in out
    assert out == ["other", "rpdf"]


def test_resolve_parsers_filters_landing_ollama_and_variants_in_override(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    monkeypatch.setenv(
        "RPDF_OPS_PARSERS",
        "landing.ai,landing_ai,landing-ai,ollama,marker_ollama,pdfsmith-ollama,pdfsmithollama,rpdf,other",
    )

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "landing.ai" not in out
    assert "landing_ai" not in out
    assert "landing-ai" not in out
    assert "ollama" not in out
    assert "marker_ollama" not in out
    assert "pdfsmith-ollama" not in out
    assert "pdfsmithollama" not in out
    assert out == ["rpdf", "other"]


def _resolve_with_override_aliases(
    monkeypatch: pytest.MonkeyPatch, aliases: dict[str, str], override: str
) -> list[str]:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = aliases
    mod.PARSER_REGISTRY = {}
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.setenv("RPDF_OPS_PARSERS", override)
    return resolve_parsers(Path("."), rpdf_only=False)


@pytest.mark.parametrize(
    ("aliases", "override", "expected"),
    [
        (
            {"fast_local": "pdfsmith-ollama", "safe_alias": "rpdf"},
            "fast_local,safe_alias,other",
            ["safe_alias", "other"],
        ),
        (
            {"fast_local": "alias_mid", "alias_mid": "pdfsmith-ollama"},
            "fast_local,other",
            ["other"],
        ),
        (
            {"FAST_LOCAL": "pdfsmith-ollama", "SAFE_ALIAS": "rpdf"},
            "fast_local,safe_alias,other",
            ["safe_alias", "other"],
        ),
        (
            {},
            "rpdf,rpdf,other,other",
            ["rpdf", "other"],
        ),
    ],
)
def test_resolve_parsers_filters_override_aliases_to_ollama(
    monkeypatch: pytest.MonkeyPatch,
    aliases: dict[str, str],
    override: str,
    expected: list[str],
) -> None:
    out = _resolve_with_override_aliases(monkeypatch, aliases=aliases, override=override)
    assert out == expected


def test_resolve_parsers_removes_ollama_from_registry(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = {}
    mod.PARSER_REGISTRY = {
        "rpdf": object(),
        "ollama": object(),
        "marker_ollama": object(),
        "pdfsmith-ollama": object(),
        "pdfsmithollama": object(),
        "other": object(),
    }
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.delenv("RPDF_OPS_EXCLUDE_CLOUD", raising=False)
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "ollama" not in out
    assert "marker_ollama" not in out
    assert "pdfsmith-ollama" not in out
    assert "pdfsmithollama" not in out
    assert out == ["other", "rpdf"]


def test_resolve_parsers_does_not_swallow_loader_runtime_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")

    def _boom(_name: str) -> object:
        raise RuntimeError("loader exploded")

    mod.__getattr__ = _boom  # type: ignore[attr-defined]
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.setenv("RPDF_OPS_PARSERS", "rpdf")

    with pytest.raises(RuntimeError, match="loader exploded"):
        _ = resolve_parsers(Path("."), rpdf_only=False)


def test_resolve_parsers_keeps_non_cloud_parser_with_aws_substring(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = {}
    mod.PARSER_REGISTRY = {
        "rpdf": object(),
        "laws-parser": object(),
        "local": object(),
    }
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.delenv("RPDF_OPS_EXCLUDE_CLOUD", raising=False)
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "laws-parser" in out


def test_resolve_parsers_excludes_pdfsmith_gemini_variants_when_cloud_filtered(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = {}
    mod.PARSER_REGISTRY = {
        "rpdf": object(),
        "pdfsmith-gemini": object(),
        "pdfsmith_gemini": object(),
        "other": object(),
    }
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.setenv("RPDF_OPS_EXCLUDE_CLOUD", "TRUE")
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "pdfsmith-gemini" not in out
    assert "pdfsmith_gemini" not in out
    assert out == ["other", "rpdf"]


def test_resolve_parsers_excludes_pdfsmith_openai_concat_when_cloud_filtered(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import resolve_parsers

    pkg = types.ModuleType("pdf_bench")
    mod = types.ModuleType("pdf_bench.loader")
    mod.PARSER_ALIASES = {}
    mod.PARSER_REGISTRY = {
        "rpdf": object(),
        "pdfsmithopenai": object(),
        "pdfsmithazure": object(),
        "other": object(),
    }
    pkg.loader = mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", mod)
    monkeypatch.setenv("RPDF_OPS_EXCLUDE_CLOUD", "TRUE")
    monkeypatch.delenv("RPDF_OPS_PARSERS", raising=False)

    out = resolve_parsers(Path("."), rpdf_only=False)
    assert "pdfsmithopenai" not in out
    assert "pdfsmithazure" not in out
    assert out == ["other", "rpdf"]


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
        "suite_expected_documents": {"synthetic": 1},
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


def _synthetic_edit_similarity_out(
    rows: list[dict[str, object]], expected_documents: int
) -> dict[str, object]:
    return {
        "suite_computed_metrics": {"synthetic": ["edit_similarity"]},
        "suite_expected_documents": {"synthetic": expected_documents},
        "per_suite": {"synthetic": {"rows": rows}},
    }


def _ok_row(doc_id: object, parser_name: object, metrics: dict[str, float]) -> dict[str, object]:
    return {
        "document_id": doc_id,
        "parser_name": parser_name,
        "success": True,
        "metrics": metrics,
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


def test_attach_kpop_requires_shared_docs_across_parsers() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    ok_metrics = _kpop_ok_metrics()
    out = _synthetic_edit_similarity_out(
        [
            _ok_row("doc-a", "rpdf", ok_metrics),
            _ok_row("doc-b", "rpdf", ok_metrics),
            _ok_row("doc-c", "other", ok_metrics),
            _ok_row("doc-d", "other", ok_metrics),
        ],
        expected_documents=2,
    )
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["parser_document_counts"] == {"other": 2, "rpdf": 2}
    assert per["shared_document_count"] == 0
    assert per["kpop_documents_complete"] is False
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_attach_kpop_counts_numeric_document_ids() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    ok_metrics = _kpop_ok_metrics()
    out = _synthetic_edit_similarity_out(
        [
            _ok_row(1, "rpdf", ok_metrics),
            _ok_row(2, "rpdf", ok_metrics),
            _ok_row(1, "other", ok_metrics),
            _ok_row(2, "other", ok_metrics),
        ],
        expected_documents=2,
    )
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["document_count"] == 2
    assert per["parser_document_counts"] == {"other": 2, "rpdf": 2}
    assert per["kpop_documents_complete"] is True
    assert per["kpop_all_rows_pass"] is True
    assert kpop_has_failure(gated) is False


def test_attach_kpop_requires_expected_document_metadata() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    ok_metrics = _kpop_ok_metrics()
    out = {
        "suite_computed_metrics": {"synthetic": ["edit_similarity"]},
        "per_suite": {
            "synthetic": {
                "rows": [
                    _ok_row("doc-a", "rpdf", ok_metrics),
                    _ok_row("doc-a", "other", ok_metrics),
                ]
            }
        },
    }
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["expected_documents"] is None
    assert per["kpop_documents_complete"] is False
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_attach_kpop_normalizes_integral_float_document_ids() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop

    ok_metrics = _kpop_ok_metrics()
    out = _synthetic_edit_similarity_out(
        [
            _ok_row(1, "rpdf", ok_metrics),
            _ok_row(1.0, "rpdf", ok_metrics),
        ],
        expected_documents=1,
    )
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["document_count"] == 1
    assert per["parser_document_counts"] == {"rpdf": 1}
    assert per["kpop_documents_complete"] is True
    assert per["kpop_all_rows_pass"] is True


def test_attach_kpop_requires_valid_parser_names_for_coverage() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    ok_metrics = _kpop_ok_metrics()
    out = {
        "suite_computed_metrics": {"synthetic": ["edit_similarity"]},
        "suite_expected_documents": {"synthetic": 2},
        "per_suite": {
            "synthetic": {
                "rows": [
                    {
                        "document_id": "doc-a",
                        "parser_name": None,
                        "success": True,
                        "metrics": ok_metrics,
                    },
                    {
                        "document_id": "doc-b",
                        "parser_name": None,
                        "success": True,
                        "metrics": ok_metrics,
                    },
                ]
            }
        },
    }
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["parser_document_counts"] == {}
    assert per["kpop_documents_complete"] is False
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_attach_kpop_rejects_whitespace_parser_names_for_coverage() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    ok_metrics = _kpop_ok_metrics()
    out = _synthetic_edit_similarity_out(
        [
            _ok_row("doc-a", "   ", ok_metrics),
            _ok_row("doc-b", "   ", ok_metrics),
        ],
        expected_documents=2,
    )
    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["parser_document_counts"] == {}
    assert per["kpop_documents_complete"] is False
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_attach_kpop_fails_when_no_metrics_were_gated() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    out = {
        "suite_computed_metrics": {"synthetic": []},
        "suite_expected_documents": {"synthetic": 1},
        "per_suite": {
            "synthetic": {
                "rows": [
                    {
                        "document_id": "doc-1",
                        "parser_name": "rpdf",
                        "success": True,
                        "metrics": {},
                    }
                ]
            }
        },
    }

    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["gated_metrics"] == []
    assert per["kpop_all_rows_pass"] is False
    assert kpop_has_failure(gated) is True


def test_attach_kpop_rejects_boolean_metric_values() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    out = {
        "suite_computed_metrics": {"synthetic": ["edit_similarity"]},
        "suite_expected_documents": {"synthetic": 1},
        "per_suite": {
            "synthetic": {
                "rows": [
                    {
                        "document_id": "doc-1",
                        "parser_name": "rpdf",
                        "success": True,
                        "metrics": {"edit_similarity": True},
                    }
                ]
            }
        },
    }

    gated = attach_kpop(out)
    per = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert per["kpop_all_rows_pass"] is False
    assert per["by_row"][0]["kpop_failed_gates"] == ["edit_similarity"]
    assert kpop_has_failure(gated) is True


def test_attach_kpop_fails_when_expected_suite_missing_from_results() -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_gates import attach_kpop, kpop_has_failure

    out = {
        "suite_computed_metrics": {"synthetic": ["edit_similarity"]},
        "suite_expected_documents": {"synthetic": 1},
        "per_suite": {},
    }
    gated = attach_kpop(out)
    synthetic = gated["kpop_gates"]["per_suite"]["synthetic"]
    assert synthetic["row_count"] == 0
    assert synthetic["kpop_all_rows_pass"] is False
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
        )
    )
    assert out["parsers"] == ["other"]


def _mock_run_from_pack_deps(
    monkeypatch: pytest.MonkeyPatch, should_raise: bool
) -> None:
    loader_mod = types.ModuleType("pdf_bench.loader")
    runner_mod = types.ModuleType("pdf_bench.runner")
    serialize_mod = types.ModuleType("kpop_serialize")
    pkg = types.ModuleType("pdf_bench")

    def _load_cfg(_path: Path) -> dict[str, object]:
        return {"ok": True}

    class _Runner:
        def __init__(self, _cfg: object, parallel_workers: int) -> None:
            assert parallel_workers == 0

        def run(self) -> dict[str, object]:
            if should_raise:
                raise RuntimeError("boom")
            return {"rows": []}

    def _serialize(_bres: object) -> dict[str, object]:
        return {"per_suite": {}}

    loader_mod.load_benchmark_config = _load_cfg
    runner_mod.BenchmarkRunner = _Runner
    serialize_mod.serialize_bench = _serialize
    pkg.loader = loader_mod
    pkg.runner = runner_mod
    monkeypatch.setitem(sys.modules, "pdf_bench", pkg)
    monkeypatch.setitem(sys.modules, "pdf_bench.loader", loader_mod)
    monkeypatch.setitem(sys.modules, "pdf_bench.runner", runner_mod)
    monkeypatch.setitem(sys.modules, "kpop_serialize", serialize_mod)


def test_run_from_pack_restores_rpdf_bin_after_success(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import run_from_pack

    yaml_path = tmp_path / "suite.yaml"
    yaml_path.write_text("name: x\n", encoding="utf-8")
    monkeypatch.setenv("RPDF_BIN", "/tmp/original-rpdf")
    _mock_run_from_pack_deps(monkeypatch, should_raise=False)

    out = run_from_pack((str(yaml_path), str(tmp_path), "/tmp/new-rpdf", 0))
    assert "elapsed_seconds" in out
    assert os.environ.get("RPDF_BIN") == "/tmp/original-rpdf"


def test_run_from_pack_restores_rpdf_bin_after_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    sys.path.insert(0, str(ROOT / "ops"))
    from kpop_loader import run_from_pack

    yaml_path = tmp_path / "suite.yaml"
    yaml_path.write_text("name: x\n", encoding="utf-8")
    monkeypatch.setenv("RPDF_BIN", "/tmp/original-rpdf")
    _mock_run_from_pack_deps(monkeypatch, should_raise=True)

    with pytest.raises(RuntimeError, match="boom"):
        _ = run_from_pack((str(yaml_path), str(tmp_path), "/tmp/new-rpdf", 0))
    assert os.environ.get("RPDF_BIN") == "/tmp/original-rpdf"


def _stub_single_suite_run(
    monkeypatch: pytest.MonkeyPatch, kpop_exec: object, parsers: list[str]
) -> None:
    monkeypatch.setattr(kpop_exec, "resolve_parsers", lambda *_a, **_k: parsers)
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


def test_run_eval_errors_when_override_filters_to_zero_parsers(
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
    _stub_single_suite_run(monkeypatch, kpop_exec, [])

    with pytest.raises(ValueError, match="no parsers selected"):
        _ = kpop_exec.run_eval(
            EvalParams(
                bench=bench,
                rpdf_bin=rpdf,
                rpdf_only=False,
                max_doc=None,
                use_process_pool=False,
            )
        )


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
    _stub_single_suite_run(monkeypatch, kpop_exec, ["rpdf"])

    _ = kpop_exec.run_eval(
        EvalParams(
            bench=bench,
            rpdf_bin=rpdf,
            rpdf_only=False,
            max_doc=None,
            use_process_pool=False,
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
            )
        )
    assert os.environ.get("RPDF_BIN") == "/tmp/original-rpdf"
