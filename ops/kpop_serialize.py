from __future__ import annotations

import json
import math
from typing import Any


def _norm_metric_value(v: float | None) -> float | None:
    if v is None:
        return None
    if isinstance(v, float) and math.isnan(v):
        return None
    return v


def _metrics_dict(src: Any) -> dict[str, float | None]:
    if not src:
        return {}
    d: dict[str, float | None] = {}
    for k, v in src.items():
        d[k] = _norm_metric_value(v)
    return d


def serialize_bench(bres: Any) -> dict[str, Any]:
    rows: list[dict[str, Any]] = []
    for r in bres.results:
        rows.append(
            {
                "document_id": r.document_id,
                "parser_name": r.parser_name,
                "success": r.success,
                "metrics": _metrics_dict(r.metrics),
                "error": r.error,
                "parse_time_seconds": r.parse_time_seconds,
            }
        )
    return {
        "benchmark_name": bres.benchmark_name,
        "timestamp": bres.timestamp,
        "success_count": bres.success_count,
        "failure_count": bres.failure_count,
        "rows": rows,
    }


def to_stdout(payload: dict[str, Any]) -> str:
    return json.dumps(payload, indent=2)
