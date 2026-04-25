from __future__ import annotations

from pathlib import Path
from typing import Any

from kpop_loader import add_bench_path


def _parsers_cfg(src: list[str], pcls: type) -> Any:
    return pcls(parsers=src, parser_options={})


def _mk_doc_filter(df: dict[str, Any], dfx: type) -> Any:
    return dfx(
        corpora=list(df.get("corpora", [])),
        domains=list(df.get("domains", [])),
        features=list(df.get("features", [])),
        exclude_doc_ids=list(df.get("exclude_doc_ids", [])),
        include_doc_ids=list(df.get("include_doc_ids", [])),
        max_documents=df.get("max_documents"),
    )


def _mk_out(rdir: Path, ocls: type) -> Any:
    return ocls(
        results_dir=str(rdir),
        save_format="json",
        generate_report=False,
        report_path=None,
        save_predictions=False,
        verbose=False,
    )


def _bcfg(k: dict[str, Any], bcl: type) -> Any:
    c: dict = k["c"]
    return bcl(
        name=f"kpop-ops-{c['suite_id']}",
        description=c["desc"],
        corpus_dir=str(k["corpus"]),
        document_filter=k["fdf"],
        parser_config=k["pcf"],
        metric_config=k["mfc"],
        output_config=k["ocf"],
        parallel_workers=c["pworkers"],
    )


def write_suite_cfg(c: dict[str, Any]) -> Path:
    b = c["bench"]
    add_bench_path(b)
    from pdf_bench.config import (
        BenchmarkConfig,
        DocumentFilter,
        MetricConfig,
        OutputConfig,
        ParserConfig,
        save_config,
    )
    corpus = (b / "corpus").resolve()
    rdir = (c["tmp"] / f"r_{c['suite_id']}").resolve()
    rdir.mkdir(parents=True, exist_ok=True)
    fdf = _mk_doc_filter(c["filt"], DocumentFilter)
    ocf = _mk_out(rdir, OutputConfig)
    pcf = _parsers_cfg(c["parsers"], ParserConfig)
    mfc = MetricConfig(metrics=c["metrics"], aggregate=True)
    key = {
        "c": c,
        "corpus": corpus,
        "fdf": fdf,
        "ocf": ocf,
        "pcf": pcf,
        "mfc": mfc,
    }
    bcfg = _bcfg(key, BenchmarkConfig)
    pth = (c["tmp"] / f"cfg_{c['suite_id']}.yaml").resolve()
    save_config(bcfg, pth)
    return pth
