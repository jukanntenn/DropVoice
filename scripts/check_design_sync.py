#!/usr/bin/env python3
"""Verify packages/ui/src/tokens/theme.css matches DESIGN.md.

Re-exports the CSS tokens (same command as `pnpm design:export:css`) and
fails if the generated file changed — the same gate CI's old design-sync job
provided, now in prek.

Fix when this fails:
    pnpm design:sync
    git add packages/ui/src/tokens/theme.css

Usage:
    python scripts/check_design_sync.py
"""

import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
THEME_CSS = "packages/ui/src/tokens/theme.css"


def main() -> int:
    # Windows: pnpm is a .CMD shim — resolve the real path so CreateProcess finds it.
    pnpm = shutil.which("pnpm")
    if pnpm is None:
        print("[design-sync] pnpm not found on PATH", file=sys.stderr)
        return 1
    export = subprocess.run(
        [pnpm, "design:export:css"], cwd=REPO_ROOT, capture_output=True, text=True
    )
    if export.returncode != 0:
        print("[design-sync] token export failed:", file=sys.stderr)
        print(export.stdout + export.stderr, file=sys.stderr)
        return 1

    diff = subprocess.run(
        ["git", "diff", "--exit-code", "--", THEME_CSS],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if diff.returncode != 0:
        print(
            f"[design-sync] {THEME_CSS} is out of sync with DESIGN.md.\n"
            f"Fix: pnpm design:sync && git add {THEME_CSS}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
