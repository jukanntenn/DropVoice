#!/usr/bin/env python3
"""PreToolUse hook: block direct edits to generated/protected files.

Reads tool_input.file_path from stdin JSON. If it targets a protected file,
exit 2 (block) with guidance on stderr. Otherwise exit 0 (allow).

Protected files:
  - packages/ui/src/tokens/theme.css  (generated from DESIGN.md)
  - Cargo.lock                        (toolchain-pinned, do not hand-edit)
"""
import json
import sys

PROTECTED = {
    "packages/ui/src/tokens/theme.css": (
        "theme.css is generated from DESIGN.md. Edit DESIGN.md then run: pnpm design:sync"
    ),
    "Cargo.lock": (
        "Cargo.lock is toolchain-pinned. Do not hand-edit "
        "(sqlx is pinned to 0.8 for rustc 1.93). Use 'cargo update <crate>' for unrelated deps."
    ),
}


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0  # can't parse input; allow (don't block on malformed hook data)

    file_path = data.get("tool_input", {}).get("file_path", "")
    if not file_path:
        return 0

    # Normalize to forward slashes for matching.
    normalized = file_path.replace("\\", "/")
    for protected_path, message in PROTECTED.items():
        if normalized == protected_path or normalized.endswith("/" + protected_path):
            print(f"BLOCKED: {message}", file=sys.stderr)
            return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
