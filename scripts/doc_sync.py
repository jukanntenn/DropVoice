#!/usr/bin/env python3
"""Aggregate documentation-gate runner — the single entry point for all Markdown gates.

Each gate lives in its own script and is independently runnable; this runner
executes them in order and reports one line per gate. With file arguments the
gates restrict themselves to those paths (prek passes staged files); with none
they run over their full scope (CI reaches this via `prek run --all-files`).

Usage:
    python scripts/doc_sync.py            # full corpus
    python scripts/doc_sync.py <files>    # restrict gates to these files
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import verify_dv_rfc_format  # noqa: E402

GATES = [
    ("verify_dv_rfc_format", verify_dv_rfc_format),
]


def main(argv: list[str]) -> int:
    files = argv[1:] or None
    failed = False
    for name, gate in GATES:
        errors = gate.run(files)
        for e in errors:
            print(f"error: {name}: {e}")
        print(f"{name}: {'FAIL' if errors else 'PASS'} ({len(errors)} error(s))")
        failed = failed or bool(errors)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
