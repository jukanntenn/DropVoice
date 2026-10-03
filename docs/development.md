# Development

Procedures: everyday commands, quality gates, migrations, deployment, and release. Current-state system facts live in [architecture.md](architecture.md); rules live in the [root `AGENTS.md`](../AGENTS.md) and [docs/AGENTS.md](AGENTS.md).

All commands run from the repo root unless noted. The project uses **pnpm** (>=11) and **Cargo workspaces**.

## Everyday development

One command per environment; required config is baked into `.vscode/tasks.json` task env (self-contained — no config files, no manual env). `Alt+R` (workspace keybinding) runs `dev:full`; nothing else to configure:

| Task                 | What it runs                                                                                                                 |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `dev:full`           | desktop + pairing-server + mobile in parallel (bound to `Alt+R`)                                                             |
| `dev:desktop`        | `pnpm dev:tauri` with `PAIRING_SERVER_URL=http://localhost:7380` built in                                                    |
| `dev:pairing-server` | `cargo run` → 127.0.0.1:7380, env-injected (`DATABASE_URL`=gitignored `dev.db`, `RATE_LIMIT_PER_SEC=10`, `ENABLE_DOCS=true`) |
| `dev:mobile`         | `pnpm dev:mobile` → :5174 (`/api` proxied to :7380 by task env)                                                              |
| `accept:desktop`     | desktop against the acceptance container (`PAIRING_SERVER_URL=:8080`)                                                        |
| `accept:mobile`      | mobile with `/api` proxied to the acceptance container (`:8080`)                                                             |

CLI equivalents (when not using VS Code): `pnpm dev:tauri`, `pnpm dev:mobile`, `pnpm dev:landing` (→ :5175), `cargo run --manifest-path apps/pairing-server/Cargo.toml` — export `PAIRING_SERVER_URL=http://localhost:7380` for the desktop process and `VITE_API_PROXY_TARGET=http://localhost:7380` for mobile (see root `.env.example`; the VS Code tasks are the sanctioned zero-config entry). After a fresh clone, run `pnpm doctor` to check toolchain, prek hooks and ports in one shot.

## Quality gate (prek — single source of truth)

All quality gating lives in **prek** (workspace mode): the root `prek.toml` plus one `prek.toml` per project (`apps/desktop`, `apps/landing`, `apps/mobile`, `apps/pairing-server`, `packages/{core,i18n,ui}`). CI, git hooks and the AI-tool hooks all invoke prek — no parallel lint/format definition exists anywhere else.

```bash
prek install            # once per clone: pre-commit + pre-push + commit-msg hooks
prek run --all-files    # format + lint + gates (pre-commit stage)
prek run --stage pre-push --all-files   # tests (pre-push stage)
pnpm quality            # alias for both of the above
pnpm docs:check         # documentation gates (dv-rfcs format, doc pairs, budgets, links)
pnpm test:e2e           # Playwright e2e (root e2e/ dir; also a CI job)
```

Hook **groups** (orthogonal to stages):

- `format` — mutating fixers with auto-fix on (prettier --write, oxlint --fix, cargo fmt, builtin whitespace/EOF fixers)
- `lint` — read-only gates (oxlint, tsc --noEmit, cargo clippy -D warnings)
- `check` — structural checks, generated-file drift, lockfile freshness (pre-commit stage); vitest + cargo test (pre-push stage)

Invocation map: AI post-edit hooks run `prek run --group format --files <edited>`; AI Stop hooks run `prek run --group lint --all-files`; commit → pre-commit stage; push → pre-push stage; CI → both `prek run --all-files` and `prek run --stage pre-push --all-files` (`.github/workflows/quality.yml`), plus a Windows/macOS desktop-Rust matrix for platform-specific coverage and a Playwright e2e job.

Generated/lock files are exempt from every fixer and guarded by drift gates instead: `packages/ui/src/tokens/theme.css` (`design-sync`, fix: `pnpm design:sync`), `apps/pairing-server/docs/openapi.{json,yaml}` (`openapi-drift`, fix: `gen-openapi`), `pnpm-lock.yaml`/`Cargo.lock` (`lockfiles-fresh`, fix: `pnpm install --lockfile-only` + any cargo command), `AGENTS.md`/`CLAUDE.md` pairs and the skills mirror (`agents-sync`, fix: `python scripts/sync_agent_files.py`), Markdown content (`doc-check` → `python scripts/doc_sync.py`).

## AI tool hooks (thin prek wrappers)

`.claude/`, `.zcode/`, `.codex/` and `.opencode/` carry no lint/format logic of their own — their hooks are disposable adapters over prek:

- PostToolUse (Edit|Write|ApplyPatch) → `prek run --group format --files <edited>` (never blocks; exit 0/1 both acceptable)
- Stop → `prek run --group lint --all-files`, blocking with diagnostics on failure (OpenCode has no blocking Stop channel — it falls back to the pre-commit gate + CI)

Per-tool differences are payload parsing only (snake_case vs camelCase keys). If a formatter/linter behavior needs changing, edit `prek.toml` — never the adapters.

## Code style

Rust — typed errors via thiserror (`AppError` enum with `error_code()`, `AppResult = Result<T, AppError>`); see `apps/desktop/src-tauri/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("server already running")]
    ServerAlreadyRunning,
    #[error("max devices reached")]
    MaxDevicesReached,
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ServerAlreadyRunning => "SERVER_ALREADY_RUNNING",
            Self::MaxDevicesReached => "MAX_DEVICES",
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

TypeScript — pure reducers for state machines, no side effects (spec 02 section 5.3); see `packages/core/src/machines/connection.ts`:

```typescript
export type ConnectionState = 'idle' | 'connecting' | 'connected' | 'error';
export type ConnectionEvent = { type: 'CONNECT' } | { type: 'OPEN' } | { type: 'CLOSE' };

export function connectionReducer(state: ConnectionState, event: ConnectionEvent): ConnectionState {
  switch (event.type) {
    case 'CONNECT':
      return state === 'idle' ? 'connecting' : state;
    case 'OPEN':
      return state === 'connecting' ? 'connected' : state;
    case 'CLOSE':
      return 'idle';
  }
}
```

Naming: Rust `snake_case`/`PascalCase` with module-level `//!` docs; TypeScript `camelCase`/`PascalCase` with co-located tests as `*.test.ts(x)`; Tauri commands `snake_case`, invoked as `invoke::<ReturnType>("snake_case_name")`.

## Individual lint / format commands (prek wraps these)

```bash
pnpm lint               # oxlint (all @dropvoice/* packages)
pnpm lint:fix           # oxlint --fix
pnpm format             # prettier --write
pnpm format:check       # prettier --check
cargo fmt               # Rust format (workspace root covers all crates)
cargo clippy --workspace -- -D warnings   # Rust lint, all crates
```

## OpenAPI docs (pairing-server)

```bash
cargo run -p dropvoice-pairing-server --bin gen-openapi   # Regenerate docs/openapi.{json,yaml}
```

- Docs are generated by utoipa 5 (compile-time) from `#[utoipa::path]` / `#[derive(ToSchema)]` annotations.
- Regenerate + commit `docs/openapi.{json,yaml}` whenever the API surface changes.
- Drift detection: `tests/openapi.rs` compares regenerated specs byte-for-byte against committed files; runs in `cargo test` and via the `openapi-drift` prek hook on any `apps/pairing-server/` change.
- Interactive Swagger UI mounts at `/docs/swagger-ui` only when `ENABLE_DOCS=true` (env-only debug toggle, not a TOML key; default off, set by the dev task and the acceptance compose).

## Database migrations (pairing-server)

```bash
# Install sqlx-cli once:
cargo install sqlx-cli --no-default-features --features sqlite,rustls

sqlx migrate add -r <description>    # Create reversible migration (.up.sql + .down.sql)
sqlx migrate add <description>       # Create simple migration (.sql, default)
sqlx migrate info                    # Show migration status
sqlx database create                 # Create dev database
# Migrations apply automatically at server startup via sqlx::migrate!() - no manual run needed.
```

## Build

```bash
pnpm build:tauri        # Production desktop installer (MSI/NSIS/dmg/deb)
pnpm ci:build           # Same as build:tauri (CI alias)
```

## Deploy pairing-server (Windows / containerized ansible)

Windows cannot run ansible natively (POSIX fork/sh dependency), so deployment runs ansible inside a throwaway Linux container (`docker run --rm`). The runner image is a local-only tool — not pushed — built on first run and cached after. `devops/deploy.py` (Python, cross-platform) drives this. `ansible.cfg` lives at the repo root (inventory + ssh tuning), so the playbook command needs no `-i` flags: `ansible-playbook apps/pairing-server/devops/ansible/deploy.yml -l <env>`.

Environments (single playbook, differences confined to `group_vars/<env>.yml`):

- **local dev acceptance** — not deployed. Two forms:
  - dev loop: VS Code task `dev:full` / `Alt+R` (= pairing-server `cargo run` → :7380 + desktop + mobile); phone opens `http://<dev-ip>:5174` (vite proxies `/api` to :7380). All config injected by the task env, not by hand.
  - container acceptance (artifact): `pnpm accept:up` (HTTP :8080, config from the committed `docker/config.acceptance.toml`); desktop/mobile via the `accept:desktop` / `accept:mobile` tasks.
- **staging** = fn @ 192.168.5.200 — dogfooding; production-isomorphic three-host layout on `*.dropvoice.bytehome.fun` (`dropvoice.`=landing, `app.`=PWA+/api, `ps.`=API; wildcard DNS + tunnel cert ops-managed, TLS terminated by the intranet tunnel). Caddyfile is the HTTP rendering of `templates/Caddyfile.j2`. Deploys the rolling `main` tag from the LAN registry. Flip domain/TLS/trusted-proxy values and it fits behind Cloudflare unchanged.
- **production** — public VPS behind Cloudflare (dropvoice.online, ttyo): `dropvoice.online`=landing, `app.`=PWA+/api, `ps.`=pure API. Traffic chain (markpost mode, same as the box's other services): Cloudflare reaches the origin on `:443` → the shared host Caddy terminates TLS (Origin CA cert) and reverse-proxies by hostname to the container's loopback-only `127.0.0.1:8089` → in-container Caddy (HTTP `:8080`) → axum. Deploys pinned version tags from ghcr.io (CI publishes releases there; prod pulls the public ghcr image anonymously — flip the package to public once in GitHub package settings; no registry login on the VPS).

Image tag semantics:

| Tag                    | Registry                     | Points at                                           | Moved by                             |
| ---------------------- | ---------------------------- | --------------------------------------------------- | ------------------------------------ |
| `main`                 | 192.168.5.50:5000 (LAN only) | whatever the local workspace last built             | `docker/build.py --push`             |
| `0.1.0` / `0.1.0-rc.1` | ghcr.io                      | git tag `v0.1.0` / `v0.1.0-rc.1`                    | CI `docker-publish.yml` on `v*` push |
| `latest`               | ghcr.io                      | newest **stable** release only (never a prerelease) | CI `docker-publish.yml`              |

Stability gate: exact `^v\d+\.\d+\.\d+$` moves `latest`. Prereleases use the hyphenated semver form (`v0.1.0-rc.1`, never `v0.1.0rc1`). CI multi-platform builds use native runners (amd64 + arm64, no QEMU); local cross-platform builds go through buildx + QEMU. `main` is never built by CI.

Build → deploy → accept flow (all from the dev machine):

```bash
# 1. Build image (single self-contained artifact: backend + Caddy + baked-in
#    mobile PWA + landing).
#    Defaults: LAN registry, host-machine platform, tags = main (+ --tags extras,
#    deduplicated). No registry cache by design (zero local gain, registry bloat).
pnpm image:push                                                       # -> LAN registry, tag: main
python apps/pairing-server/docker/build.py --push --all-platforms --tags v0.3.0  # cross-arch + immutable tag
pnpm build:tauri                                                      # desktop installer -> local install

# 2. Local container acceptance (against the artifact, before any deploy).
#    Plain HTTP :8080 — no TLS on the local box (this machine's WSL2 breaks
#    Go TLS servers; deployed forms terminate TLS externally anyway).
pnpm accept:up        # compose up (config: committed docker/config.acceptance.toml)
pnpm accept:test      # e2e-caddy (--ignored) + scripts/smoke.py http://localhost:8080
pnpm accept:down      # compose down -v (clean slate)

# 3. Deploy staging. The playbook renders config.toml + compose + Caddyfile from
#    group_vars, then runs scripts/smoke.py against public_url (health + PWA +
#    round trip + version match via /health APP_VERSION), so a green deploy
#    already passed the smoke gate.
pnpm deploy:staging                    # deploys `main` tag
pnpm deploy:staging -- --tag v0.3.0    # pin / rollback to immutable tag
python apps/pairing-server/devops/deploy.py staging --check --diff   # dry-run preview
```

Human acceptance gate before production promotion: `apps/pairing-server/devops/runbooks/accept-staging.md`.

Mounts: repo root → `/workspace` (playbook edits live, no image rebuild; the repo-root ansible.cfg applies from this workdir), `~/.ssh` → `/ssh` (entrypoint copies keys + chmod 600 for Windows perms), `~/.ansible-vault` → `/vault`. See `apps/pairing-server/devops/runner/` and `devops/deploy.py`.

Prerequisites (once): vault password file at `~/.ansible-vault/dropvoice-<env>.pwd` (e.g. `dropvoice-staging.pwd`); image publishing needs no registry secrets (ghcr.io uses the built-in GITHUB_TOKEN).

## Versioning & release

```bash
pnpm version:patch      # Bump patch + sync versions across packages + changelog
pnpm version:minor
pnpm version:major
# Release: push tag `v*` -> release.yml builds all platforms automatically,
# docker-publish.yml publishes the pairing-server image (ghcr.io).
```

An exact prerelease (e.g. `v0.1.0-rc.1`) is cut by hand: set the root version,
run `pnpm version:sync` (syncs every version field, including all package.json
files), commit, then `git tag v0.1.0-rc.1` — `pnpm version:pre:*` only
increments from the current version.

Desktop updater distribution rides the same stable tag: the `publish-r2` job in `release.yml` uploads the signed updater artifacts to Cloudflare R2 (`releases.dropvoice.online`) and moves `update/manifest.json`; prerelease tags never do. One-time prerequisites (R2 bucket + custom domain, `CLOUDFLARE_API_TOKEN` and `TAURI_SIGNING_PRIVATE_KEY` secrets): [production Cloudflare runbook](../apps/pairing-server/devops/runbooks/production-cloudflare.md). Rationale: [DV-RFC](../.agents/dv-rfcs/proposed/2026-10-03-desktop-updater-r2-edge-distribution.md).
Before the first stable, the update chain is rehearsed by hand: install `rc.1`, cut `rc.2`,
download its release assets and run `scripts/publish_updater_manifest.py --tag v0.1.0-rc.2
--allow-prerelease` (with `CLOUDFLARE_API_TOKEN` exported), then watch the rc.1 client update;
once a stable install base exists, the manifest moves only via CI's stable-tag gate.

Commits follow Conventional Commits (commitlint-enforced); scopes: `(desktop)`, `(mobile)`, `(pairing-server)`, `(core)`, `(ui)`, `(i18n)`, `(ci)`, `(docs)`.
