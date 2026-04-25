from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

import yaml
from kpop_metric_ref import FIVE

_LEGAL = "benchmarks/legal_108docs.yaml"
_INVO = "benchmarks/invoices_100docs.yaml"
_HR = "benchmarks/hr_34docs.yaml"
_CUAD = "benchmarks/cuad_75docs.yaml"
_ARX = "benchmarks/arxiv_10docs.yaml"


def _load_yaml(bench: Path, rel: str) -> dict[str, Any]:
    p = bench / rel
    if not p.is_file():
        msg = f"missing {p}"
        raise FileNotFoundError(msg)
    with open(p, encoding="utf-8") as f:
        return yaml.safe_load(f)


def _metrics_from_suite_yaml(y: dict[str, Any], rel: str) -> list[str]:
    mcfg = y.get("metric_config")
    if not isinstance(mcfg, dict):
        msg = f"metric_config missing or invalid in {rel}"
        raise KeyError(msg)
    mets = mcfg.get("metrics")
    if not isinstance(mets, list) or not mets:
        msg = f"metric_config.metrics missing or empty in {rel}"
        raise KeyError(msg)
    return [str(x) for x in mets]


def cap(filt: dict[str, Any], max_doc: int | None) -> dict[str, Any]:
    o = copy.deepcopy(filt)
    if max_doc is not None:
        if max_doc <= 0:
            msg = "max_documents must be positive"
            raise ValueError(msg)
        o["max_documents"] = max_doc
    return o


def list_suite_rows(
    bench: Path, max_doc: int | None
) -> list[tuple[str, str, str, dict[str, Any], list[str]]]:
    if not bench.is_dir():
        msg = f"bench dir not found: {bench}"
        raise FileNotFoundError(msg)
    if max_doc is not None and max_doc <= 0:
        msg = "max_documents must be positive"
        raise ValueError(msg)
    syn: dict[str, Any] = {
        "corpora": ["synthetic"],
        "domains": [],
        "features": [],
        "exclude_doc_ids": [],
        "include_doc_ids": [],
        "max_documents": 32,
    }
    if max_doc is not None:
        syn["max_documents"] = min(max_doc, 32)
    rows: list[tuple[str, str, str, dict[str, Any], list[str]]] = [
        (
            "synthetic",
            "synthetic (kpop cap 32)",
            "kpop synthetic corpus, max 32 per kpop_prompt",
            cap(syn, None),
            list(FIVE),
        ),
    ]
    for sid, t, rel in (
        (
            "legal",
            "legal templates",
            _LEGAL,
        ),
        (
            "invoices",
            "invoices",
            _INVO,
        ),
        (
            "hr",
            "hr / resumes",
            _HR,
        ),
        (
            "cuad",
            "CUAD contracts",
            _CUAD,
        ),
        (
            "arxiv",
            "academic / arXiv",
            _ARX,
        ),
    ):
        y = _load_yaml(bench, rel)
        f = copy.deepcopy(y["document_filter"])
        mets = _metrics_from_suite_yaml(y, rel)
        rows.append(
            (sid, t, "from kpop_prompt + benchmark " + rel, cap(f, max_doc), mets),
        )
    return rows
