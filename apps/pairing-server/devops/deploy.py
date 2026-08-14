#!/usr/bin/env python3
"""Deploy dropvoice pairing-server from any host (Windows dev machine included).

Windows cannot run ansible natively (ansible-core requires POSIX fork/sh), so
the playbook runs inside a throwaway Linux container (docker run --rm). The
runner image is built on first use and cached afterwards; the repo is
bind-mounted so playbook edits need no image rebuild.

Mounts:
  repo root        -> /workspace (playbooks/templates/vaults)
  ~/.ssh           -> /ssh      (entrypoint copies keys, chmod 600)
  ~/.ansible-vault -> /vault    (entrypoint copies password file)

Environments:
  staging     — fn @ 192.168.5.200 (dropvoice.bytehome.fun, tunnel-terminated
                HTTPS). Dogfooding target; the gate before production.
  production  — future public SaaS (placeholder VPS).

Deploys the `main` image tag by default (latest main branch). Pin a specific
build — or roll back — with --tag <immutable-tag> (e.g. `git describe`).

Examples:
  python apps/pairing-server/devops/deploy.py staging
  python apps/pairing-server/devops/deploy.py staging --tag v0.3.0
  python apps/pairing-server/devops/deploy.py staging --check --diff
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    # Windows console: force UTF-8 so Chinese output never raises
    # UnicodeEncodeError (cp936 default stdout).
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8")
        except (AttributeError, ValueError):
            pass

    parser = argparse.ArgumentParser(
        description="Deploy dropvoice pairing-server via a dockerized ansible runner"
    )
    parser.add_argument(
        "env",
        nargs="?",
        choices=["staging", "production"],
        default="staging",
        help="target environment (default: staging)",
    )
    parser.add_argument(
        "--tag",
        metavar="IMAGE_TAG",
        help="immutable image tag to deploy, e.g. v0.3.0 or v0.2.0-3-gabc123 "
        "(default: group_vars image_tag, i.e. `main`)",
    )
    parser.add_argument(
        "--check", action="store_true", help="ansible dry-run (no changes)"
    )
    parser.add_argument(
        "--diff", action="store_true", help="show template/config diffs"
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent.parent
    runner_dir = repo_root / "apps" / "pairing-server" / "devops" / "runner"
    runner_image = "dropvoice-ansible-runner"

    vault_dir = Path.home() / ".ansible-vault"
    vault_file = vault_dir / f"dropvoice-{args.env}.pwd"
    if not vault_file.exists():
        print(
            f"[deploy] vault password file not found: {vault_file}\n"
            "  create it first: write the environment's vault password into\n"
            f"  {vault_dir}\\dropvoice-{args.env}.pwd"
        )
        return 1
    ssh_dir = Path.home() / ".ssh"
    if not ssh_dir.exists():
        print(f"[deploy] SSH directory not found: {ssh_dir}")
        return 1

    print("[deploy] Building runner image (cached after first run)...")
    rc = subprocess.run(
        ["docker", "build", "-q", "-t", runner_image, str(runner_dir)],
        check=False,
    ).returncode
    if rc != 0:
        print("[deploy] runner image build failed", file=sys.stderr)
        return rc

    cmd = [
        "docker",
        "run",
        "--rm",
        "-v",
        f"{repo_root}:/workspace",
        "-v",
        f"{ssh_dir}:/ssh:ro",
        "-v",
        f"{vault_dir}:/vault:ro",
        "-e",
        "ANSIBLE_CONFIG=/workspace/apps/pairing-server/devops/ansible/ansible.cfg",
        "-w",
        "/workspace/apps/pairing-server/devops/ansible",
        runner_image,
        args.env,
    ]
    if args.tag:
        cmd += ["--extra-vars", f"image_tag={args.tag}"]
    if args.check:
        cmd += ["--check"]
    if args.diff:
        cmd += ["--diff"]

    print(f"[deploy] Running ansible-playbook -l {args.env} in container...")
    return subprocess.run(cmd, check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
