#!/usr/bin/env python3
"""Verify pnpm-lock.yaml and Cargo.lock are up to date with their manifests.

Runs the package managers' resolution in lockfile-only / no-build mode, then
checks git for any lockfile modification (worktree vs index). A dirty diff
means someone changed package.json / Cargo.toml without committing the
refreshed lockfile — the exact drift that `pnpm install --frozen-lockfile`
(CI) would later reject.

Fix when this fails:
    pnpm install --lockfile-only
    cargo metadata --format-version 1        (any cargo command refreshes it)
    git add pnpm-lock.yaml Cargo.lock

Usage:
    python scripts/check_lockfiles.py
"""

import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
LOCKS = ["pnpm-lock.yaml", "Cargo.lock"]


def run(cmd):
    # Windows: pnpm is a .CMD shim — resolve the real path so CreateProcess finds it.
    executable = shutil.which(cmd[0])
    if executable is None:
        print(f"[lockfiles] command not found on PATH: {cmd[0]}", file=sys.stderr)
        sys.exit(1)
    return subprocess.run([executable, *cmd[1:]], cwd=REPO_ROOT, capture_output=True, text=True)


def main() -> int:
    failed = False

    r = run(["pnpm", "install", "--lockfile-only", "--ignore-scripts"])
    if r.returncode != 0:
        print("[lockfiles] pnpm install --lockfile-only failed:", file=sys.stderr)
        print(r.stdout + r.stderr, file=sys.stderr)
        failed = True

    r = run(["cargo", "metadata", "--format-version", "1"])
    if r.returncode != 0:
        print("[lockfiles] cargo metadata failed:", file=sys.stderr)
        print(r.stdout + r.stderr, file=sys.stderr)
        failed = True

    if not failed:
        r = run(["git", "diff", "--exit-code", "--", *LOCKS])
        if r.returncode != 0:
            print(
                "[lockfiles] lock files are out of date with their manifests.\n"
                "Refresh and commit them:\n"
                "    pnpm install --lockfile-only\n"
                "    cargo metadata --format-version 1\n"
                "    git add pnpm-lock.yaml Cargo.lock",
                file=sys.stderr,
            )
            failed = True

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
