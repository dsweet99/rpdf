from __future__ import annotations

import builtins
import importlib
import sys
import types
from pathlib import Path

import pytest


def _load_module():
    root = Path(__file__).resolve().parent.parent
    scripts = root / "scripts"
    if str(scripts) not in sys.path:
        sys.path.insert(0, str(scripts))
    import register_bench_corpora as mod
    return mod


def test_default_bench_dir_is_not_hardcoded_user_path(monkeypatch) -> None:
    monkeypatch.delenv("PDF_PARSER_BENCHMARK_DIR", raising=False)
    mod = _load_module()
    mod = importlib.reload(mod)
    root = Path(__file__).resolve().parent.parent
    expected = {root.parent / "pdf-parser-benchmark", root / "pdf-parser-benchmark"}
    assert mod.DEFAULT_BENCH_DIR in expected


def test_default_bench_dir_honors_env(monkeypatch) -> None:
    mod = _load_module()
    monkeypatch.setenv("PDF_PARSER_BENCHMARK_DIR", "/tmp/bench-root")
    mod = importlib.reload(mod)
    assert mod._default_bench_dir() == Path("/tmp/bench-root")
    assert mod.DEFAULT_BENCH_DIR == Path("/tmp/bench-root")


def test_build_arxiv_ground_truth_handles_missing_success_key(monkeypatch, tmp_path) -> None:
    mod = _load_module()
    bench = tmp_path / "bench"
    ar5iv = bench / "corpus" / "academic" / "arxiv" / "ar5iv"
    ar5iv.mkdir(parents=True)
    (ar5iv / "1234.5678.html").write_text("<html></html>", encoding="utf-8")
    monkeypatch.setattr(mod.shutil, "which", lambda _name: "/usr/bin/pandoc")
    fake_mod = types.ModuleType("builders.ar5iv_to_markdown")

    def _convert_all_arxiv_samples(*, sample_dir, output_dir):
        assert sample_dir == ar5iv
        output_dir.mkdir(parents=True, exist_ok=True)
        return {"a": {}, "b": {"success": True}}

    fake_mod.convert_all_arxiv_samples = _convert_all_arxiv_samples
    monkeypatch.setitem(sys.modules, "builders.ar5iv_to_markdown", fake_mod)
    before = list(sys.path)
    ok, total = mod._build_arxiv_ground_truth(bench)
    assert (ok, total) == (1, 2)
    assert sys.path == before


def test_build_arxiv_ground_truth_raises_on_import_failure(monkeypatch, tmp_path) -> None:
    mod = _load_module()
    bench = tmp_path / "bench"
    ar5iv = bench / "corpus" / "academic" / "arxiv" / "ar5iv"
    ar5iv.mkdir(parents=True)
    (ar5iv / "1234.5678.html").write_text("<html></html>", encoding="utf-8")
    monkeypatch.setattr(mod.shutil, "which", lambda _name: "/usr/bin/pandoc")
    monkeypatch.delitem(sys.modules, "builders.ar5iv_to_markdown", raising=False)
    real_import = builtins.__import__

    def _fail_import(name, globals=None, locals=None, fromlist=(), level=0):
        if name == "builders.ar5iv_to_markdown":
            raise ImportError("boom")
        return real_import(name, globals, locals, fromlist, level)

    monkeypatch.setattr(builtins, "__import__", _fail_import)

    with pytest.raises(RuntimeError):
        mod._build_arxiv_ground_truth(bench)


def test_build_arxiv_ground_truth_skips_when_no_html_inputs(monkeypatch, tmp_path) -> None:
    mod = _load_module()
    bench = tmp_path / "bench"
    ar5iv = bench / "corpus" / "academic" / "arxiv" / "ar5iv"
    ar5iv.mkdir(parents=True)
    monkeypatch.setattr(mod.shutil, "which", lambda _name: "/usr/bin/pandoc")
    fake_mod = types.ModuleType("builders.ar5iv_to_markdown")
    called = {"value": False}

    def _convert_all_arxiv_samples(*, sample_dir, output_dir):
        called["value"] = True
        assert sample_dir == ar5iv
        output_dir.mkdir(parents=True, exist_ok=True)
        return {}

    fake_mod.convert_all_arxiv_samples = _convert_all_arxiv_samples
    monkeypatch.setitem(sys.modules, "builders.ar5iv_to_markdown", fake_mod)
    ok, total = mod._build_arxiv_ground_truth(bench)
    assert (ok, total) == (0, 0)
    assert called["value"] is False


def test_rewrite_all_metadata_uses_convert_helpers(monkeypatch, tmp_path) -> None:
    mod = _load_module()
    bench = tmp_path / "bench"
    calls: list[tuple[str, str]] = []

    def _fake_convert_business(path: Path, domain: str, title_prefix: str) -> int:
        calls.append((path.name, domain))
        return 1

    def _fake_convert_arxiv(path: Path) -> int:
        calls.append(("arxiv", "arxiv"))
        assert path == bench
        return 2

    monkeypatch.setattr(mod, "_convert_business", _fake_convert_business)
    monkeypatch.setattr(mod, "_convert_arxiv", _fake_convert_arxiv)
    counts = dict(mod._rewrite_all_metadata(bench))
    assert counts == {"hr": 1, "legal": 1, "invoices": 1, "arxiv": 2}
    assert calls == [
        ("hr", "hr"),
        ("legal", "legal"),
        ("invoices", "invoices"),
        ("arxiv", "arxiv"),
    ]


def test_convert_business_writes_loader_metadata(tmp_path) -> None:
    mod = _load_module()
    root = tmp_path / "corpus" / "business" / "hr"
    pdfs = root / "pdfs"
    pdfs.mkdir(parents=True)
    (pdfs / "alpha.pdf").write_bytes(b"%PDF-1.4\n")
    (pdfs / "beta.pdf").write_bytes(b"%PDF-1.4\n")
    n = mod._convert_business(root, domain="hr", title_prefix="HR document ")
    assert n == 2
    payload = (root / "metadata" / "alpha.yaml").read_text(encoding="utf-8")
    assert "doc_id: alpha" in payload
    assert "corpus: business" in payload
    assert "domain: hr" in payload
    assert "title: HR document alpha" in payload


def test_convert_arxiv_writes_loader_metadata(tmp_path) -> None:
    mod = _load_module()
    bench = tmp_path / "bench"
    pdfs = bench / "corpus" / "academic" / "arxiv" / "pdfs"
    pdfs.mkdir(parents=True)
    (pdfs / "1234.5678.pdf").write_bytes(b"%PDF-1.4\n")
    n = mod._convert_arxiv(bench)
    assert n == 1
    payload = (
        bench / "corpus" / "academic" / "arxiv" / "metadata" / "1234.5678.yaml"
    ).read_text(encoding="utf-8")
    assert "doc_id: arxiv_1234.5678" in payload
    assert "corpus: academic" in payload
    assert "domain: arxiv" in payload
    assert "title: arXiv 1234.5678" in payload


def test_rewrite_all_metadata_counts_expected_corpora(tmp_path) -> None:
    mod = _load_module()
    bench = tmp_path / "bench"
    for name in ("hr", "legal", "invoices"):
        pdfs = bench / "corpus" / "business" / name / "pdfs"
        pdfs.mkdir(parents=True, exist_ok=True)
        (pdfs / f"{name}_doc.pdf").write_bytes(b"%PDF-1.4\n")
    arxiv_pdfs = bench / "corpus" / "academic" / "arxiv" / "pdfs"
    arxiv_pdfs.mkdir(parents=True, exist_ok=True)
    (arxiv_pdfs / "9999.0001.pdf").write_bytes(b"%PDF-1.4\n")
    counts = dict(mod._rewrite_all_metadata(bench))
    assert counts == {"hr": 1, "legal": 1, "invoices": 1, "arxiv": 1}
