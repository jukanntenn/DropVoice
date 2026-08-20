# Coding principles

Behavioral constraints for the agent. Each is a rule the agent gets wrong without being told. Production safety and data integrity outrank every principle here — including the license to redesign from scratch; against a mere default or style rule, the principle wins.

## Ground every conclusion in fact

Library facts, APIs, and protocols must be read from source or docs before you act on them — training data is a blind spot, not a source. Verify every conclusion on the ground: `file:line` for logic, Playwright (`pnpm test:e2e`) for UI, read-only HTTP against dev for behavior — the pairing-server API on `:38424` (bare `cargo run`) or the acceptance container on `:8080` via `scripts/smoke.py` — and `cargo test` against the real SQLite database (bundled libsqlite3, migrations applied at startup via `sqlx::migrate!()`), means the model can actually perform. Pure algorithm or syntax knowledge may use training knowledge.

The `docker compose run` flag incident is the shape of getting this wrong: ansible's `interactive=false` would emit `--no-interactive`, a flag docker compose does not have — caught only because it was checked against the compose docs (v5.1.3) before relying on it.

## Defer to community convention

When a convention or best practice is uncertain, ask "what is the community/official convention?" and verify against authoritative open-source source, not training memory (e.g. whether `format`/`lint`/`check` are the prek group names here, or how oxlint 0.15 wants its config — both verifiable against the tool's own schema/docs).

Distinct from _Ground every conclusion in fact_: that one governs facts about a library you are integrating; this one governs convention and best-practice decisions.

## Converge before you implement

A spec or plan must be self-contained, complete, and unambiguous — an executor with no taste can land it mechanically, with no room to improvise. Resolve every open point before implementing; do not start on the strength of a half-settled plan.

## Fix the root cause, not the symptom

The solution you choose must be the most natural and optimal — not a patch over the symptom, and not one trapped by the existing implementation. You may shed all legacy and start from zero when the root fix requires it.

When formatter/lint coverage leaked across tools, the fix was not to configure each AI hook separately but to delegate to `prek.toml` as the single truth — no parallel formatter or lint definition exists outside prek.

## Design from first principles

Derive a design from the business essence; every premise is breakable; an elegant scheme beats an inherited one. Distinct from _Fix the root cause, not the symptom_: that one is how you _fix_ a problem (root, not patch); this one is how you _design_ a system (re-derive, question assumptions). Dropping the desktop's local axum HTTP/WS server and UDP discovery entirely, in favor of WebRTC DataChannel P2P with only a rendezvous pairing-server over public HTTPS, is this principle applied.

## Single source of truth

Each category of information has exactly one authoritative source: schema truth is the sqlx migrations in `apps/pairing-server/migrations/` (applied automatically at startup), API truth is the generated `apps/pairing-server/docs/openapi.{json,yaml}` (regenerated via `gen-openapi`, byte-compared by a drift gate, never hand-edited), design tokens are `packages/ui/src/tokens/theme.css` generated from `DESIGN.md` via `pnpm design:sync`, UI text lives in the locale files (`packages/i18n/locales/`), lint/format in `prek.toml`, deploy config in ansible group_vars + templates. The frontend renders; it does not decide.

## Naming is part of the API

A name is an API surface. If a name does not fit its business meaning, do not force it — brainstorm candidates and let the user choose, to prevent semantic drift.

## Degrade gracefully, never silently

A failure must be handled and observably recorded, and must not block downstream work — but a silent failure is always wrong. A failed heartbeat registration or relayed offer must not block the pairing flow, but it must surface in the UI connection state and server metrics, not vanish; config validation fails loudly at startup instead of falling back to defaults no one knows about.

## Minimal mock, maximal real

Mock only the request boundary, never the whole service. Keyboard injection is mocked at the OS boundary only (`MockInjector` vs the real `EnigoInjector`); backend tests run against a real SQLite database via sqlx — repository mocks in CI hide SQL drift; e2e drives the real container image through the public URL (`pnpm accept:test`: cargo e2e-caddy + `scripts/smoke.py` against `localhost:8080`, and smoke.py re-runs against the public URL after each staging deploy); local and CI run the same suite as fully as feasible.
