# apps/pairing-server — standing orders

Rust rendezvous server (axum 0.8 + sqlx 0.8/SQLite) for LAN address lookup over public HTTPS. Signaling and device registry only — text data never transits this server. System facts live in [docs/architecture.md](../../docs/architecture.md); deploy procedures in [docs/development.md](../../docs/development.md).

## Commands

- Dev: VS Code task `dev:pairing-server` (env-injected: `DATABASE_URL`=gitignored `dev.db`, `RATE_LIMIT_PER_SEC=10`, `ENABLE_DOCS=true`), or `cargo run --manifest-path apps/pairing-server/Cargo.toml` with the same env.
- Regenerate API docs: `cargo run -p dropvoice-pairing-server --bin gen-openapi` (utoipa 5, compile-time annotations) — regenerate and commit `docs/openapi.{json,yaml}` on any API-surface change; the `openapi-drift` gate byte-compares.
- Migrations: `sqlx migrate add [-r] <description>` (timestamp-prefixed); they apply automatically at startup via `sqlx::migrate!()`.
- Tests: `cargo test` (includes `tests/e2e/api.rs` and the openapi drift test); `pnpm accept:test` runs the container-level e2e + smoke. Clippy via prek (`apps/pairing-server/prek.toml`).

## Layout

`src/`: `api/` (router, auth, devices, signaling + SSE tickets, signaling_handlers, error, rate_limit, state), `store/` (open_pool, device_repo), `domain/` (pure device structs), `batch/` (BatchWriter coalesced writes), `cache/` (in-memory token cache), `config.rs`, `observability.rs`. `migrations/`, `docker/` (multi-stage image + Caddy), `devops/` (ansible + runner).

## Orders

- Tokens are issued to holders, never echoed back, and stored as SHA-256 hashes only. Long-lived tokens never appear in URLs or access logs — SSE subscriptions exchange them for one-time 60s tickets.
- Registration is a credential-controlled idempotent upsert: an existing device must carry the current Bearer token; a bare re-register gets 401.
- Runtime config: TOML probed at `/app/config.toml` then `config.local.toml`; a present-but-broken file fails startup — no silent fallback. Dev task env outranks TOML. `ENABLE_DOCS` is env-only.
- Real client IP comes from the trusted-proxy + XFF contract documented in docs/architecture.md — don't weaken `trusted_proxies_strict`.
- Caddy routing belongs to `docker/Caddyfile` + `devops/ansible/templates/Caddyfile.j2`; deployed values come from `group_vars/<env>.yml` only.
