---
name: pushing
description: Use before pushing, force-pushing, marking ready for review, or claiming checks pass on a branch of this repository, and immediately after gh stack sync publishes rewritten branches, to select the smallest tests and checks that cover the outgoing or just-published diff without reflexively running the full repository suite.
---

# Pushing branches

Use this skill to run relevant local evidence once before a push. The sole ordering exception is `gh stack sync`, which may publish a cascading rebase before the rewritten layers can be validated; validate them immediately afterward and do not merge until the evidence passes. Hooks are worktree-local (install them with `hdsh worktree install`) and intentionally narrow:

<!-- hdsh:slot hooks -->

DropVoice's pre-commit runs the prek workspace (prettier/builtin fixers and oxlint on staged TS, cargo fmt/clippy on staged Rust, plus the check gates: agents-sync, design-sync, lockfiles-fresh, and the hdsh pairing/docs/RFC gates); commit-msg runs commitlint; the pre-push stage runs the per-app test suites (vitest for TS packages, cargo test for the desktop and pairing-server crates).
<!-- /hdsh:slot -->

CI owns exhaustive coverage and the platform matrix.

## Inspect the outgoing change

1. Confirm the checkout and branch.

```sh
git status --short --branch
git rev-parse --show-toplevel
```

2. Verify the live PR base or stack parent, fetch that ref, and inspect the complete scope against it.

```sh
hdsh scope --base <merge-base-ref>
```

The command never guesses or fetches a base. Supply the ref verified from current remote or stack state; use `--head <ref>` when inspecting a commit other than `HEAD`. Its versioned JSON records committed paths relative to the resolved merge base, while staged, unstaged, and untracked paths describe the current worktree. After merging a changed base, rerun the report, reassess which behavior the combined scope can affect, and rerun only checks invalidated by the merge.

## Select relevant evidence

There is no universal local baseline beyond the hooks. Every behavior change needs the narrowest available test or purpose-built check that would fail for its regression; add broader checks only for surfaces the diff actually reaches.

- **Behavior:** run the focused tests owning the changed behavior.

<!-- hdsh:slot focused-tests -->

Run the owning suite: desktop Rust `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml <name>`; pairing-server `cargo test -p dropvoice-pairing-server <name>`; one TS package `pnpm --filter <workspace-name> test` (vitest; narrow with `-- <file>`); browser flows `pnpm test:e2e`. Co-located `*.test.ts(x)` files own their behavior; leave the full matrix to CI unless the change is genuinely cross-cutting or the user requests it.
<!-- /hdsh:slot -->

- **Documentation:** run `hdsh pairing verify`; re-record a reviewed pair with `hdsh pairing record <pair>`. Decision records under `.agents/rfcs/` additionally run `hdsh rfc verify`.
- **Typed source:** run the repository's type check when the diff touches annotations it owns.

<!-- hdsh:slot type-check -->

`pnpm typecheck` runs `tsc --noEmit` per workspace member and owns annotations across `apps/*/src` and `packages/*/src`; Rust diagnostics come from `cargo clippy` in the lint group (desktop crate and pairing-server workspace).
<!-- /hdsh:slot -->

- **Packaging or entry points:**

<!-- hdsh:slot packaging-smoke -->

After changing `prek.toml`, an app/package `prek.toml`, `package.json`, or a `Cargo.toml`, run `uvx prek@0.4.12 validate-config prek.toml apps/*/prek.toml packages/*/prek.toml`, refresh the lock files (`pnpm install --lockfile-only`, then `python scripts/check_lockfiles.py`), and `pnpm doctor` for the toolchain/ports health check.
<!-- /hdsh:slot -->

- **Style beyond staged files:** the pre-commit fixers cover staged files only; run the repository's whole-tree style checks when relevant files are not yet staged.

<!-- hdsh:slot style-check -->

`prek run --group format --all-files` and `prek run --group lint --all-files` cover the whole workspace (prettier and oxlint per app config, `cargo fmt`/`cargo clippy` on both Rust workspaces); `pnpm format:check` and `pnpm lint` reach the same tools per package.
<!-- /hdsh:slot -->

Do not manually repeat a passing check merely because commit or push follows. In particular, do not duplicate the pre-push hook's check immediately before pushing.

### Focus unit coverage on the affected source

<!-- hdsh:slot coverage -->

Coverage is per-package vitest (`pnpm --filter <workspace-name> test:coverage`, config in each member's `vitest.config.ts`/`vite.config.ts`); there is no repository-wide 100% aggregate gate, so read the report for exactly the modules the diff touches. Rust coverage is not measured locally — the owning `cargo test` suites are the gate. The unmodified pre-push test stage remains what must pass before the change lands:

```sh
pnpm --filter @dropvoice/core test:coverage
```
<!-- /hdsh:slot -->

Do not delete branches, exclude modules from measurement, or weaken assertions merely to hide an uncovered affected file; add the missing owning tests instead.

## Full local rehearsal

Run the complete local approximation only when the user explicitly requests it, while diagnosing a CI failure, or when the change spans the repository so broadly that no narrower set is credible. The full local set is:

<!-- hdsh:slot full-suite -->

`pnpm quality` (prek `--all-files` plus the pre-push test stage over all files), `pnpm test:all` (all vitest suites plus the desktop Rust suite), and `pnpm test:e2e`; the CI OS matrix (windows/macos Rust, ubuntu e2e) has no full local equivalent.
<!-- /hdsh:slot -->

## Protect history-rewriting pushes

Rebase is allowed for standalone and stacked PR branches, including after review. Before a standalone history rewrite, fetch the current remote branch and record its exact OID; publish with `--force-with-lease=<branch>:<observed-oid>` so a concurrent update aborts the push. `gh stack push` and `gh stack sync` supply lease protection for their managed branches. Raw `--force` is never allowed.

After any rewritten push, fetch the live heads again and re-audit unresolved review threads, approvals, mergeability, and checks. Commit hashes and inline-comment anchors from before the rewrite are not current evidence.

### Post-sync validation

`gh stack sync` fetches, cascade-rebases, and pushes as one operation, so it cannot place local validation between rewrite and publication. Before running it, require a clean worktree and record the official stack order and exact remote heads. After it returns:

1. Re-query every branch head and the official GitHub stack order.
2. Inspect the changed scope of every rewritten layer against its live PR base.
3. Run the relevant evidence selected by this skill for each affected layer.
4. Keep every PR unmerged and report validation as pending until all selected checks pass.

If post-sync evidence fails, leave the lease-protected published heads in place, repair the failure, validate the repair, and publish the correction. Do not claim the sync made the stack ready merely because the command succeeded.

## Handle failures

If a relevant check fails before an ordinary push, stop and fix or explain the blocker. Do not push and hope CI differs. For the post-sync exception, block the merge and follow the repair procedure above.

If a failure looks environment-specific, prove it:

- Record the exact command, failing test, and platform-specific mismatch.
- Confirm the relevant non-platform evidence.
- Prefer fixing cross-platform nondeterminism when the check is required.
- Bypass a local hook only when the user explicitly asks or agrees, and report exactly what failed and why CI is expected to differ.

## Push procedure

For ordinary and standalone rebase pushes:

1. Run the selected relevant checks once.
2. Commit normally and inspect any files changed by fixer hooks before continuing.
3. Push normally, or use the exact lease for an authorized rewritten branch, so the pre-push hook runs.
4. Verify the remote ref matches local `HEAD`.

```sh
git rev-parse HEAD origin/$(git branch --show-current)
```

For GitHub PRs, inspect remote CI after the push:

```sh
gh pr checks
```

Report pending checks as pending. Inspect failures before attributing them to the branch or the environment.

When `gh pr checks` reports "no checks reported" and `/actions/runs?head_sha=<sha>` returns `total_count: 0`, read mergeability before suspecting the push or a dropped GitHub event:

```sh
gh pr view <number> --json mergeable,mergeStateStatus
```

GitHub creates no `pull_request` workflow runs while a PR is `CONFLICTING`/`DIRTY`, so the absent signal is the conflict, not infrastructure. Resolving the conflict is the only fix; empty commits, `--allow-empty` pushes, draft/ready toggles, and revert-and-restore bounces all leave `total_count` at zero and add junk history. Confirm the conflicting paths with `git merge-tree --write-tree HEAD origin/<base>` when the branch cannot be merged locally yet.

For `gh stack sync`, use the post-sync validation sequence instead of pretending the ordinary order was possible.
