# DV-RFC: prek is the single source of format, lint, and test gating

Status: implemented

## Problem

DropVoice is developed by several AI coding agents (Claude Code, ZCode, Codex, OpenCode) alongside CI, across a pnpm workspace of four TypeScript apps, three TS packages, and two Rust crates. Every one of those surfaces has its own native way to declare hooks and checks. Without a single declared home for format/lint/test wiring, the same rules get defined per tool, per project, and in CI — and drift apart: one tool's hook formats differently than another's, a child project pins a diverging dependency version (an axum 0.7/0.8 split happened before the consolidation), and CI becomes the only place violations surface, long after the edit that caused them.

## Decision

All quality gating is declared exactly once, in **prek** (workspace mode): a root `prek.toml` plus one `prek.toml` per project (`apps/desktop`, `apps/mobile`, `apps/landing`, `apps/pairing-server`, `packages/{core,i18n,ui}`). prek discovers the child configs and runs each project's hooks with cwd at that directory, so project-relative filters stay honest.

Hook **groups** are orthogonal to stages:

- `format` — mutating fixers (prettier, oxlint --fix, cargo fmt, builtin whitespace/EOF fixers),
- `lint` — read-only gates (oxlint, tsc --noEmit, cargo clippy -D warnings),
- `check` — structural checks, generated-file drift, lockfile freshness; the pre-push stage runs the test suites.

Every entry point invokes the same definitions: AI post-edit hooks run `prek run --group format --files <edited>`, the AI Stop hooks run `prek run --group lint --all-files`, commit runs the pre-commit stage, push runs pre-push, CI (`.github/workflows/quality.yml`) runs both `prek run --all-files` and `prek run --stage pre-push --all-files`. `pnpm quality` mirrors CI.

The four tool directories (`.claude/`, `.zcode/`, `.codex/`, `.opencode/`) are disposable adapters carrying zero logic of their own; their PostToolUse/Stop hooks only translate payload formats and call prek. Generated and lock files (`theme.css`, `openapi.{json,yaml}`, `Cargo.lock`, `pnpm-lock.yaml`) are exempt from every fixer and guarded by drift gates instead (`design-sync`, `openapi-drift`, `lockfiles-fresh`, `agents-sync`).

## Alternatives considered

**Per-tool native configuration.** Each agent tool wires its own formatter/linter hooks, CI keeps its own job. It lost: one fact gains a home per tool and drifts between them — the exact failure the consolidation removed; adapters that do nothing but call prek are the fix.

**lefthook / husky+lint-stages.** Mature git-hook runners. They lost: both assume a single config file and a Node-centric repo; neither natively discovers per-project configs across a mixed cargo+pnpm workspace the way prek's multi-`prek.toml` workspace mode does. markpost had just recorded the same consolidation around prek (its 2026-08-12 record); DropVoice adopted the pattern two days later, adapted to a workspace with Rust crates.

**CI-only gating.** Define everything in GitHub Actions; no local hooks. It lost: violations surface an entire feedback loop late, and AI agents — the primary editors — edit and iterate locally where nothing would check them.

**No aggregator — each project's package.json scripts as the source.** It lost: `pnpm lint`/`format` scripts already exist but are convenience wrappers for humans; making them the definition re-creates per-project drift and leaves root-level files (workflows, scripts, docs) ungated.

## Consequences

- Changing any gate behavior means editing a `prek.toml` — never an AI adapter, never CI. The adapters are disposable by design; the repo documents this rule in the root `AGENTS.md`.
- Generated-file correctness is enforced by regeneration-and-compare drift gates rather than formatters, so generator output stays byte-stable.
- Commits are never bypassed (`--no-verify` is forbidden); conventional commits are enforced by commitlint at the commit-msg stage, also via prek.
- Any new check lands first in `prek.toml` (usually the root one); new tool integrations get a thin adapter only. The documentation gates added by the agent-harness record follow this exact path.
