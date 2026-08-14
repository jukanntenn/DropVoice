#!/usr/bin/env python3
"""Deployment smoke test for a deployed dropvoice pairing-server.

Runs against any environment (local acceptance container, staging via
dropvoice.bytehome.fun, future production) from the dev machine. Pure stdlib
(urllib) — zero dependencies. Checks the edges the container healthcheck
cannot see:

  1. /health  -> 200 + schema_version (migrations applied with this image)
  2. PWA      -> index.html + service worker served (PWA baked into image)
  3. Round trip -> POST /api/devices (rate limit + DB + token issue)
                    -> PUT /api/devices/{id}/status with Bearer (auth + cache)

Smoke registers a throwaway device per run (random UUID); stale records may
accumulate in dogfooding environments — acceptable, cleaned by the periodic
offline marking.

Usage:
  python scripts/smoke.py https://dropvoice.bytehome.fun
  python scripts/smoke.py https://localhost:4443 --insecure   # local acceptance
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
    parser.add_argument("url", help="base URL, e.g. https://dropvoice.bytehome.fun")
    parser.add_argument(
        "--insecure",
        action="store_true",
        help="skip TLS verification (local acceptance: self-signed certs)",
    )
    parser.add_argument("--name", default="smoke-test", help="device_name for the round trip")
    args = parser.parse_args()

    base = args.url.rstrip("/")
    ctx = ssl.create_default_context()
    if args.insecure:
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE

    checks = []

    def request(method: str, path: str, body=None, headers=None, want=(200,)):
        url = f"{base}{path}"
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(
            url, data=data, method=method, headers={"User-Agent": UA, **(headers or {})}
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
            except urllib.error.URLError:
                raise

    def check(name, fn):
        try:
            detail = fn()
            print(f"  PASS  {name}")
            return True
        except Exception as e:  # noqa: BLE001 - report every failure detail
            print(f"  FAIL  {name}: {e}")
            return False

    print(f"[smoke] {base}")

    def health():
        status, _, raw = request("GET", "/health")
        body = json.loads(raw)
        assert status == 200, f"status={status}"
        assert body["status"] == "ok", f"status field={body!r}"
        assert body["schema_version"] > 0, f"schema_version={body.get('schema_version')!r}"
        print(f"        schema_version={body['schema_version']}")
        return True

    def pwa_index():
        status, headers, raw = request("GET", "/")
        assert status == 200, f"status={status}"
        ctype = headers.get("Content-Type", "")
        assert "text/html" in ctype, f"Content-Type={ctype}"
        assert b"root" in raw, "index.html missing app root element"
        return True

    def pwa_sw():
        status, _, _ = request("GET", "/sw.js")
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
            "POST", "/api/devices", payload,
            headers={"Content-Type": "application/json"},
            want=(200, 201),
        )
        body = json.loads(raw)
        assert status in (200, 201), f"status={status} body={body}"
        token = body["pairing_token"]
        assert token, "no pairing_token in response"

        time.sleep(RATE_LIMIT_WAIT_SECS)  # stay under the rate limit
        status, _, raw = request(
            "PUT", f"/api/devices/{device_id}/status",
            {"address": {"ip": "127.0.0.1", "port": 38425}},
            headers={"Content-Type": "application/json", "Authorization": f"Bearer {token}"},
        )
        assert status == 200, f"status={status} body={raw[:300]!r}"
        return True

    checks.append(("health + schema_version", health))
    checks.append(("PWA index", pwa_index))
    checks.append(("PWA service worker", pwa_sw))
    checks.append(("register -> status round trip", round_trip))

    passed = sum(1 for _, fn in checks if check(_, fn))
    print(f"[smoke] {passed}/{len(checks)} checks passed")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    sys.exit(main())
