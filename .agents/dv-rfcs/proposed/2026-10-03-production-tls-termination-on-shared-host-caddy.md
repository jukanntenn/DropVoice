# DV-RFC: Production TLS termination on the shared host Caddy

Status: proposed

## Problem

Production must run on ttyo (alice @ 43.133.160.29), a shared VPS that already fronts several services through one operator-managed Caddy on :443 — Cloudflare reaches the box on standard 443 for every zone (the operator's uniform standard, kept for CDN-cache behavior). The repo's production design — the in-container Caddy terminating TLS with a Cloudflare Origin CA cert on :4443 plus a per-zone Origin Rule port rewrite — does not fit that box: it would publish a second TLS listener beside the shared one, add a nonstandard port and a per-zone rule to maintain (4443 is not a port Cloudflare proxies natively), and diverge from how every other service on the machine is fronted.

## Proposal

Terminate production TLS at the shared host Caddy (the markpost pattern): Cloudflare → ttyo:443 → host Caddy (Origin CA cert SAN `dropvoice.online` + `*.dropvoice.online`, `trusted_proxies` = Cloudflare CIDRs, `header_up X-Forwarded-For {http.request.header.CF-Connecting-IP}`) → reverse_proxy to the container published loopback-only on `127.0.0.1:8089` → the in-container Caddy keeps its HTTP three-host routing (`:8080`) → axum. The Caddyfile template drops its TLS branch entirely — one HTTP form for every environment (staging: tunnel-terminated; production: host-Caddy-terminated); the certs bind-mount and the `tls_profile` knob are deleted; production's trusted-proxy segment becomes the docker-bridge gateway. The host Caddy site blocks are operator-side config documented in the production runbook, outside this playbook. This supersedes only the termination point of the [2026-08-30 topology record](../implemented/2026-08-30-deterministic-three-host-public-topology.md) — its deterministic three-host layout, same-origin PWA/API, and real-IP discipline are unchanged.

## Alternatives considered

**Keep the shipped design (in-container TLS :4443 + Origin Rule rewrite).** It lost against the operator's uniform standard: every other zone on the account already origins on plain 443; a second TLS listener plus a per-zone port rule is one more thing to maintain for no benefit, and 4443 is not a port Cloudflare proxies natively.

**markpost's exact form (in-container TLS on :2053, a natively proxied port).** It removes the Origin Rule but still duplicates TLS and certificate handling per service on a box that already runs a shared terminator — one more published TLS port and one more cert lifecycle, and it still is not 443.

**Publish the DropVoice container directly on host :443.** It lost trivially: 443 is already owned by the shared Caddy fronting the other services; the container cannot have it.

## Acceptance criteria

- Three-host smoke green through the public URLs (health + version, landing, PWA + service worker, register round trip) after deploy.
- Access logs show true client IPs — the XFF chain Cloudflare → host Caddy → container Caddy → axum survives the extra hop; rate limiting keys per client, not per gateway.
- The container's published port is unreachable from off-box (loopback bind only).
- Staging and production render the identical Caddyfile form (HTTP); the certs mount and `tls_profile` no longer exist.

## Risks

- The real-IP chain gains a hop owned by operator-side config: if the host Caddy block drops `trusted_proxies` or the CF-Connecting-IP rewrite, every user shares one rate-limit bucket (all requests appear from the gateway IP) — the runbook marks those lines as non-optional.
- The container's `trusted_proxies_cidrs` trusts the whole docker-bridge range until narrowed to the actual gateway /32 after the first deploy (TODO in group_vars).
- The host Caddy config lives outside this repo's ansible; the runbook snippet can drift from box reality — the first-deploy verification (curl through the public URL) is mandatory.
- Loopback port 8089 is a chosen default; a collision on ttyo requires editing both group_vars and the host site block in one move.
