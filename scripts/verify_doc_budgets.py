#!/usr/bin/env python3
"""Gate: word-count ceilings for AGENTS.md files (scripts/doc_budgets.manifest.json).

Budgets are guardrails, not reduction targets. On red: relocate to the owning
tier, condense, raise the ceiling last with a justified manifest diff in the
same change. A manifest entry whose file is missing fails the gate, so a rename
cannot silently orphan its budget. Check-only; never writes.

Usage:
    python scripts/verify_doc_budgets.py [files...]   # no args = all manifest entries
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST = REPO_ROOT / "scripts" / "doc_budgets.manifest.json"


def run(files: list[str] | None) -> list[str]:
    budgets: dict[str, int] = json.loads(MANIFEST.read_text(encoding="utf-8"))
    errors: list[str] = []

    # Orphan detection always runs: a budgeted path that vanished is a drift
    # signal even when only a subset of files was passed in.
    for path in budgets:
        if not (REPO_ROOT / path).exists():
            errors.append(f"{path}: budgeted file is missing (rename? update doc_budgets.manifest.json)")

    targets = [f.replace("\\", "/") for f in files] if files else list(budgets)
    for path in targets:
        if path not in budgets:
            continue
        file = REPO_ROOT / path
        if not file.exists():
            continue  # already reported as an orphan above
        words = len(file.read_text(encoding="utf-8").split())
        ceiling = budgets[path]
        if words > ceiling:
            errors.append(
                f"{path}: {words} words exceed the {ceiling}-word ceiling "
                "(relocate -> condense -> raise the ceiling last, with a justified manifest diff)"
            )
    return errors


def main(argv: list[str]) -> int:
    errors = run(argv[1:] or None)
    for e in errors:
        print(f"error: {e}")
    print(f"verify_doc_budgets: {'FAIL' if errors else 'PASS'} ({len(errors)} error(s))")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
