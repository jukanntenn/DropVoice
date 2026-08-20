#!/usr/bin/env python3
"""ZCode Stop adapter — thin wrapper over prek.

Runs the prek `lint` group (read-only gates: oxlint / tsc / clippy) on all
files and blocks the stop with diagnostics when it fails. Note: ZCode caps
Stop blocks at 3 attempts — after that the prek pre-commit gate and CI are
the remaining safety nets. The lint definitions live exclusively in
prek.toml — this script contains no lint logic.
"""

import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

REASON = """Lint errors must be fixed before finishing.

Diagnostics:
<lint_output>
{diagnostics}
</lint_output>

Re-run the failing linter(s) and verify they exit 0 before finishing. This gate
fires once per turn; if you stop again with errors remaining they slip through to CI.
"""


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return

    if payload.get("stop_hook_active") or payload.get("stopHookActive"):
        return

    result = subprocess.run(
        ["prek", "run", "--group", "lint", "--all-files"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        diagnostics = (result.stdout + "\n" + result.stderr).strip()
        print(
            json.dumps(
                {
                    "decision": "block",
                    "reason": REASON.format(diagnostics=diagnostics[-4000:]),
                }
            )
        )


if __name__ == "__main__":
    main()
