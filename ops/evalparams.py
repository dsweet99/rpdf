from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True, slots=True)
class EvalParams:
    bench: Path
    rpdf_bin: Path
    rpdf_only: bool
    max_doc: int | None
    use_process_pool: bool
