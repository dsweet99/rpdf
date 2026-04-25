from __future__ import annotations

import importlib.util
from importlib import metadata

import pytest


def test_torchvision_import_matches_torch_when_both_installed() -> None:
    if importlib.util.find_spec("torch") is None:
        pytest.skip("torch not installed")
    if importlib.util.find_spec("torchvision") is None:
        pytest.skip("torchvision not installed")
    import torch

    torch_ver = torch.__version__.split("+", maxsplit=1)[0]
    tv_ver = metadata.version("torchvision")
    try:
        import torchvision
    except RuntimeError as exc:
        msg = str(exc)
        if "torchvision::nms" in msg or "does not exist" in msg:
            pytest.fail(
                "torchvision is not ABI-compatible with torch; reinstall paired wheels "
                f"(torch {torch.__version__}, torchvision {tv_ver} from metadata). "
                f"Original error: {exc}"
            )
        raise
    tv_mod = torchvision.__version__.split("+", maxsplit=1)[0]
    assert tv_mod == tv_ver.split("+", maxsplit=1)[0]
    major_minor = tuple(int(x) for x in torch_ver.split(".")[:2])
    tv_mm = tuple(int(x) for x in tv_ver.split(".")[:2])
    if major_minor >= (2, 10) and tv_mm < (0, 25):
        pytest.fail(
            f"torch {torch_ver} requires torchvision>=0.25 (have {tv_ver}); "
            "see requirements-ops.txt and PyTorch install matrix."
        )
