# RFC: WebRTC DataChannel P2P with a public pairing-server rendezvous

Status: implemented

English | [中文](2026-08-14-webrtc-datachannel-public-pairing-server.zh.md)

## Problem

A phone must deliver voice-to-text to a PC, and the PC must accept an inbound connection from it. The original transport — a local axum HTTP/WS server on the desktop plus UDP discovery — required both devices to sit on the same LAN segment with permissive firewalls, broke whenever the desktop's local server lifecycle tangled with tray-hide or system sleep, and could not reach a desktop behind NAT from a phone on cellular. Pairing also had to be a QR scan, not an IP-typing exercise, and the text flowing keystroke-by-keystroke should never live on a third-party server.

## Decision

Text flows over a **WebRTC DataChannel, peer-to-peer**: the phone is the offerer, the desktop is the answerer. Neither side's data transits any server; the desktop binds no local data port at all.

A **public pairing-server** (axum + SQLite, deployed at `ps.<domain>`) is a pure rendezvous — signaling only, never data:

1. The desktop's heartbeat registers the device (credential-controlled idempotent upsert; re-registering an existing device requires the current Bearer token) and receives a pairing token. Device tokens are stored as SHA-256 hashes only, never echoed back; a desktop that gets 401 resets its device identity (re-scan required).
2. The phone scans a QR (`dropvoice://pair?code=&device=&name=`) and POSTs a WebRTC offer to the pairing-server.
3. The pairing-server relays the offer to the desktop's SSE subscription. The desktop exchanges its long-lived Bearer token for a one-time 60-second ticket before opening the event stream, so long-lived tokens never appear in URLs or access logs. The desktop's **Rust signaling supervision task** consumes offers: verifies the credential against the local pairing code/token, has the webrtc-rs answerer (one `RTCPeerConnection` per offer) produce an answer, and posts it back. SSE liveness is detected deterministically via a reqwest chunk-level read timeout, so a half-open connection is rebuilt entirely on Rust clocks (tokio timers fire immediately after sleep/wake).
4. The phone long-polls the answer; on `accepted` it sets the remote description and the DataChannel opens. Text arrives as `{type:"text"}` messages; the desktop queues them into Enigo keyboard injection. The desktop issues a connection `token` over the channel, which the phone uses to reconnect without re-pairing.

Both desktop legs (heartbeat registration + SSE subscription) live in the desktop's Rust process and resolve the server URL through the single config source (`network.pairing_server_url` in `config.toml`, env override for dev orchestration only). The webview is a pure view and never participates in signaling — WebView2 freezes hidden-page JS timers and network, which kills page-side signaling across tray-hide and sleep/wake.

## Alternatives considered

**Keep the local HTTP/WS server + UDP discovery.** The incumbent at the time. It lost: same-L2-segment and firewall requirements defeat roaming phones and NAT; the local server's lifecycle was coupled to window/tray state; and there is no path from "phone on cellular" to "desktop at home" at all without a public rendezvous.

**Relay the text through the cloud server.** Simple, works through any NAT. It lost: every keystroke would transit and rest on a third party (unacceptable for an input tool), server bandwidth scales with usage instead of with pairings, and latency gains nothing over a direct DataChannel once signaling is solved.

**A generic message broker (MQTT, or a hosted realtime service) for signaling.** It lost: the rendezvous needs a device registry, token issuance and verification, rate limiting, and health/smoke surfaces anyway — a purpose-built axum server provides registry and signaling in one place with no new external dependency.

**WebSocket instead of SSE for the desktop signaling leg.** It lost: SSE over HTTPS reuses the heartbeat leg's HTTP client and Bearer auth unchanged, and a chunk-level read timeout on the response stream gives deterministic half-open detection; a ws client would add a dependency and its own keepalive machinery for no capability gain.

## Consequences

- The pairing-server stays a pure rendezvous: it relays offers/answers and device registry state, and nothing else; its load is independent of typing volume. Data-plane evolution (e.g. binary frames) never touches it.
- The desktop binds no local data port; tray-hide and sleep/wake no longer interrupt connectivity, because both long-lived legs (heartbeat, SSE) live in the Rust process and rebuild on tokio timers.
- Long-lived credentials stay out of URLs and logs (one-time SSE tickets); tokens are hash-only at rest; token reuse attempts 401 and reset the desktop identity.
- The public rendezvous surface is bounded by construction: devices unseen for 30 days are garbage-collected, the in-process rate limiter caps its own key set (overflow resets rather than grows), and the rate-exempt endpoints (SSE, answer long-poll) sit under global concurrency ceilings — 429 beyond them, never unbounded resource use.
- The phone remains a same-origin PWA (dev: Vite proxy; deployed: app host reverse proxy — the host layout is its own record, [2026-08-30](2026-08-30-deterministic-three-host-public-topology.md)) — no build-time API base exists, and the DataChannel carries the actual traffic.
- Any future transport change that moves data off the P2P channel must supersede this record, not amend it.
