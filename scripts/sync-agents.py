#!/usr/bin/env python3
"""Check that AGENTS.md and CLAUDE.md are byte-for-byte identical.

AGENTS.md is the source of truth. CLAUDE.md must be an exact copy.
This script only CHECKS; it never writes. If the two files differ,
exit 1 with a message telling the developer to copy AGENTS.md to CLAUDE.md.

Usage:
    python scripts/sync-agents.py
"""
import filecmp
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
AGENTS = REPO_ROOT / "AGENTS.md"
CLAUDE = REPO_ROOT / "CLAUDE.md"


def main() -> int:
    if not AGENTS.exists():
        print("ERROR: AGENTS.md not found", file=sys.stderr)
        return 1
    if not CLAUDE.exists():
        print("ERROR: CLAUDE.md not found", file=sys.stderr)
        return 1
    if not filecmp.cmp(AGENTS, CLAUDE, shallow=False):
        print(
            "ERROR: AGENTS.md and CLAUDE.md are out of sync.\n"
            "AGENTS.md is the source of truth.\n"
            "Fix: copy AGENTS.md to CLAUDE.md (they must be byte-for-byte identical):\n"
            "    cp AGENTS.md CLAUDE.md",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
