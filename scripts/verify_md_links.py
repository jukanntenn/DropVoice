#!/usr/bin/env python3
"""Gate: relative Markdown links and fragment anchors resolve.

Scope (full mode): every .md in the repo except generated/vendored trees, the
four tool-adapter dirs, the skills mirror (byte copy, checked at the source),
read-only specs/, and CHANGELOG.md (generated). With file arguments, only those
files are checked. Check-only; never writes.

Usage:
    python scripts/verify_md_links.py [files...]
"""

from __future__ import annotations

import re
import sys
import urllib.parse
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

EXCLUDED_DIRS = {
    ".git", "node_modules", "target", "dist", "build", ".local",
    ".claude", ".codex", ".opencode", ".zcode", "specs",
    "test-results", "playwright-report", "coverage",
}
EXCLUDED_FILES = {"CHANGELOG.md"}

LINK = re.compile(r"\[([^\]]*)\]\(\s*<?([^)>#\s]+)(?:#[^)\s>]*)?(?:\s+\"[^\"]*\")?\s*>?\s*\)")
LINK_WITH_ANCHOR = re.compile(r"\[([^\]]*)\]\(\s*<?([^)\s>]*#[^)\s>]*)>(?:\s+\"[^\"]*\")?\s*\)")
HEADING = re.compile(r"^(#{1,6})\s+(.+)$")
ANCHOR_ID = re.compile(r"<a\s+id=[\"']([^\"']+)[\"']")

SKIP_SCHEMES = ("http://", "https://", "mailto:", "ftp://")


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


def slugify(heading: str) -> str:
    heading = re.sub(r"[*_`]", "", heading).strip().lower()
    heading = re.sub(r"[^\w\s-]", "", heading)
    return re.sub(r"\s", "-", heading).strip("-")


def anchors_of(text: str) -> set[str]:
    anchors = {slugify(HEADING.match(line).group(2)) for line in text.split("\n") if HEADING.match(line)}
    anchors.update(ANCHOR_ID.findall(text))
    return {a for a in anchors if a}


def check_file(path: Path) -> list[str]:
    rel = path.relative_to(REPO_ROOT).as_posix()
    errors: list[str] = []
    text = strip_fences(path.read_text(encoding="utf-8").replace("\r\n", "\n"))
    own_anchors = anchors_of(text)
    for lineno, line in enumerate(text.split("\n"), start=1):
        for match in list(LINK.finditer(line)) + list(LINK_WITH_ANCHOR.finditer(line)):
            label, raw = match.group(1), match.group(2)
            if not raw or raw.startswith(SKIP_SCHEMES):
                continue
            raw = urllib.parse.unquote(raw)
            if raw.startswith("#"):
                anchor = raw[1:]
                if anchor.isdigit():
                    continue
                if anchor not in own_anchors:
                    errors.append(f"{rel}:{lineno}: broken anchor `#{anchor}` (same-file heading not found)")
                continue
            target_part, _, anchor = raw.partition("#")
            target = (path.parent / target_part).resolve() if not target_part.startswith("/") else (REPO_ROOT / target_part.lstrip("/")).resolve()
            if target == REPO_ROOT or (REPO_ROOT not in target.parents and target != path):
                errors.append(f"{rel}:{lineno}: link `[{label}]({raw})` escapes the repository")
                continue
            if target.is_dir():
                errors.append(f"{rel}:{lineno}: link `[{label}]({raw})` targets a directory, not a file")
                continue
            if not target.is_file():
                errors.append(f"{rel}:{lineno}: broken link `[{label}]({raw})` (file not found)")
                continue
            if anchor and not anchor.isdigit():
                target_anchors = anchors_of(strip_fences(target.read_text(encoding="utf-8").replace("\r\n", "\n")))
                if anchor not in target_anchors:
                    errors.append(f"{rel}:{lineno}: broken anchor `#{anchor}` in {target.relative_to(REPO_ROOT).as_posix()}")
    return errors


def discover() -> list[Path]:
    files: list[Path] = []
    stack = [REPO_ROOT]
    while stack:
        current = stack.pop()
        for entry in current.iterdir():
            if entry.is_dir():
                if entry.name not in EXCLUDED_DIRS:
                    stack.append(entry)
            elif entry.suffix == ".md" and entry.name not in EXCLUDED_FILES:
                files.append(entry)
    return sorted(files)


def run(files: list[str] | None) -> list[str]:
    targets = (
        [REPO_ROOT / f.replace("\\", "/") for f in files if f.replace("\\", "/").endswith(".md")]
        if files
        else discover()
    )
    errors: list[str] = []
    for path in targets:
        if not path.is_file() or REPO_ROOT not in path.parents:
            continue
        if any(part in EXCLUDED_DIRS for part in path.relative_to(REPO_ROOT).parts):
            continue
        errors.extend(check_file(path))
    return errors


def main(argv: list[str]) -> int:
    errors = run(argv[1:] or None)
    for e in errors:
        print(f"error: {e}")
    print(f"verify_md_links: {'FAIL' if errors else 'PASS'} ({len(errors)} error(s))")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
