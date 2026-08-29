#!/usr/bin/env python3
"""Gate: bilingual prose pairs — every in-scope foo.md has a structurally equal foo.zh.md.

Pairing scope and per-path exclusions live in scripts/doc_languages.manifest.json
(positive glob patterns over repo-relative paths). Agent instructions (any
AGENTS.md/CLAUDE.md) and the skills tree are categorically single-language.

Parity rules: the twin exists (both directions), the heading-depth sequence is
identical, and fenced code blocks are byte-identical. Prose may translate —
including headings. Check-only; never writes.

Usage:
    python scripts/verify_doc_pairs.py [files...]   # no args = full scope
"""

from __future__ import annotations

import fnmatch
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST = REPO_ROOT / "scripts" / "doc_languages.manifest.json"

HEADING = re.compile(r"^(#{1,6})\s+\S")


def strip_fences(text: str) -> str:
    out: list[str] = []
    fence = None
    for line in text.split("\n"):
        stripped = line.lstrip()
        if fence is None:
            if stripped.startswith("```") or stripped.startswith("~~~"):
                fence = stripped[:3]
            else:
                out.append(line)
        elif stripped.startswith(fence):
            fence = None
    return "\n".join(out)


def fenced_blocks(text: str) -> list[str]:
    blocks: list[str] = []
    current: list[str] | None = None
    fence = None
    for line in text.split("\n"):
        stripped = line.lstrip()
        if current is None and (stripped.startswith("```") or stripped.startswith("~~~")):
            fence = stripped[:3]
            current = []
            continue
        if current is not None and stripped.startswith(fence):
            blocks.append("\n".join(current))
            current = None
            fence = None
            continue
        if current is not None:
            current.append(line)
    if current is not None:
        blocks.append("\n".join(current))
    return blocks


def heading_depths(text: str) -> list[str]:
    return [HEADING.match(line).group(1) for line in text.split("\n") if HEADING.match(line)]


def scope(manifest: dict) -> set[str]:
    files: set[str] = set()
    for pattern in manifest.get("patterns", []):
        base = REPO_ROOT / Path(pattern).parent
        if base.is_dir():
            files.update(
                p.relative_to(REPO_ROOT).as_posix()
                for p in base.glob(Path(pattern).name)
                if p.is_file()
            )
    for exclusion in manifest.get("exclude", []):
        files.discard(exclusion)
        files = {f for f in files if not fnmatch.fnmatch(f, exclusion)}
    # Agent instructions are categorically single-language; .zh.md files are the
    # twins of in-scope roots, never roots themselves.
    return {
        f
        for f in files
        if Path(f).name not in ("AGENTS.md", "CLAUDE.md") and not f.endswith(".zh.md")
    }


def check_pair(en: Path, zh: Path, errors: list[str]) -> None:
    rel = en.relative_to(REPO_ROOT).as_posix()
    if not zh.exists():
        errors.append(f"{rel}: missing twin {zh.name} (pairs update in the same change)")
        return
    en_text = en.read_text(encoding="utf-8").replace("\r\n", "\n")
    zh_text = zh.read_text(encoding="utf-8").replace("\r\n", "\n")
    if heading_depths(strip_fences(en_text)) != heading_depths(strip_fences(zh_text)):
        errors.append(f"{rel}: heading-depth sequence differs from {zh.name}")
    if fenced_blocks(en_text) != fenced_blocks(zh_text):
        errors.append(f"{rel}: fenced code blocks differ from {zh.name} (blocks must be byte-identical)")


def run(files: list[str] | None) -> list[str]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    in_scope = scope(manifest)
    errors: list[str] = []
    if files is None:
        candidates = sorted(in_scope)
    else:
        # Map passed files (either member of a pair) to their English root.
        roots = set()
        for f in files:
            rel = f.replace("\\", "/")
            if rel.endswith(".zh.md"):
                rel = rel[: -len(".zh.md")] + ".md"
            roots.add(rel)
        candidates = sorted(roots & in_scope)
    for rel in candidates:
        check_pair(REPO_ROOT / rel, REPO_ROOT / (rel[: -len(".md")] + ".zh.md"), errors)
    return errors


def main(argv: list[str]) -> int:
    errors = run(argv[1:] or None)
    for e in errors:
        print(f"error: {e}")
    print(f"verify_doc_pairs: {'FAIL' if errors else 'PASS'} ({len(errors)} error(s))")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
