#!/usr/bin/env python3
"""Deployment smoke test for a deployed dropvoice pairing-server.

Runs against any environment (local acceptance container, staging, production)
from the dev machine. Pure stdlib (urllib) — zero dependencies. Checks the
edges the container healthcheck cannot see:

  1. /health  -> 200 + schema_version (migrations applied with this image)
                + version == --expect-version (catches a healthy container
                  still serving an OLD image; APP_VERSION is baked at build)
  2. PWA      -> index.html + service worker served (PWA baked into image)
  3. Round trip -> POST /api/devices (rate limit + DB + token issue)
                    -> PUT /api/devices/{id}/status with Bearer (auth + cache)

Three-host topology (production/staging): pass --app-url (PWA host) and
--api-url (pure API host); the positional URL is then the landing host and an
extra landing check runs. Without the flags the single base URL serves
everything (local acceptance container).

Smoke registers a throwaway device per run (random UUID); stale records may
accumulate in dogfooding environments — acceptable, cleaned by the periodic
device GC (30d retention).

Usage:
  python scripts/smoke.py http://localhost:8080                # local acceptance (pnpm accept:up)
  python scripts/smoke.py https://dropvoice.bytehome.fun \
      --app-url https://app.dropvoice.bytehome.fun \
      --api-url https://ps.dropvoice.bytehome.fun              # staging, three hosts
  python scripts/smoke.py https://dropvoice.online \
      --app-url https://app.dropvoice.online \
      --api-url https://ps.dropvoice.online --expect-version v0.3.0
"""

import argparse
import json
import ssl
import sys
import time
import urllib.error
import urllib.request
import uuid

RATE_LIMIT_WAIT_SECS = 1.2  # default per-IP limit is 1 req/s; keep below it
UA = "dropvoice-smoke/1.0"


def main() -> int:
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except (AttributeError, ValueError):
        pass

    parser = argparse.ArgumentParser(description="Smoke test a deployed pairing-server")
    parser.add_argument("url", help="landing (or single-host) base URL, e.g. https://dropvoice.online")
    parser.add_argument(
        "--app-url",
        default="",
        help="PWA host (three-host topology); enables the landing check",
    )
    parser.add_argument(
        "--api-url",
        default="",
        help="pure API host (three-host topology); health + round trip run there",
    )
    parser.add_argument(
        "--insecure",
        action="store_true",
        help="skip TLS verification (local acceptance: self-signed certs)",
    )
    parser.add_argument("--name", default="smoke-test", help="device_name for the round trip")
    parser.add_argument(
        "--expect-version",
        default="",
        help="expected APP_VERSION (git describe) baked into the image; "
        "mismatch means the deployment is healthy but stale",
    )
    args = parser.parse_args()

    base = args.url.rstrip("/")
    app_base = args.app_url.rstrip("/") or base
    api_base = args.api_url.rstrip("/") or base
    three_host = bool(args.app_url or args.api_url)
    ctx = ssl.create_default_context()
    if args.insecure:
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE

    checks = []

    def request(method: str, url: str, path: str, body=None, headers=None):
        full = f"{url}{path}"
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(
            full, data=data, method=method, headers={"User-Agent": UA, **(headers or {})}
        )
        for attempt in range(2):
            try:
                with urllib.request.urlopen(req, context=ctx, timeout=15) as resp:
                    return resp.status, dict(resp.headers), resp.read()
            except urllib.error.HTTPError as e:
                if e.code == 429 and attempt == 0:
                    time.sleep(RATE_LIMIT_WAIT_SECS)
                    continue
                raise

    def check(name, fn):
        try:
            detail = fn()
            print(f"  PASS  {name}")
            return True
        except Exception as e:  # noqa: BLE001 - report every failure detail
            print(f"  FAIL  {name}: {e}")
            return False

    print(f"[smoke] landing={base} app={app_base} api={api_base}")

    def landing_index():
        status, headers, raw = request("GET", base, "/")
        assert status == 200, f"status={status}"
        ctype = headers.get("Content-Type", "")
        assert "text/html" in ctype, f"Content-Type={ctype}"
        assert b"DropVoice" in raw, "landing index.html missing DropVoice marker"
        return True

    def health():
        status, _, raw = request("GET", api_base, "/health")
        body = json.loads(raw)
        assert status == 200, f"status={status}"
        assert body["status"] == "ok", f"status field={body!r}"
        assert body["schema_version"] > 0, f"schema_version={body.get('schema_version')!r}"
        print(f"        schema_version={body['schema_version']} version={body.get('version')!r}")
        if args.expect_version:
            assert body.get("version") == args.expect_version, (
                f"version={body.get('version')!r} but expected {args.expect_version!r} "
                "(healthy but stale image?)"
            )
        return True

    def pwa_index():
        status, headers, raw = request("GET", app_base, "/")
        assert status == 200, f"status={status}"
        ctype = headers.get("Content-Type", "")
        assert "text/html" in ctype, f"Content-Type={ctype}"
        assert b"root" in raw, "index.html missing app root element"
        return True

    def pwa_sw():
        status, _, _ = request("GET", app_base, "/sw.js")
        assert status == 200, f"status={status}"
        return True

    def round_trip():
        device_id = str(uuid.uuid4())
        payload = {
            "device_id": device_id,
            "platform": "desktop",
            "device_name": args.name,
            "address": {"ip": "127.0.0.1", "port": 38425},
        }
        status, _, raw = request(
            "POST", api_base, "/api/devices", payload,
            headers={"Content-Type": "application/json"},
        )
        body = json.loads(raw)
        assert status in (200, 201), f"status={status} body={body}"
        token = body["pairing_token"]
        assert token, "no pairing_token in response"

        time.sleep(RATE_LIMIT_WAIT_SECS)  # stay under the rate limit
        status, _, raw = request(
            "PUT", api_base, f"/api/devices/{device_id}/status",
            {"address": {"ip": "127.0.0.1", "port": 38425}},
            headers={"Content-Type": "application/json", "Authorization": f"Bearer {token}"},
        )
        assert status == 200, f"status={status} body={raw[:300]!r}"
        return True

    if three_host:
        checks.append(("landing index", landing_index))
    checks.append(("health + schema_version", health))
    checks.append(("PWA index", pwa_index))
    checks.append(("PWA service worker", pwa_sw))
    checks.append(("register -> status round trip", round_trip))

    passed = sum(1 for _, fn in checks if check(_, fn))
    print(f"[smoke] {passed}/{len(checks)} checks passed")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    sys.exit(main())
