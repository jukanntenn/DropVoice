#!/usr/bin/env python3
"""Publish the Tauri updater manifest and artifacts to Cloudflare R2.

Runs in release.yml once a stable release exists: the GitHub release assets
(including tauri-action's latest.json) are downloaded into --release-dir
first. This script rewrites each platform URL to the R2 public domain and
uploads the referenced artifacts (immutable) plus the manifest (no-cache)
via wrangler, authenticating with the CLOUDFLARE_API_TOKEN env var.

Tag rules mirror docker-publish.yml's `latest` semantics:
  v0.2.1      -> uploads files/v0.2.1/* and moves /update/manifest.json
  v0.2.1-rc.1 -> CI never uploads (prereleases never move the manifest the
                 shipped installers poll); exits 2 so misuse is loud. The
                 --allow-prerelease escape hatch exists solely to rehearse
                 the update chain BEFORE the first stable ships (no install
                 base to surprise): run it by hand against downloaded assets.

Usage:
  python scripts/publish_updater_manifest.py --release-dir dist --tag v0.2.1 \
      --base-url https://releases.dropvoice.online [--bucket dropvoice-releases] [--dry-run]
"""

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

STABLE_TAG = re.compile(r"^v\d+\.\d+\.\d+$")
IMMUTABLE_CACHE = "public, max-age=31536000, immutable"
MANIFEST_CACHE = "no-cache"

# Windows 上 npx 是 npx.cmd，CreateProcess 不解析 PATHEXT——subprocess 直接
# 传 "npx" 会 FileNotFoundError；shutil.which 解析出真实路径，两端通用。
NPX = shutil.which("npx") or "npx"
CONTENT_TYPES = {
    ".exe": "application/octet-stream",
    ".msi": "application/octet-stream",
    ".appimage": "application/octet-stream",
    ".deb": "application/vnd.debian.binary-package",
    ".dmg": "application/x-apple-diskimage",
    ".tar.gz": "application/gzip",
    ".zip": "application/zip",
}


def content_type(name: str) -> str:
    lowered = name.lower()
    for suffix, ctype in CONTENT_TYPES.items():
        if lowered.endswith(suffix):
            return ctype
    return "application/octet-stream"


def main() -> int:
    # Windows console: force UTF-8 so Chinese output never raises
    # UnicodeEncodeError (cp936 default stdout).
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8")
        except (AttributeError, ValueError):
            pass

    parser = argparse.ArgumentParser(description="Publish Tauri updater artifacts to Cloudflare R2")
    parser.add_argument(
        "--release-dir",
        type=Path,
        required=True,
        help="directory holding the downloaded release assets (incl. latest.json)",
    )
    parser.add_argument("--tag", required=True, help="release tag, e.g. v0.2.1")
    parser.add_argument(
        "--base-url",
        required=True,
        help="R2 public base URL, e.g. https://releases.dropvoice.online",
    )
    parser.add_argument("--bucket", default="dropvoice-releases", help="R2 bucket name")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="rewrite and print the upload plan without uploading",
    )
    parser.add_argument(
        "--allow-prerelease",
        action="store_true",
        help="let a prerelease tag move /update/manifest.json — rehearsal "
        "escape hatch, only safe before any stable install base exists",
    )
    args = parser.parse_args()

    if not STABLE_TAG.match(args.tag) and not args.allow_prerelease:
        print(
            f"[r2] {args.tag} is not a stable tag (^vX.Y.Z) — the manifest never "
            "moves for a prerelease through CI; pass --allow-prerelease "
            "deliberately (pre-stable rehearsal only)"
        )
        return 2
    if not STABLE_TAG.match(args.tag):
        print(
            f"[r2] WARNING: force-publishing a PRERELEASE manifest ({args.tag}) — "
            "every installed client will be offered this version"
        )

    latest_path = args.release_dir / "latest.json"
    if not latest_path.is_file():
        print(
            f"[r2] {latest_path} not found — did tauri-action generate it "
            "(needs createUpdaterArtifacts + TAURI_SIGNING_PRIVATE_KEY)?",
            file=sys.stderr,
        )
        return 1
    manifest = json.loads(latest_path.read_text(encoding="utf-8"))
    platforms = manifest.get("platforms")
    if not isinstance(platforms, dict) or not platforms:
        print("[r2] latest.json has no platforms entries", file=sys.stderr)
        return 1

    base_url = args.base_url.rstrip("/")
    uploads: list[tuple[str, Path, str, str]] = []  # (r2_key, local_file, content_type, cache_control)
    for platform, entry in sorted(platforms.items()):
        url = entry.get("url", "")
        signature = entry.get("signature", "")
        if not url or not signature:
            print(f"[r2] platform {platform}: url/signature incomplete", file=sys.stderr)
            return 1
        filename = url.rsplit("/", 1)[-1]
        local = args.release_dir / filename
        if not local.is_file():
            print(f"[r2] platform {platform}: asset {local.name} not in --release-dir", file=sys.stderr)
            return 1
        entry["url"] = f"{base_url}/files/{args.tag}/{filename}"
        key = f"files/{args.tag}/{filename}"
        if not any(existing == key for existing, *_ in uploads):
            uploads.append((key, local, content_type(filename), IMMUTABLE_CACHE))

    manifest_path = args.release_dir / "manifest.r2.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    uploads.append(("update/manifest.json", manifest_path, "application/json", MANIFEST_CACHE))

    print(f"[r2] manifest version {manifest.get('version')} — {len(platforms)} platform(s)")
    failed = 0
    for key, local, ctype, cache in uploads:
        print(f"[r2] {'DRY-RUN ' if args.dry_run else ''}put {args.bucket}/{key} <- {local.name} ({ctype}, {cache})")
        if args.dry_run:
            continue
        rc = subprocess.run(
            [
                NPX,
                "--yes",
                "wrangler@4",
                "r2",
                "object",
                "put",
                f"{args.bucket}/{key}",
                "--file",
                str(local),
                "--content-type",
                ctype,
                "--cache-control",
                cache,
            ],
        ).returncode
        if rc != 0:
            print(f"[r2] upload failed for {key}", file=sys.stderr)
            failed = rc
            break
    if failed:
        return failed

    print(f"[r2] done — manifest live at {base_url}/update/manifest.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
