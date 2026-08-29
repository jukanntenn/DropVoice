---
name: commit
description: Split and organize AI code changes into well-structured commits following this project's conventions. Use this skill whenever committing changes — whether one file or many. Trigger when the user asks to commit, save, submit, or stage changes, or when dirty files need committing after a task.
argument-hint: 'Optional: focus scope or hint about the logical changes'
---

Group edits into logical commits — by the change they make, never per-file. This is a pnpm + Cargo workspace gated by prek; verification is prek's job, not a hand-picked command list.

**Before committing**

1. Run `git status` and `git diff` (staged and unstaged). Identify the logical changes: a feature, its tests, its docs, and generated regenerations (openapi, theme.css) usually belong together; unrelated product changes in the worktree stay out of the commit. When the worktree carries someone else's in-flight changes, commit with an explicit pathspec (`git commit -- <paths>`) so only your files land.
2. Non-trivial changes need their decision recorded — a new or updated DV-RFC in the same commit (see the writing-dv-rfcs skill). Purely mechanical edits are exempt.
3. Regenerate what regenerates: API-surface changes → `cargo run -p dropvoice-pairing-server --bin gen-openapi`; DESIGN.md changes → `pnpm design:sync`; manifest changes → lockfiles refresh themselves on the next package/cargo command.

**Committing**

- Format: `<type>(<scope>): <description>` — types `feat|fix|docs|style|refactor|perf|test|chore|revert`; scopes `(desktop) (mobile) (pairing-server) (core) (ui) (i18n) (ci) (docs)`. commitlint enforces this at commit-msg.
- Order multiple commits: build/chore → feat → fix → refactor → style → docs → test → release.
- Present the commit plan (messages + files per commit) once, get confirmation, then execute. Never `--no-verify` — if a gate fails, fix the finding, not the gate.
- Never amend or rewrite pushed history. Never push unless asked.

**Verification**

The pre-commit stage runs the gates on staged files (prettier/oxlint/cargo fmt, clippy/tsc, drift gates, doc gates, commitlint). If prek modified files during a hook run, re-stage them and commit again. For outgoing commits, the pre-push stage runs the test suites — run the narrowest relevant tests yourself first (`pnpm test`, `pnpm test:rust`, `cargo test -p dropvoice-pairing-server`); CI owns the exhaustive matrix.
