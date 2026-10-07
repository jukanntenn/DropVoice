#!/usr/bin/env python3
"""Gate: every AGENTS.md has a byte-identical CLAUDE.md twin; .claude/skills mirrors .agents/skills.

AGENTS.md is the source of truth at every level; CLAUDE.md files and the skills
mirror are copies. The gate's contract is "what is committed must be mirrored",
so all comparisons run against the git INDEX (staged blobs, falling back to
HEAD's tracked state for unstaged paths). The worktree is deliberately not
scanned: prek's stash cycle and partial (`git commit -- <paths>`) commits both
rewrite the worktree and the index-view in transient ways a filesystem scan
misreads (staged deletions can appear resurrected; out-of-pathspec files can
appear at their HEAD state).

Edits become visible to this gate once staged (`git add`); CI's
`prek run --all-files` checks the committed state the same way.

This script only CHECKS; it never writes. On drift, exit 1 with the fix:
`python scripts/sync_agent_files.py` (then re-stage both sides).

Usage:
    python scripts/sync-agents.py
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# (source AGENTS.md, mirrored CLAUDE.md) — fixed direction, AGENTS.md is the source.
PAIRS = [
    ("AGENTS.md", "CLAUDE.md"),
    ("apps/desktop/AGENTS.md", "apps/desktop/CLAUDE.md"),
    ("apps/mobile/AGENTS.md", "apps/mobile/CLAUDE.md"),
    ("apps/pairing-server/AGENTS.md", "apps/pairing-server/CLAUDE.md"),
    ("apps/landing/AGENTS.md", "apps/landing/CLAUDE.md"),
    ("packages/AGENTS.md", "packages/CLAUDE.md"),
    ("docs/AGENTS.md", "docs/CLAUDE.md"),
]

SKILLS_SOURCE = ".agents/skills"
SKILLS_MIRROR = ".claude/skills"
_FIX = "fix: python scripts/sync_agent_files.py, then re-stage both sides"


def _git(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=REPO_ROOT, capture_output=True)


def _tracked_files() -> set[str]:
    result = _git(["ls-files", "--", *[src for pair in PAIRS for src in pair], SKILLS_SOURCE, SKILLS_MIRROR])
    return set(result.stdout.decode("utf-8").split()) if result.returncode == 0 else set()


_INDEXED = _tracked_files()


def _present(path: str) -> bool:
    return path in _INDEXED


def _content(path: str) -> bytes | None:
    result = _git(["show", f":{path}"])
    return result.stdout if result.returncode == 0 else None


def check_pairs() -> list[str]:
    errors: list[str] = []
    for src, dst in PAIRS:
        if not _present(src):
            errors.append(f"{src}: source missing (remove the pair or restore the file)")
            continue
        if not _present(dst):
            errors.append(f"{dst}: missing twin of {src} — {_FIX}")
            continue
        if _content(src) != _content(dst):
            errors.append(f"{dst}: differs from {src} (AGENTS.md is the source) — {_FIX}")
    return errors


def _skills_tree(prefix: str) -> set[str]:
    return {p[len(f"{prefix}/"):] for p in _INDEXED if p.startswith(f"{prefix}/")}


def check_skills_mirror() -> list[str]:
    src_files = _skills_tree(SKILLS_SOURCE)
    if not src_files:
        return [f"{SKILLS_SOURCE}: skills source missing"]
    dst_files = _skills_tree(SKILLS_MIRROR)
    errors: list[str] = []
    for extra in sorted(dst_files - src_files):
        errors.append(f"{SKILLS_MIRROR}/{extra}: not in {SKILLS_SOURCE} (stale mirror file) — {_FIX}")
    for missing in sorted(src_files - dst_files):
        errors.append(f"{SKILLS_MIRROR}/{missing}: missing mirror of {SKILLS_SOURCE}/{missing} — {_FIX}")
    for rel in sorted(src_files & dst_files):
        if _content(f"{SKILLS_SOURCE}/{rel}") != _content(f"{SKILLS_MIRROR}/{rel}"):
            errors.append(f"{SKILLS_MIRROR}/{rel}: differs from {SKILLS_SOURCE}/{rel} — {_FIX}")
    return errors


def main() -> int:
    errors = check_pairs() + check_skills_mirror()
    for e in errors:
        print(f"error: {e}", file=sys.stderr)
    print(f"agents-sync: {'FAIL' if errors else 'PASS'} ({len(errors)} error(s), {len(PAIRS)} pair(s))")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
