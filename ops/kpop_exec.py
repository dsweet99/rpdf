from __future__ import annotations

import os
import tempfile
from concurrent.futures import ProcessPoolExecutor, as_completed
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


def _build_yamls(b: dict[str, Any]) -> list[tuple[str, str]]:
    p, tmp, par, raw = b["p"], b["tmp"], b["parsers"], b["raw"]
    ypaths: list[tuple[str, str]] = []
    for suite_id, _lab, d, f, metrics in raw:
        c = {
            "bench": p.bench,
            "tmp": tmp,
            "suite_id": suite_id,
            "desc": d,
            "filt": f,
            "parsers": par,
            "pworkers": b["inner"],
            "metrics": metrics,
        }
        pth = write_suite_cfg(c)
        ypaths.append((suite_id, str(pth)))
    return ypaths


def _pool_run(h: dict[str, Any]) -> dict[str, Any]:
    yp, b_s, r, inner, su = h["ypaths"], h["bench_s"], h["rpdf"], h["inner"], h["su"]
    wargs: list[tuple[str, str, str, int]] = [
        (str(ypo), b_s, r, inner) for _, ypo in yp
    ]
    m: dict[str, Any] = {}
    with ProcessPoolExecutor(max_workers=su) as ex:
        futs = {ex.submit(run_from_pack, w): yp[i][0] for i, w in enumerate(wargs)}
        for ft in as_completed(futs):
            m[futs[ft]] = ft.result()
    return m


def _serial_run(s: dict[str, Any]) -> dict[str, Any]:
    yp, bench, r, inner = s["ypaths"], s["bench"], s["rpdf"], s["inner"]
    b_s = str(bench.resolve())
    m: dict[str, Any] = {}
    for sid, ypth in yp:
        wk: tuple[str, str, str, int] = (ypth, b_s, r, inner)
        m[sid] = run_from_pack(wk)
    return m


def _run_dispatch(d: dict) -> Any:
    yp, p, rpdf, h = d["yp"], d["p"], d["rd"], d["h"]
    b_s, su, i = str(p.bench.resolve()), h["su"], h["inner"]
    if su > 1:
        return _pool_run(
            {
                "ypaths": yp,
                "bench_s": b_s,
                "rpdf": rpdf,
                "inner": i,
                "su": su,
            }
        )
    return _serial_run(
        {
            "ypaths": yp,
            "bench": p.bench,
            "rpdf": rpdf,
            "inner": i,
        }
    )


def _suite_expected_documents(
    raw: list[tuple[str, str, str, dict[str, Any], list[str]]],
) -> dict[str, int]:
    out: dict[str, int] = {}
    for sid, _lab, _d, filt, _metrics in raw:
        n = filt.get("max_documents")
        if isinstance(n, int) and n > 0:
            out[sid] = n
    return out


def _run_eval_body(
    p: EvalParams, part: list[str], rpdf: str
) -> tuple[dict[str, Any], list[tuple[str, str, str, dict[str, Any], list[str]]]]:
    raw = list_suite_rows(p.bench, p.max_doc)
    su, inn = par_count(p, len(raw)), 0
    with tempfile.TemporaryDirectory() as td:
        tmp, inn = Path(td), inner_workers(su)
        b = {"p": p, "tmp": tmp, "parsers": part, "inner": inn, "raw": raw}
        yd = _build_yamls(b)
        m = _run_dispatch(
            {
                "yp": yd,
                "p": p,
                "rd": rpdf,
                "h": {"su": su, "inner": inn},
            }
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
    part = resolve_parsers(p.bench, p.rpdf_only, p.all_registry_parsers)
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
