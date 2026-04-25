from __future__ import annotations

FIVE: list[str] = [
    "edit_similarity",
    "chrf++",
    "character_error_rate",
    "tree_similarity",
    "element_f1",
]

CUAD_FOUR: list[str] = [
    "edit_similarity",
    "chrf++",
    "character_error_rate",
    "element_f1",
]

ARXIV_THREE: list[str] = [
    "edit_similarity",
    "chrf++",
    "character_error_rate",
]


def gating_metrics_for_suite(sid: str) -> list[str]:
    if sid in {"synthetic", "legal", "invoices", "hr"}:
        return list(FIVE)
    if sid == "cuad":
        return list(CUAD_FOUR)
    if sid == "arxiv":
        return list(ARXIV_THREE)
    return list(FIVE)


def applicable_gates(sid: str, computed: list[str]) -> list[str]:
    want = gating_metrics_for_suite(sid)
    cset = set(computed)
    return [m for m in want if m in cset]
