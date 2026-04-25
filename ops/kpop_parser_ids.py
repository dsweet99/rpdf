from __future__ import annotations

import re


def ollama(
    pid: str,
    aliases: dict[str, object] | None = None,
    seen: set[str] | None = None,
) -> bool:
    t = pid.strip().lower()
    tokens = {tok for tok in re.split(r"[^a-z0-9]+", t) if tok}
    if "ollama" in tokens:
        return True
    normalized = re.sub(r"[^a-z0-9]+", "", t)
    if "ollama" in normalized:
        return True
    if aliases is None:
        return False
    if not normalized:
        return False
    if seen is None:
        seen = set()
    if normalized in seen:
        return False
    seen.add(normalized)
    target = aliases.get(pid)
    if not isinstance(target, str):
        for k, v in aliases.items():
            if not isinstance(k, str):
                continue
            nk = re.sub(r"[^a-z0-9]+", "", k.strip().lower())
            if nk == normalized:
                target = v
                break
    return isinstance(target, str) and ollama(target, aliases, seen)


def cloud(pid: str) -> bool:
    t = pid.strip().lower()
    tokens = {x for x in re.split(r"[^a-z0-9]+", t) if x}
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
        if s in tokens:
            return True
    n = re.sub(r"[^a-z0-9]+", "", t)
    if n.startswith("pdfsmithgemini"):
        return True
    for s in ("azure", "google", "llamaparse", "anthropic", "openai", "databricks"):
        if f"pdfsmith{s}" in n:
            return True
    return "pdfsmithsagemaker" in n or "pdfsmithaws" in n


def landing(pid: str) -> bool:
    n = re.sub(r"[^a-z0-9]+", "", pid.strip().lower())
    return n == "landingai" or n.startswith("landingai")
