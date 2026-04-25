from __future__ import annotations

import os
import sys
import time
from collections.abc import Iterable
from pathlib import Path
from typing import Any

from evalparams import EvalParams
from kpop_parser_ids import cloud, landing, ollama


def add_bench_path(bench: Path) -> None:
    s = str(bench.resolve())
    if s not in sys.path:
        sys.path.insert(0, s)


def _loader_aliases_or_empty() -> dict[str, object]:
    try:
        from pdf_bench.loader import PARSER_ALIASES
        if isinstance(PARSER_ALIASES, dict):
            return PARSER_ALIASES
    except ModuleNotFoundError as e:
        if e.name is None or not e.name.startswith("pdf_bench"):
            raise
    return {}


def _filter_explicit_parser_order(
    names: Iterable[str],
    aliases: dict[str, object],
) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for x in names:
        p = x.strip()
        if not p or p in seen or ollama(p, aliases) or cloud(p) or landing(p):
            continue
        seen.add(p)
        out.append(p)
    return out


def resolve_parsers(
    bench: Path,
    rpdf_only: bool,
    eval_default_parsers: tuple[str, ...] | None = None,
) -> list[str]:
    add_bench_path(bench)
    if rpdf_only:
        return ["rpdf"]
    aliases = _loader_aliases_or_empty()
    override = os.environ.get("RPDF_OPS_PARSERS", "").strip()
    if override:
        return _filter_explicit_parser_order(override.split(","), aliases)
    if eval_default_parsers is not None:
        return _filter_explicit_parser_order(eval_default_parsers, aliases)
    from pdf_bench.loader import PARSER_ALIASES, PARSER_REGISTRY
    base = sorted(
        p
        for p in PARSER_REGISTRY
        if p not in PARSER_ALIASES and not ollama(p, aliases)
    )
    return [p for p in base if not cloud(p) and not landing(p)]


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
    add_bench_path(Path(benc))
    return _run_with_rpdf_bin(yaml_path, rpdf_bin, pworkers)


def _run_with_rpdf_bin(
    yaml_path: str, rpdf_bin: str, pworkers: int
) -> dict[str, Any]:
    old_rpdf_bin = os.environ.get("RPDF_BIN")
    os.environ["RPDF_BIN"] = rpdf_bin
    try:
        return _run_pack_once(yaml_path, pworkers)
    finally:
        _restore_rpdf_bin(old_rpdf_bin)


def _restore_rpdf_bin(old_rpdf_bin: str | None) -> None:
    if old_rpdf_bin is None:
        os.environ.pop("RPDF_BIN", None)
        return
    os.environ["RPDF_BIN"] = old_rpdf_bin


def _run_pack_once(yaml_path: str, pworkers: int) -> dict[str, Any]:
    from kpop_serialize import serialize_bench
    from pdf_bench.loader import load_benchmark_config
    from pdf_bench.runner import BenchmarkRunner
    t0 = time.perf_counter()
    loaded = load_benchmark_config(Path(yaml_path))
    br = BenchmarkRunner(loaded, parallel_workers=pworkers)
    bres = br.run()
    out = serialize_bench(bres)
    out["elapsed_seconds"] = time.perf_counter() - t0
    return out
