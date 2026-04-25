from __future__ import annotations

import os
import tempfile
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from evalparams import EvalParams
from kpop_loader import (
    inner_workers,
    par_count,
    resolve_parsers,
    run_from_pack,
)
from kpop_suites import list_suite_rows
from kpop_write import write_suite_cfg

SuiteRow = tuple[str, str, str, dict[str, Any], list[str]]


@dataclass(frozen=True, slots=True)
class RunCtx:
    bench: Path
    rpdf_bin: str
    params: EvalParams | None = None
    tmp: Path | None = None
    parsers: list[str] | None = None
    inner_workers: int = 0
    raw_rows: list[SuiteRow] | None = None
    ypaths: list[tuple[str, str]] | None = None
    suite_workers: int = 1


def _build_yamls(inp: RunCtx) -> list[tuple[str, str]]:
    if inp.params is None or inp.tmp is None or inp.parsers is None or inp.raw_rows is None:
        raise ValueError("build context missing required fields")
    ypaths: list[tuple[str, str]] = []
    for suite_id, _lab, d, f, metrics in inp.raw_rows:
        c = {
            "bench": inp.params.bench,
            "tmp": inp.tmp,
            "suite_id": suite_id,
            "desc": d,
            "filt": f,
            "parsers": inp.parsers,
            "pworkers": inp.inner_workers,
            "metrics": metrics,
        }
        pth = write_suite_cfg(c)
        ypaths.append((suite_id, str(pth)))
    return ypaths


def _pool_run(inp: RunCtx) -> dict[str, Any]:
    if inp.ypaths is None:
        raise ValueError("dispatch context missing ypaths")
    b_s = str(inp.bench.resolve())
    wargs: list[tuple[str, str, str, int]] = [
        (str(ypo), b_s, inp.rpdf_bin, inp.inner_workers) for _, ypo in inp.ypaths
    ]
    m: dict[str, Any] = {}
    with ProcessPoolExecutor(max_workers=inp.suite_workers) as ex:
        futs = {
            ex.submit(run_from_pack, w): inp.ypaths[i][0] for i, w in enumerate(wargs)
        }
        for ft in as_completed(futs):
            m[futs[ft]] = ft.result()
    return m


def _serial_run(inp: RunCtx) -> dict[str, Any]:
    if inp.ypaths is None:
        raise ValueError("dispatch context missing ypaths")
    b_s = str(inp.bench.resolve())
    m: dict[str, Any] = {}
    for sid, ypth in inp.ypaths:
        wk: tuple[str, str, str, int] = (
            ypth,
            b_s,
            inp.rpdf_bin,
            inp.inner_workers,
        )
        m[sid] = run_from_pack(wk)
    return m


def _run_dispatch(inp: RunCtx) -> dict[str, Any]:
    if inp.suite_workers > 1:
        return _pool_run(inp)
    return _serial_run(inp)


def _suite_expected_documents(
    raw: list[SuiteRow],
) -> dict[str, int]:
    out: dict[str, int] = {}
    for sid, _lab, _d, filt, _metrics in raw:
        n = filt.get("max_documents")
        if isinstance(n, int) and n > 0:
            out[sid] = n
    return out


def _run_eval_body(
    p: EvalParams, part: list[str], rpdf: str
) -> tuple[dict[str, Any], list[SuiteRow]]:
    raw = list_suite_rows(p.bench, p.max_doc)
    su, inn = par_count(p, len(raw)), 0
    with tempfile.TemporaryDirectory() as td:
        tmp, inn = Path(td), inner_workers(su)
        yd = _build_yamls(
            RunCtx(
                params=p,
                tmp=tmp,
                parsers=part,
                inner_workers=inn,
                raw_rows=raw,
                bench=p.bench,
                rpdf_bin=rpdf,
            )
        )
        m = _run_dispatch(
            RunCtx(
                ypaths=yd,
                bench=p.bench,
                rpdf_bin=rpdf,
                inner_workers=inn,
                suite_workers=su,
            )
        )
    return m, raw


def _restore_rpdf_bin_env(old_rpdf_bin: str | None, needs_rpdf_bin: bool) -> None:
    if not needs_rpdf_bin:
        return
    if old_rpdf_bin is None:
        os.environ.pop("RPDF_BIN", None)
    else:
        os.environ["RPDF_BIN"] = old_rpdf_bin


def run_eval(p: EvalParams) -> dict[str, Any]:
    corp = p.bench / "corpus"
    if not corp.is_dir():
        msg = f"bench corpus missing: {corp}"
        raise FileNotFoundError(msg)
    part = resolve_parsers(p.bench, p.rpdf_only, p.eval_default_parsers)
    if not part:
        msg = "no parsers selected"
        raise ValueError(msg)
    rpdf = str(p.rpdf_bin.resolve())
    needs_rpdf_bin = "rpdf" in part
    if needs_rpdf_bin and not p.rpdf_bin.is_file():
        msg = f"rpdf binary not found: {p.rpdf_bin}"
        raise FileNotFoundError(msg)
    old_rpdf_bin = os.environ.get("RPDF_BIN")
    if needs_rpdf_bin:
        os.environ["RPDF_BIN"] = rpdf
    try:
        m, raw = _run_eval_body(p, part, rpdf)
    finally:
        _restore_rpdf_bin_env(old_rpdf_bin, needs_rpdf_bin)
    scm = {t[0]: t[4] for t in raw}
    expected = _suite_expected_documents(raw)
    return {
        "parsers": part,
        "per_suite": m,
        "suite_computed_metrics": scm,
        "suite_expected_documents": expected,
    }
