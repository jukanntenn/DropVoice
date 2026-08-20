#!/usr/bin/env python3
"""ZCode PostToolUse adapter — thin wrapper over prek.

Runs the prek `format` group (prettier/oxlint --fix/cargo fmt/builtin fixers)
on the edited file only. Never blocks the agent: exit codes 0 (clean) and
1 (formatter modified files) are both acceptable; anything else is surfaced
on stderr as feedback.

ZCode's payload uses camelCase keys (toolInput.file_path). The formatter
definitions live exclusively in prek.toml — this script contains no
lint/format logic.
"""

import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return

    tool_input = payload.get("toolInput") or payload.get("tool_input") or {}
    file_path = tool_input.get("file_path") or tool_input.get("filePath")
    if not file_path or not os.path.isfile(file_path):
        return

    result = subprocess.run(
        ["prek", "run", "--group", "format", "--files", file_path],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    # exit 0 = clean, 1 = files modified (expected for a formatter);
    # surface only real errors.
    if result.returncode not in (0, 1):
        print(f"[zcode-post-tool-use] prek format exited {result.returncode}", file=sys.stderr)
        if result.stdout:
            print(result.stdout[-2000:], file=sys.stderr)
        if result.stderr:
            print(result.stderr[-2000:], file=sys.stderr)


if __name__ == "__main__":
    main()
