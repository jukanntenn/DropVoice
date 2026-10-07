# RFC: SSE keepalive cadence for aggressive proxy middleboxes

Status: proposed

English | [中文](2026-10-03-sse-keepalive-cadence-for-proxy-middleboxes.zh.md)

## Problem

The desktop maintains one SSE long connection (`GET /api/devices/{id}/webrtc/events`) to receive incoming offers. The 15s ping cadence (spec §4.6, sized for Cloudflare's 125s Proxy Read Timeout) assumes the middleboxes on the client's path tolerate ≥15s of silence. Production disproved that assumption: a desktop behind an office egress that rides a commercial proxy chain had every events stream canceled **~5s after the last byte flowed** (container Caddy logged `aborting with incomplete response: reading: context canceled`; streams died +5s after headers when no event raced in, and +5s after the last offer event otherwise — a rolling 5s idle rule). The client's own read timeout is 45s, Cloudflare's floor is 100s+, and the server has no such cap, so the killer is the proxy chain itself.

Consequences: the desktop's subscription churns (subscribe → ~5s gap → resubscribe), any phone offer landing in a gap is refused 503 ("answerer offline"), offer bursts trip the per-IP rate limit (429), and the mobile UI shows the pairing as connected-then-immediately-disconnected. Mainland users behind corporate proxies are a first-class audience (zh locale), so this is a product-level defect, not a one-office quirk.

## Proposal

Close the idle windows instead of assuming them away:

- Ping cadence 15s → **3s** (`SSE_PING_INTERVAL`): 2s of margin under the observed 5s rule; ~15 B/s per connected desktop at ~45 B per ping. Both the named `ping` event and axum's `KeepAlive` comment stream now derive from the one constant.
- **Emit the first ping at connect** (t=0) instead of silently consuming tokio's immediate first tick: middleboxes that gauge idleness from first body byte see data the instant the stream opens; previously the stream was headers-only for a full interval.

Cloudflare's 125s and the staging openresty 300s caps remain satisfied with wide margin; the 240s graceful lifetime + `retry:` reconnect protocol is unchanged.

## Alternatives considered

**Keep 15s and tell affected users to bypass the proxy for our domains.** It lost because we don't control our users' networks — a voice-input tool must work behind whatever corporate egress a Chinese office runs, and "fix your proxy" is not a support answer at trial scale.

**WebSocket instead of SSE.** It lost on scope: the signaling protocol, ticket auth, reconnect semantics and the mobile long-poll counterpart are all designed around HTTP SSE, and a rolling 5s idle rule kills an idle WebSocket just as dead — the transport is not the problem, the silence is.

**Ping every 1s.** It lost on cost discipline: doubling the safety margin over 3s buys nothing observed, while per-desktop signaling traffic — through Cloudflare, onto a 3 Mbps origin — should stay as small as the evidence supports.

## Acceptance criteria

- A desktop behind the production proxy chain keeps a single events stream alive across ping boundaries (no resubscribe churn in server logs; `aborting with incomplete response` disappears).
- The first bytes of an events response leave the server immediately on connect (a `curl -N` sees the first `ping` before any interval elapses).
- Phone pairing during an established stream no longer produces 503 offer rejections attributable to subscriber gaps.
- The openapi description and the embedded cadence stay in sync (regenerated in the same commit).

## Risks

- A middlebox with an idle rule stricter than 3s would still kill the stream; the cadence is tuned to observed evidence (5s), not a guarantee — the desktop's 1s-hint reconnect keeps the symptom to a bounded blip rather than a deadlock.
- 5× the ping rate is 5× the signaling bytes; at ~15 B/s per desktop it is negligible at trial scale but is a line item to revisit if connection counts grow by orders of magnitude.
