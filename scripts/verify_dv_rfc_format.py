#!/usr/bin/env python3
"""DV-RFC format gate: filename grammar, header block, lifecycle skeletons, pair parity.

Contract: .agents/dv-rfcs/README.md. Check-only; never writes.

Checks (on both members of a pair):
  - filename grammar ^yyyy-mm-dd-topic(.zh)?.md$ with a real calendar date
  - line 1 `# DV-RFC: <title>`, line 3 `Status: ...` agreeing with the folder
    (rejected requires a one-line reason on the Status line)
  - body opens with `## Problem`
  - mandatory `## Alternatives considered`
  - per-lifecycle skeleton: proposed needs Proposal/Acceptance criteria/Risks;
    implemented needs Decision/Consequences and bans proposal-speak headings;
    rejected keeps its proposal-time sections
  - pair parity: the .zh.md twin exists (both directions), heading sequence is
    identical, fenced code blocks are byte-identical
  - full-tree mode also rejects stray files and centralized index files

Usage:
    python scripts/verify_dv_rfc_format.py            # whole tree
    python scripts/verify_dv_rfc_format.py <files>    # only these files (prek passes staged)
"""

from __future__ import annotations

import re
import sys
from datetime import date
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
RFC_ROOT = REPO_ROOT / ".agents" / "dv-rfcs"

LIFECYCLES = ("implemented", "proposed", "rejected")
CONTRACT_FILES = {"AGENTS.md", "README.md", "README.zh.md"}
KEEPERS = {".gitkeep"}

FILENAME = re.compile(r"^(\d{4})-(\d{2})-(\d{2})-[a-z0-9][a-z0-9-]*(\.zh)?\.md$")
TITLE = re.compile(r"^# DV-RFC: \S")
STATUS = re.compile(r"^Status: (proposed|implemented|rejected)(.*)$")
PROPOSAL_SPEAK = re.compile(r"^## (?:proposal|plan|migration plan|acceptance criteria)\b", re.IGNORECASE)

REQUIRED = {
    "proposed": ("Proposal", "Acceptance criteria", "Risks"),
    "implemented": ("Decision", "Consequences"),
    "rejected": ("Proposal",),
}
ALTERNATIVES = "Alternatives considered"


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8").replace("\r\n", "\n")


def strip_fences(text: str) -> str:
    """Drop fenced code blocks so markdown examples cannot fake headings."""
    out: list[str] = []
    fence = None
    for line in text.split("\n"):
        stripped = line.lstrip()
        if fence is None and (stripped.startswith("```") or stripped.startswith("~~~")):
            fence = stripped[:3]
            continue
        if fence is not None and stripped.startswith(fence):
            fence = None
            continue
        out.append(line)
    return "\n".join(out)


def heading_lines(text: str) -> list[str]:
    return [line for line in text.split("\n") if re.match(r"^#{1,6} \S", line)]


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


def _check_file(rel: str, path: Path, errors: list[str]) -> None:
    def err(msg: str) -> None:
        errors.append(f"{rel}: {msg}")

    lifecycle = rel.split("/")[0]
    match = FILENAME.match(path.name)
    if not match:
        err(f"filename does not match yyyy-mm-dd-topic-title(.zh).md grammar: {path.name}")
        return
    y, m, d = int(match.group(1)), int(match.group(2)), int(match.group(3))
    try:
        date(y, m, d)
    except ValueError:
        err(f"filename date {y:04d}-{m:02d}-{d:02d} is not a real calendar date")
        return
    if match.group(4):  # .zh twin: its English original must exist
        if not path.with_name(path.name[: -len(".zh.md")] + ".md").exists():
            err("zh twin has no English original")
            return

    text = _read(path)
    lines = text.split("\n")
    if not lines or not TITLE.match(lines[0]):
        err("line 1 must be `# DV-RFC: <title>`")
        return
    if len(lines) < 3 or lines[1] != "" or not STATUS.match(lines[2]):
        err("lines 1-3 must be `# DV-RFC: <title>`, blank, `Status: <status>`")
        return

    status = STATUS.match(lines[2])
    verb, rest = status.group(1), status.group(2)
    if verb != lifecycle:
        err(f"Status `{verb}` disagrees with lifecycle folder `{lifecycle}`")
    if verb == "rejected" and not re.match(r"^ — \S", rest):
        err("rejected records carry the verdict on the Status line: `Status: rejected — <one-line reason>`")
    if verb in ("proposed", "implemented") and rest.strip():
        err(f"`Status: {verb}` carries no extra text (dates live in the filename, history in git)")

    body = strip_fences(text)
    headings = heading_lines(body)
    h2 = [h[3:].strip() for h in headings if h.startswith("## ")]
    if not h2:
        err("body has no `##` sections")
        return
    if h2[0] != "Problem":
        err(f"body must open with `## Problem` (found `## {h2[0]}`)")
    for section in REQUIRED[lifecycle]:
        if section not in h2:
            err(f"missing required section `## {section}`")
    if ALTERNATIVES not in h2:
        err(f"missing mandatory section `## {ALTERNATIVES}`")
    if lifecycle == "implemented":
        for line in headings:
            if line.startswith("## ") and PROPOSAL_SPEAK.match(line):
                err(f"proposal-speak heading banned in implemented/: `{line}`")

    # Pair parity, checked once per pair from the English side.
    if not match.group(4):
        twin = path.with_name(path.name[:-len(".md")] + ".zh.md")
        if not twin.exists():
            err("English record has no .zh.md twin (both update in the same change)")
            return
        zh_text = _read(twin)
        if heading_lines(strip_fences(text)) != heading_lines(strip_fences(zh_text)):
            err("heading sequence differs from .zh.md twin (headings stay English in both)")
        if fenced_blocks(text) != fenced_blocks(zh_text):
            err("fenced code blocks differ from .zh.md twin (blocks must be byte-identical)")


def _walk_tree(errors: list[str]) -> list[Path]:
    """Validate tree structure; return RFC file paths (English + zh)."""
    files: list[Path] = []
    for entry in sorted(RFC_ROOT.iterdir()):
        name = entry.name
        if entry.is_file():
            if name not in CONTRACT_FILES and name not in KEEPERS:
                errors.append(f"{name}: stray file at tree root (records live under lifecycle dirs)")
            continue
        if name not in LIFECYCLES:
            errors.append(f"{name}: unknown entry (tree = {', '.join(LIFECYCLES)} + contract files)")
            continue
        for f in sorted(entry.iterdir()):
            rel = f.relative_to(RFC_ROOT).as_posix()
            if f.is_dir():
                errors.append(f"{rel}: directories do not nest below the lifecycle folder")
            elif f.name == "INDEX.md":
                errors.append(f"{rel}: centralized indexes are forbidden; browse the lifecycle tree or search")
            elif f.name in KEEPERS:
                continue
            else:
                files.append(f)
    return files


def run(files: list[str] | None) -> list[str]:
    """Validate the whole tree (files=None) or just the given repo-relative paths."""
    errors: list[str] = []
    if files is None:
        targets = _walk_tree(errors)
    else:
        targets = []
        for f in files:
            rel = f.replace("\\", "/")
            parts = rel.split("/")
            if (
                len(parts) == 3
                and parts[0] == ".agents"
                and parts[1] == "dv-rfcs"
                and parts[2] in LIFECYCLES
            ):
                targets.append(REPO_ROOT / rel)
    checked = 0
    for path in targets:
        rel = path.relative_to(RFC_ROOT).as_posix()
        if path.suffix == ".md" and path.name not in CONTRACT_FILES:
            _check_file(rel, path, errors)
            checked += 1
    return errors


def main(argv: list[str]) -> int:
    errors = run(argv[1:] or None)
    for e in errors:
        print(f"error: {e}")
    total = len(list(RFC_ROOT.glob("*/*.md"))) if RFC_ROOT.exists() else 0
    state = "FAIL" if errors else "PASS"
    print(f"verify_dv_rfc_format: {state} ({len(errors)} error(s), {total} record(s) in tree)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
