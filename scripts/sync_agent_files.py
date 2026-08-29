#!/usr/bin/env python3
"""Fixer: rebuild CLAUDE.md twins from AGENTS.md sources and .claude/skills from .agents/skills.

The mirror direction is fixed (AGENTS.md / .agents/skills are the sources); this
script overwrites the copies unconditionally. The gate is scripts/sync-agents.py.

Usage:
    python scripts/sync_agent_files.py
"""

from __future__ import annotations

import importlib.util
import shutil
import sys
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "sync_agents_gate", Path(__file__).resolve().parent / "sync-agents.py"
)
assert _spec is not None and _spec.loader is not None
_gate = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_gate)

PAIRS, REPO_ROOT = _gate.PAIRS, _gate.REPO_ROOT
SKILLS_SOURCE, SKILLS_MIRROR = _gate.SKILLS_SOURCE, _gate.SKILLS_MIRROR


def main() -> int:
    for src, dst in PAIRS:
        source, target = REPO_ROOT / src, REPO_ROOT / dst
        if not source.exists():
            print(f"skip: {src} (source missing)", file=sys.stderr)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        print(f"copied {src} -> {dst}")

    if SKILLS_SOURCE.exists():
        if SKILLS_MIRROR.exists():
            shutil.rmtree(SKILLS_MIRROR)
        shutil.copytree(SKILLS_SOURCE, SKILLS_MIRROR)
        print("rebuilt .claude/skills from .agents/skills")
    else:
        print("skip: .agents/skills (source missing)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
