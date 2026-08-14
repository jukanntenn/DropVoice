#!/usr/bin/env python3
"""PostToolUse hook: auto-format files after AI edits them.

Reads tool_input.file_path from stdin JSON. Routes by extension:
  - .ts/.tsx/.js/.jsx/.json/.css/.html/.md  -> prettier --write
  - .rs                                      -> cargo fmt (workspace)

Always exits 0 (formatting must not block the agent). Failures go to stderr
as feedback. The repo root is resolved from CLAUDE_PROJECT_DIR /
ZCODE_PROJECT_DIR environment variable, falling back to git toplevel.
"""
import json
import os
import subprocess
import sys
from pathlib import Path

PRETTIER_EXTS = {".ts", ".tsx", ".js", ".jsx", ".json", ".css", ".html", ".md"}
RUST_EXTS = {".rs"}


def repo_root() -> Path:
    env = os.environ.get("CLAUDE_PROJECT_DIR") or os.environ.get("ZCODE_PROJECT_DIR")
    if env:
        return Path(env)
    try:
        out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
        return Path(out.strip())
    except Exception:
        return Path.cwd()


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    file_path = data.get("tool_input", {}).get("file_path", "")
    if not file_path:
        return 0

    path = Path(file_path)
    ext = path.suffix.lower()
    root = repo_root()

    try:
        if ext in PRETTIER_EXTS:
            subprocess.run(
                ["pnpm", "prettier", "--write", str(path)],
                cwd=str(root),
                capture_output=True,
                text=True,
                timeout=30,
            )
        elif ext in RUST_EXTS:
            subprocess.run(
                ["cargo", "fmt"],
                cwd=str(root),
                capture_output=True,
                text=True,
                timeout=60,
            )
    except Exception as e:
        print(f"format warning: {e}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
