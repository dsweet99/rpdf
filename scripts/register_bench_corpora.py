"""Make non-synthetic pdf-parser-benchmark corpora visible to its own loader.

Background
----------
The upstream ``pdf-parser-benchmark`` repo ships builders that produce per-doc
metadata YAMLs in schemas the upstream loader cannot read. As a result, after
running ``builders/build_hr_corpus.py``, ``builders/build_legal_corpus.py``,
``builders/generate_invoice_corpus.py``, or ``builders/download_arxiv_corpus.py``,
``pdf_bench.utils.metadata.load_metadata`` raises ``KeyError: 'doc_id'`` for
every produced YAML and ``discover_documents`` silently drops the corpus.

This script
-----------
1. Walks ``corpus/business/{hr,legal,invoices}/pdfs/`` and ``corpus/academic/arxiv/pdfs/``
   in the benchmark repo and overwrites/creates a minimal loader-conforming
   YAML at ``<corpus>/metadata/<stem>.yaml`` for each PDF on disk.
2. For arxiv, if ``corpus/academic/arxiv/ar5iv/*.html`` exists and ``pandoc``
   is on PATH, runs the upstream ``builders.ar5iv_to_markdown`` converter to
   write Markdown ground truth to ``corpus/academic/arxiv/ground_truth/``.

The script is idempotent and safe to re-run.

Usage
-----
    python scripts/register_bench_corpora.py [--bench-dir /path/to/pdf-parser-benchmark]
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

import yaml

DEFAULT_BENCH_DIR = Path("/home/dsweet/Projects/pdfs/pdf-parser-benchmark")


def _emit_yaml(meta_path: Path, payload: dict) -> None:
    meta_path.parent.mkdir(parents=True, exist_ok=True)
    with open(meta_path, "w", encoding="utf-8") as f:
        yaml.safe_dump(payload, f, sort_keys=False, allow_unicode=True)


def _convert_business(corpus_root: Path, domain: str, title_prefix: str) -> int:
    pdf_dir = corpus_root / "pdfs"
    meta_dir = corpus_root / "metadata"
    if not pdf_dir.is_dir():
        return 0
    n = 0
    for pdf in sorted(pdf_dir.glob("*.pdf")):
        stem = pdf.stem
        _emit_yaml(meta_dir / f"{stem}.yaml", {
            "doc_id": stem,
            "source_url": "generated://",
            "corpus": "business",
            "domain": domain,
            "title": f"{title_prefix}{stem}",
        })
        n += 1
    return n


def _convert_arxiv(bench_dir: Path) -> int:
    arxiv_root = bench_dir / "corpus" / "academic" / "arxiv"
    pdf_dir = arxiv_root / "pdfs"
    meta_dir = arxiv_root / "metadata"
    n = 0
    for pdf in sorted(pdf_dir.glob("*.pdf")):
        arxiv_id = pdf.stem
        _emit_yaml(meta_dir / f"{arxiv_id}.yaml", {
            "doc_id": f"arxiv_{arxiv_id}",
            "source_url": f"https://arxiv.org/abs/{arxiv_id}",
            "corpus": "academic",
            "domain": "arxiv",
            "title": f"arXiv {arxiv_id}",
        })
        n += 1
    return n


def _build_arxiv_ground_truth(bench_dir: Path) -> tuple[int, int]:
    """Returns (succeeded, total). Returns (0, 0) if not runnable."""
    ar5iv_dir = bench_dir / "corpus" / "academic" / "arxiv" / "ar5iv"
    if not ar5iv_dir.is_dir():
        return (0, 0)
    if shutil.which("pandoc") is None:
        print("  pandoc not found on PATH; skipping arxiv ground-truth generation",
              file=sys.stderr)
        return (0, 0)
    sys.path.insert(0, str(bench_dir))
    try:
        from builders.ar5iv_to_markdown import convert_all_arxiv_samples
    except Exception as exc:
        print(f"  failed to import ar5iv_to_markdown: {exc}", file=sys.stderr)
        return (0, 0)
    output_dir = bench_dir / "corpus" / "academic" / "arxiv" / "ground_truth"
    results = convert_all_arxiv_samples(sample_dir=ar5iv_dir, output_dir=output_dir)
    ok = sum(1 for r in results.values() if r["success"])
    return (ok, len(results))


def _rewrite_all_metadata(bench_dir: Path) -> list[tuple[str, int]]:
    business_specs = [
        ("hr", "hr", "HR document "),
        ("legal", "legal", "Legal document "),
        ("invoices", "invoices", "Invoice "),
    ]
    counts: list[tuple[str, int]] = []
    for name, domain, title_prefix in business_specs:
        n = _convert_business(
            bench_dir / "corpus" / "business" / name,
            domain=domain,
            title_prefix=title_prefix,
        )
        counts.append((name, n))
    counts.append(("arxiv", _convert_arxiv(bench_dir)))
    return counts


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bench-dir", type=Path, default=DEFAULT_BENCH_DIR,
                        help="Path to pdf-parser-benchmark checkout")
    parser.add_argument("--skip-arxiv-gt", action="store_true",
                        help="Do not run the ar5iv->markdown ground-truth converter")
    return parser.parse_args()


def main() -> None:
    args = _parse_args()

    bench_dir: Path = args.bench_dir.resolve()
    if not bench_dir.is_dir():
        sys.exit(f"benchmark dir not found: {bench_dir}")

    print(f"benchmark dir: {bench_dir}")
    print("rewriting metadata yaml so the loader can discover each PDF...")
    for name, n in _rewrite_all_metadata(bench_dir):
        print(f"  {name}: {n} metadata yaml files")

    if not args.skip_arxiv_gt:
        print("\ngenerating arxiv ground-truth markdown via pandoc + ar5iv...")
        ok, total = _build_arxiv_ground_truth(bench_dir)
        if total:
            print(f"  arxiv ground truth: {ok}/{total} markdown files written")
        else:
            print("  arxiv ground truth: nothing to do")


if __name__ == "__main__":
    main()
