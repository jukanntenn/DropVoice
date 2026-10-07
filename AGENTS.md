# AGENTS.md

Guidance for AI coding agents (Claude Code, Codex, ZCode, Cursor) working in this repository. `AGENTS.md` is the single source of truth at every level; each `CLAUDE.md` is a byte-for-byte copy.

DropVoice sends voice-to-text input from a mobile phone to a PC over a WebRTC DataChannel (P2P; phone = offerer, desktop = answerer); a public pairing-server relays signaling only — data never transits it. Monorepo, four applications plus shared packages:

- **`apps/desktop`** — Tauri 2 app: Rust backend (`src-tauri/`, crate `dropvoice-desktop`) hosting the WebRTC answerer + Enigo keyboard injection; React 19 frontend.
- **`apps/mobile`** — standalone Vite + React 19 PWA: the phone-side typing surface, DataChannel offerer.
- **`apps/pairing-server`** — Rust rendezvous (axum 0.8 + sqlx 0.8/SQLite) for LAN address lookup over public HTTPS.
- **`apps/landing`** — static marketing site built from the product's own design system (`@dropvoice/ui` + `@dropvoice/i18n`).
- **`packages/{core,i18n,ui}`** — pure-TS state machines/atoms/types, 4-locale i18n, design system.

Subtree-specific orders live in each subtree's `AGENTS.md`. Full structure tree, tech-stack versions, configuration single-source table, pairing flow, event bus, and ports: [docs/architecture.md](docs/architecture.md).

## Commands

pnpm (>=11) + Cargo workspaces, run from the repo root. The zero-config entry is the VS Code tasks in `.vscode/tasks.json` — `dev:full` (desktop + pairing-server + mobile, bound to `Alt+R`), `dev:desktop`, `dev:pairing-server`, `dev:mobile`, `accept:*`; all env is baked into the task config. CLI equivalents plus build/deploy/migration procedures: [docs/development.md](docs/development.md).

- `pnpm doctor` — toolchain + prek hooks + ports health check; run it after a fresh clone.
- `pnpm dev:tauri` / `pnpm dev:mobile` / `pnpm dev:landing` — dev servers (export `PAIRING_SERVER_URL` / `VITE_API_PROXY_TARGET` per docs/development.md).
- `pnpm build:tauri` — production desktop installer (MSI/NSIS/dmg/deb).

<a id="run-relevant-checks-locally"></a>

Select the narrowest checks that cover the changed surface — focused suites owning the behavior, `hdsh pairing verify` for documentation pairs, `hdsh rfc verify` for decision records — and leave exhaustive rehearsal to CI; [pushing](.agents/skills/pushing/SKILL.md) owns the selection procedure.

## Quality gate

**prek is the single source of format, lint, and test gating** — workspace mode, root `prek.toml` plus one per project. AI post-edit hooks run `prek run --group format --files <edited>`; AI Stop hooks run `prek run --group lint --all-files`; commit → pre-commit stage; push → pre-push stage; CI → both (`pnpm quality`). Groups: `format` (fixers), `lint` (read-only), `check` (structure, drift, tests), `hdsh` (adopted governance gates). Never `--no-verify`; never bypass a gate. Details: [docs/development.md](docs/development.md#quality-gate-prek--single-source-of-truth).

Generated/lock files are exempt from every fixer and guarded by drift gates instead: `design-sync`, `openapi-drift`, `lockfiles-fresh`, `agents-sync`, and the `hdsh` group (pairing records, RFC format and archive, docs wrap/links/budgets, adopt drift — installed by `hdsh adopt`, verified with `hdsh adopt verify`).

## Code style

- Rust: typed errors via thiserror — `AppError` enum + `error_code()`, `AppResult = Result<T, AppError>`; module docs with `//!`.
- TypeScript: pure reducers for state machines, no side effects; shared state in `@dropvoice/core` jotai atoms.
- Naming: Rust `snake_case`/`PascalCase`; TS `camelCase`/`PascalCase` with co-located `*.test.ts(x)`; Tauri commands `snake_case`, invoked as `invoke::<ReturnType>("snake_case_name")`.
- Worked examples: [docs/development.md](docs/development.md#code-style).

## Git workflow

- Conventional Commits, commitlint-enforced: `feat|fix|docs|style|refactor|perf|test|chore|revert`.
- Scopes: `(desktop)`, `(mobile)`, `(pairing-server)`, `(core)`, `(ui)`, `(i18n)`, `(ci)`, `(docs)`.
- Versioning: `pnpm version:patch|minor|major` bumps and syncs everything. Release: push a `v*` tag → `release.yml` builds all platforms; `docker-publish.yml` publishes the pairing-server image.

## Boundaries (do NOT directly edit)

1. `specs/` — approved design specs are read-only; propose changes via discussion, never in place.
2. Generated files — regenerate, never hand-edit: `packages/ui/src/tokens/theme.css` (`pnpm design:sync`), `apps/pairing-server/docs/openapi.{json,yaml}` (`cargo run -p dropvoice-pairing-server --bin gen-openapi`).
3. Lock files — `pnpm-lock.yaml` / `Cargo.lock` are written by package managers only.
4. Agent instruction files — `AGENTS.md` is the source at every level; each `CLAUDE.md` is a byte-identical copy; `.claude/skills/` mirrors `.agents/skills/`. Rebuild all mirrors with `python scripts/sync_agent_files.py`; the `agents-sync` gate rejects drift.
5. Secrets — never commit. App config goes in TOML files (local configs gitignored, `*.example.toml` templates committed); env vars are escape hatches. CI image publishing needs `DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` repo secrets.

## Conventions

- Workspace dependency inheritance: shared versions are declared at the workspace root; members inherit via `workspace = true` (Cargo) / `workspace:*` (pnpm). Never pin inline what the root declares.
- Configuration: every knob has exactly one source of truth — the [single-source table](docs/architecture.md#configuration-single-sources).
- The pairing flow (QR → offer via pairing-server → Rust signaling supervision → DataChannel → Enigo injection) and the [connection-state event bus](docs/architecture.md#desktop-connection-state-event-bus) are documented in docs/architecture.md. Data is P2P; no server port carries it.
- i18n: 4 locales (`en`, `zh`, `zh-TW`, `ja`) × 5 namespaces in `packages/i18n/locales/`; detection `?lang=` → localStorage `dropvoice-lang` → navigator.
- AI tool hooks: `.claude/`, `.zcode/`, `.codex/`, `.opencode/` are disposable adapters over prek — change gating in `prek.toml`, never the adapters.

## Documentation & decision records

Documentation follows [docs/AGENTS.md](docs/AGENTS.md): one fact, one home; current state, not change history; in-scope prose ships as bilingual pairs under the [pairing contract](docs/i18n/README.md) (`hdsh pairing verify`); English for agent instructions; word ceilings in `.hdsh/docs.manifest.json`; links must resolve.

Decision rationale lives in [RFCs](.agents/rfcs/README.md) (`.agents/rfcs/`): every non-trivial change adds or updates a record in the same change — the why, the alternatives that lost, the consequences. Only purely mechanical edits with no change to behavior, contracts, structure, process, or rationale are exempt.

## Editing these instructions

Root and subtree `AGENTS.md` files are standing orders; subtree files supplement this one and never repeat it. Keep each rule 1–3 lines, linking its home instead of restating it. Word ceilings live in `.hdsh/docs.manifest.json`; on red: relocate to the owning tier, condense, raise the ceiling last with a justified diff. After editing any `AGENTS.md`, `CLAUDE.md`, or skill, run `python scripts/sync_agent_files.py`.
