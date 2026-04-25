from __future__ import annotations

import os
import sys
import time
from pathlib import Path
from typing import Any

from evalparams import EvalParams


def _cloud(pid: str) -> bool:
    t = pid.lower()
    for s in (
        "aws",
        "azure",
        "google",
        "llamaparse",
        "anthropic",
        "openai",
        "databricks",
        "sagemaker",
    ):
        if s in t:
            return True
    return t.startswith("pdfsmith-gemini")


def add_bench_path(bench: Path) -> None:
    s = str(bench.resolve())
    if s not in sys.path:
        sys.path.insert(0, s)


def resolve_parsers(
    bench: Path, rpdf_only: bool, all_registry: bool
) -> list[str]:
    add_bench_path(bench)
    if rpdf_only:
        return ["rpdf"]
    override = os.environ.get("RPDF_OPS_PARSERS", "").strip()
    if override:
        return [x.strip() for x in override.split(",") if x.strip()]
    from pdf_bench.loader import PARSER_ALIASES, PARSER_REGISTRY
    base = sorted(p for p in PARSER_REGISTRY if p not in PARSER_ALIASES)
    ex = os.environ.get("RPDF_OPS_EXCLUDE_CLOUD", "").strip().lower()
    ex_on = ex in ("1", "true", "yes", "y")
    if ex_on or not all_registry:
        return [p for p in base if not _cloud(p)]
    return base


def inner_workers(su: int) -> int:
    c = os.cpu_count() or 1
    if su > 1:
        return 1
    if c <= 1:
        return 0
    return min(4, c)


def par_count(p: EvalParams, n: int) -> int:
    if p.use_process_pool and n > 1:
        return min(3, n, max(1, (os.cpu_count() or 1) // 2))
    return 1


def run_from_pack(pack: tuple[str, str, str, int]) -> dict[str, Any]:
    yaml_path, benc, rpdf_bin, pworkers = pack
    bench = Path(benc)
    add_bench_path(bench)
    from kpop_serialize import serialize_bench
    from pdf_bench.loader import load_benchmark_config
    from pdf_bench.runner import BenchmarkRunner
    os.environ["RPDF_BIN"] = rpdf_bin
    t0 = time.perf_counter()
    loaded = load_benchmark_config(Path(yaml_path))
    br = BenchmarkRunner(loaded, parallel_workers=pworkers)
    bres = br.run()
    out = serialize_bench(bres)
    out["elapsed_seconds"] = time.perf_counter() - t0
    return out
