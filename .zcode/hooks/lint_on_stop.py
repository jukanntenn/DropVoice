#!/usr/bin/env python3
"""Stop hook: run full lint before the agent ends the session.

Runs oxlint (frontend) + cargo clippy --workspace (Rust). If either fails,
exit 2 (block) with the error output on stderr, forcing the agent to fix
issues before it can stop. If all pass, exit 0.

Note: ZCode limits Stop blocks to 3 attempts. If the agent can't fix all
lint errors in 3 rounds, it will end anyway - the precommit hook is the
final safety net.
"""
import os
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    env = os.environ.get("CLAUDE_PROJECT_DIR") or os.environ.get("ZCODE_PROJECT_DIR")
    if env:
        return Path(env)
    try:
        out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
        return Path(out.strip())
    except Exception:
        return Path.cwd()


def run(cmd, cwd: Path):
    """Returns (exit_code, combined_output).

    Exit code -1 means the tool itself was not found (skip, don't block).
    """
    try:
        result = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True, timeout=240)
        output = result.stdout + result.stderr
        return result.returncode, output
    except FileNotFoundError:
        return -1, "SKIP: " + cmd[0] + " not found in PATH"
    except subprocess.TimeoutExpired:
        return 1, "TIMEOUT: " + " ".join(cmd)
    except Exception as e:
        return 1, "ERROR running " + " ".join(cmd) + ": " + str(e)


def main() -> int:
    root = repo_root()
    failures = []

    # Frontend lint
    code, out = run(["pnpm", "oxlint"], root)
    if code > 0:
        failures.append(("oxlint", out))
    elif code == -1:
        print("WARN: " + out, file=sys.stderr)

    # Rust lint (all workspace crates)
    code, out = run(["cargo", "clippy", "--workspace", "--", "-D", "warnings"], root)
    if code > 0:
        failures.append(("cargo clippy", out))
    elif code == -1:
        print("WARN: " + out, file=sys.stderr)

    if failures:
        print("Lint check FAILED - fix these before ending:", file=sys.stderr)
        for name, out in failures:
            print("\n--- " + name + " ---", file=sys.stderr)
            # Truncate to last 3000 chars to avoid huge output.
            print(out[-3000:], file=sys.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
