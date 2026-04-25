#!/usr/bin/env python3
from __future__ import annotations

import os
import time
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import TypeVar, cast

import click

F = TypeVar("F", bound=Callable[..., object])


@dataclass(frozen=True, slots=True)
class GoIn:
    rpdf_only: bool
    bench_dir: Path | None
    rpdf_bin: Path | None
    max_doc: int | None
    no_suite_mp: bool


def _root() -> Path:
    return Path(__file__).resolve().parent.parent


def _default_bench() -> Path:
    p = os.environ.get("PDF_PARSER_BENCHMARK_DIR", "").strip()
    if p:
        return Path(p)
    r = _root()
    cands = [r.parent / "pdf-parser-benchmark", r / "pdf-parser-benchmark"]
    for c in cands:
        if c.is_dir():
            return c
    return cands[0]


def _pick_rpdf() -> Path:
    for p in (
        _root() / "target" / "release" / "rpdf",
        _root() / "target" / "debug" / "rpdf",
    ):
        if p.is_file():
            return p
    return _root() / "target" / "release" / "rpdf"


def _ops_traceback_env() -> bool:
    v = os.environ.get("RPDF_OPS_TRACEBACK", "").strip().lower()
    return v in {"1", "true", "yes", "y"}


def _run_wrapped(f: Callable[[], object]) -> object:
    try:
        return f()
    except click.ClickException:
        raise
    except Exception as e:
        if _ops_traceback_env():
            raise
        raise click.ClickException(str(e)) from e


def go(g: GoIn) -> None:
    from evalparams import EvalParams
    from kpop_exec import run_eval
    from kpop_gates import attach_kpop, kpop_has_failure
    from kpop_serialize import to_stdout
    b = _default_bench() if g.bench_dir is None else g.bench_dir
    if not b.is_dir():
        msg = f"bench dir not found: {b}"
        raise click.ClickException(msg)
    r = _pick_rpdf() if g.rpdf_bin is None else g.rpdf_bin
    ev = os.environ.get("RPDF_BIN", "")
    if g.rpdf_bin is None and ev:
        t = Path(ev)
        if t.is_file():
            r = t
    t0 = time.perf_counter()
    p = EvalParams(
        bench=b,
        rpdf_bin=r,
        rpdf_only=g.rpdf_only,
        max_doc=g.max_doc,
        use_process_pool=not g.no_suite_mp,
    )
    out = _run_wrapped(lambda: run_eval(p))
    out = _run_wrapped(lambda: attach_kpop(out))
    out["output_schema"] = "kpop_ops_eval_v1_json"
    out["elapsed_seconds"] = time.perf_counter() - t0
    click.echo(_run_wrapped(lambda: to_stdout(out)))
    if kpop_has_failure(out):
        raise SystemExit(1)


_SUITE_MP_HELP = "Run suites one after another; disable suite-level process pool."


def _shared_command_options(f: F) -> F:
    f = click.option(
        "--bench-dir",
        type=click.Path(file_okay=False, path_type=Path),
        default=None,
    )(f)
    f = click.option(
        "--rpdf-bin",
        type=click.Path(exists=True, path_type=Path, dir_okay=False),
        default=None,
    )(f)
    f = click.option(
        "--max-documents",
        "max_doc",
        type=click.IntRange(min=1),
        default=None,
    )(f)
    f = click.option(
        "--no-suite-mp",
        is_flag=True,
        default=False,
        help=_SUITE_MP_HELP,
    )(f)
    return cast("F", f)


@click.group()
def cli() -> None:
    pass


@cli.command(
    "all",
    context_settings={"show_default": True},
    help=(
        "Run every parser in the pdf_bench registry that can be exercised locally without "
        "remote APIs or paid services (omits ollama, cloud/API ids, and landing). "
        "Set RPDF_OPS_PARSERS to a comma list to override."
    ),
)
@_shared_command_options
def all_cmd(
    bench_dir: Path | None,
    rpdf_bin: Path | None,
    max_doc: int | None,
    *,
    no_suite_mp: bool,
) -> None:
    g = GoIn(
        rpdf_only=False,
        bench_dir=bench_dir,
        rpdf_bin=rpdf_bin,
        max_doc=max_doc,
        no_suite_mp=no_suite_mp,
    )
    go(g)


@cli.command("rpdf", context_settings={"show_default": True})
@_shared_command_options
def rpdf_cmd(
    bench_dir: Path | None,
    rpdf_bin: Path | None,
    max_doc: int | None,
    *,
    no_suite_mp: bool,
) -> None:
    g = GoIn(
        rpdf_only=True,
        bench_dir=bench_dir,
        rpdf_bin=rpdf_bin,
        max_doc=max_doc,
        no_suite_mp=no_suite_mp,
    )
    go(g)


if __name__ == "__main__":
    cli()
