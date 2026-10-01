# DV-RFC: Deterministic three-host public topology (landing / app / ps)

Status: implemented

## Problem

One small VPS behind Cloudflare must publicly serve three different content types — a marketing landing, an installable phone PWA, and a pure JSON signaling API. The PWA must reach the API with no CORS machinery at all; the desktop's Rust HTTP legs need one stable API host; rate limiting must recover the true client IP through the CDN; releases must not let frontend and backend versions drift; and the staging dogfooding environment must exercise exactly the production shape — otherwise it verifies nothing.

## Decision

Split the public surface into **three deterministic hosts routed by Host header — one URL, one content, no UA sniffing**:

- `<domain>` (apex) — landing, served from `landing-dist`
- `app.<domain>` — phone PWA from `mobile-dist`, with `/api` reverse-proxied same-origin
- `ps.<domain>` — pure API for the desktop's Rust legs (heartbeat + SSE)

Production is `dropvoice.online` under Cloudflare; staging maps 1:1 (`*.dropvoice.bytehome.fun`) so flipping three value groups — domains, TLS profile, trusted-proxy CIDRs — moves staging behind Cloudflare unchanged. One template, `devops/ansible/templates/Caddyfile.j2`, renders both environments (staging HTTP with tunnel-terminated TLS, production TLS with a Cloudflare Origin CA cert).

Both frontends (mobile PWA + landing) are baked into the pairing-server image: the image is the single self-contained release artifact, and frontend versions cannot drift from the backend.

The real-client-IP chain is server-level `trusted_proxies` (upstream CIDRs) + `trusted_proxies_strict` (right-to-left trust-chain parse) + `header_up X-Forwarded-For {client_ip}`: untrusted direct peers get a spoofed XFF overwritten, so axum's leftmost-XFF parse always yields the true client — the rate-limit key and the access log agree by construction.

That the PWA must be same-origin at all is a consequence of the [2026-08-14 WebRTC DataChannel record](2026-08-14-webrtc-datachannel-public-pairing-server.md) ("the phone remains a same-origin PWA"); this record decides the host layout that satisfies it. Mechanism facts — which file renders what, ports, cache headers — live in [docs/architecture.md](../../../docs/architecture.md) and [docs/development.md](../../../docs/development.md).

## Alternatives considered

**The incumbent: one public origin (`api.dropvoice.app`) serving PWA + API on the same host.** It lost: a consumer PWA lived at a hostname that says "api", with no room for a landing page on the same artifact; and CORS origin lists (public URL + tauri webview origins) had to be derived and maintained in every environment even though, once signaling moved into the Rust process, no cross-origin caller existed anymore.

**One host with UA sniffing to pick between PWA, API, and landing content.** It lost: routing must be deterministic — one URL always yields one content. UA-dependent responses poison edge caches (Cloudflare caches per URL), make support investigations non-reproducible, and put a silent wrong-variant failure mode in front of every phone.

**Serve the landing from a separate static host; keep the image PWA-only.** It lost: the landing advertises the PWA entry URL, so a separately deployed landing can disagree with the backend about where the phone should go; and it reintroduces per-release operator uploads, the exact staleness class the baked-in PWA had already eliminated.

**Keep two Caddyfile artifacts — an HTTP copy for staging, a production-only TLS template.** It lost: production routing lived in a file staging never rendered, so staging verified a different Caddyfile than the one production ran — the failure staging exists to catch. The two renderings also had to be hand-mirrored on every routing change.

## Consequences

- `cors_origins` is empty in every standard environment (dev tasks, acceptance, staging, production); it survives only as an escape hatch for special experiments. The desktop's default server URL is `https://ps.dropvoice.online` (`network.pairing_server_url`).
- HTML is never edge-cached (`no-cache`), hashed assets are `immutable` — per-URL determinism is what makes Cloudflare edge caching safe at all on a 3 Mbps origin.
- Cloudflare panel prerequisites (three orange-cloud records, Full strict, Origin Rule port rewrite, rate-limit/WAF/cache rules) are part of the production runbook, not of the repo.
- Staging exercises the production Caddyfile template itself, differing only in values; the post-deploy smoke check probes all three hosts (`--app-url` / `--api-url`).
- A release ships one image carrying backend + PWA + landing; the landing's `pwaUrl` and the backend can no longer disagree.
