from __future__ import annotations

import math
from typing import Any

from kpop_metric_ref import applicable_gates, gating_metrics_for_suite


def _check(n: str, v: float | None) -> bool:
    if v is None or (isinstance(v, float) and math.isnan(v)):
        return False
    if n == "edit_similarity":
        return v >= 0.95
    if n == "chrf++":
        return v >= 95.0
    if n == "character_error_rate":
        return v <= 0.05
    if n == "tree_similarity":
        return v >= 0.80
    return n == "element_f1" and v >= 0.80


def _row_fl(
    metrics: dict[str, float | None], need: list[str], ok: bool
) -> tuple[bool, list[str]]:
    if not ok:
        return False, ["parse_error"]
    if not need:
        return True, []
    bad: list[str] = []
    for n in need:
        v = metrics.get(n)
        if not _check(n, v):
            bad.append(n)
    return (len(bad) == 0, bad)


def _normalize_row(row: Any) -> tuple[dict[str, Any], bool, dict[str, Any]]:
    if not isinstance(row, dict):
        return ({}, False, {"document_id": None, "parser_name": None})
    m = row.get("metrics", {})
    if not isinstance(m, dict):
        m = {}
    return (
        m,
        row.get("success") is True,
        {
            "document_id": row.get("document_id"),
            "parser_name": row.get("parser_name"),
        },
    )


def _count_documents(rows: list[dict[str, Any]]) -> int:
    docs = {
        str(x["document_id"])
        for x in rows
        if "document_id" in x and isinstance(x["document_id"], str) and x["document_id"]
    }
    return len(docs)


def _count_documents_for_parser(rows: list[dict[str, Any]], parser: str) -> int:
    docs = {
        str(x["document_id"])
        for x in rows
        if x.get("parser_name") == parser
        and "document_id" in x
        and isinstance(x["document_id"], str)
        and x["document_id"]
    }
    return len(docs)


def _build_suite_rows(block: dict[str, Any], gmed: list[str]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for row in block.get("rows", []):
        m, success, head = _normalize_row(row)
        passed, fl = _row_fl(m, gmed, success)
        out.append(
            {
                "document_id": head["document_id"],
                "parser_name": head["parser_name"],
                "kpop_pass": passed,
                "kpop_failed_gates": fl,
            }
        )
    return out


def _parser_document_counts(rows: list[dict[str, Any]]) -> dict[str, int]:
    parser_names = {
        str(x["parser_name"])
        for x in rows
        if isinstance(x.get("parser_name"), str) and x["parser_name"]
    }
    return {p: _count_documents_for_parser(rows, p) for p in sorted(parser_names)}


def _suite_kpop(
    suite_id: str, block: Any, computed_for_suite: Any
) -> dict[str, Any]:
    clist = computed_for_suite
    if not isinstance(clist, list):
        clist = []
    gmed = applicable_gates(suite_id, [str(x) for x in clist])
    if not isinstance(block, dict):
        block = {}
    exp = block.get("_expected_documents")
    expected_documents = exp if isinstance(exp, int) and exp > 0 else None
    rows = _build_suite_rows(block, gmed)
    pass_count = sum(1 for x in rows if x["kpop_pass"])
    document_count = _count_documents(rows)
    per_parser_doc_counts = _parser_document_counts(rows)
    if expected_documents is None:
        documents_complete = True
    elif per_parser_doc_counts:
        documents_complete = all(
            v >= expected_documents for v in per_parser_doc_counts.values()
        )
    else:
        documents_complete = document_count >= expected_documents
    all_rows_pass = bool(rows) and pass_count == len(rows) and documents_complete
    return {
        "gated_metrics": gmed,
        "kpop_reference_metrics": gating_metrics_for_suite(suite_id),
        "row_count": len(rows),
        "document_count": document_count,
        "parser_document_counts": per_parser_doc_counts,
        "expected_documents": expected_documents,
        "kpop_documents_complete": documents_complete,
        "kpop_pass_count": pass_count,
        "kpop_all_rows_pass": all_rows_pass,
        "by_row": rows,
    }


def attach_kpop(out: dict[str, Any]) -> dict[str, Any]:
    ps = out.get("per_suite", {})
    scm = out.get("suite_computed_metrics", {})
    exp = out.get("suite_expected_documents", {})
    if not isinstance(scm, dict):
        scm = {}
    if not isinstance(exp, dict):
        exp = {}
    suite_out: dict[str, Any] = {}
    for sid, block in ps.items():
        if isinstance(block, dict):
            item = dict(block)
        else:
            item = block
        exp_docs = exp.get(sid)
        if isinstance(item, dict) and isinstance(exp_docs, int) and exp_docs > 0:
            item["_expected_documents"] = exp_docs
        suite_out[sid] = _suite_kpop(sid, item, scm.get(sid, []))
    out["kpop_gates"] = {
        "version": 1,
        "per_suite": suite_out,
    }
    return out


def kpop_has_failure(out: dict[str, Any]) -> bool:
    for v in out.get("kpop_gates", {}).get("per_suite", {}).values():
        if not isinstance(v, dict):
            continue
        rc = int(v.get("row_count", 0))
        if rc < 1:
            return True
        if not bool(v.get("kpop_documents_complete", True)):
            return True
        if not bool(v.get("kpop_all_rows_pass", False)):
            return True
    return False
