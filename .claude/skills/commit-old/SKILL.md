---
name: commit
description: "Split and organize AI code changes into well-structured commits following this project's conventions. Use this skill whenever committing changes — whether one file or many. Trigger when the user asks to commit, save, submit, or stage changes, or when dirty files need committing after a task. Also use when multiple files were edited and need logical grouping into separate commits, or when the user asks about commit conventions for this project."
---

One big commit is a black box — you can't bisect, can't cherry-pick, and can't tell "which change broke what." Split commits give you:

- **`git bisect`** works — each commit is either good or bad, no mixed states
- **`git revert`** is safe — revert one logical change without pulling out others
- **Reviewable history** — humans and AI can audit what happened, in order
- **Rollback confidence** — if a refactor broke something, revert just the refactor

## Design principles

- **Split granularity — group by logical change unit**
  A logical change unit is a set of file edits that together accomplish one coherent purpose. We do this instead of per-file splitting because this project's files are tightly coupled (e.g., handler + service + repository in backend, or page component + API fetcher + types in frontend). Per-file would create meaningless fragments.

- **Who decides grouping — AI drafts plan, human confirms**
  The AI inspects dirty state and proposes a commit plan, but the human has final say. This balances automation speed with human oversight over their own git history.

- **Commit message format — Conventional Commits, match existing history**
  Our history uses `feat(auth):`, `fix(delivery):`, `refactor:`, `chore:`, `style:`, `build:`, `test:`. Staying consistent makes `git log` readable and predictable.

- **Verification gate — lint + test must pass before each commit**
  Backend has `go test ./...` + `golangci-lint run`; frontend has `pnpm lint` + `pnpm test:run`. Running them before commit catches issues early and saves CI round-trips.

- **Push policy — never auto-push**
  Pushing is an irreversible outward-facing action. The user always decides when to push.

## Rules

### 1. Group by logical change, not by file

A "logical change unit" is a set of file edits that together accomplish one coherent purpose. Examples:

| Logical change                           | Files                                              | Rationale                                               |
| ---------------------------------------- | -------------------------------------------------- | ------------------------------------------------------- |
| Add a new REST endpoint                  | handler + service + repository (backend)            | Full vertical slice — one commit                        |
| Implement a frontend page with API call  | page component + API fetcher + type definition      | Feature spans component, data, and types — one commit   |
| Fix a CSS padding bug                    | frontend component only                             | Isolated fix — one commit                               |
| Update Docker build and compose config   | Dockerfile + docker-compose.yml                    | DevOps change — one commit                              |
| Bump Go dependency and update code       | go.mod + go.sum + affected backend files            | Dependency update — always together                     |

### 2. Ordering: infrastructure → feature → fix → docs → chore

When multiple commits are needed, follow this order:

```
1. build / chore     — build system, dependencies, tooling, devops
2. feat              — new features
3. fix               — bug fixes
4. refactor          — code reorganization
5. style             — UI styling, theme changes
6. docs              — README, AGENTS.md, specs
7. test              — test additions/changes
8. chore(release)    — version bump (always last)
```

Rationale: infrastructure changes first (they're prerequisites), features and fixes in the middle (the actual work), docs and release housekeeping last (they describe what happened).

### 3. Commit message format

```
<type>(<scope>): <description>
```

**Types:** `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`, `build`, `style`, `perf`

**Scopes:** `auth`, `delivery`, `admin`, `i18n`, `frontend`, `backend`, `devops`, or omit scope for cross-cutting changes.

**Rules:**

- Lowercase description, no trailing period
- Match the language of the change (Chinese files → Chinese message, English → English)
- Imperative mood: "add" not "added", "fix" not "fixed"

**Examples from our history:**

```
feat(auth): simplify github oauth flow and state handling
fix(delivery): restore delivery pipeline removed during codebase refactoring
refactor: unify frontend env vars to single BACKEND_URL
chore: clean up dev env docs and configs
style: update ui styles and theme, fix feishu notification logic
test: add comprehensive unit and integration tests across backend stack
build: add pnpm workspace config and update Dockerfile
```

### 4. Verification gate — lint & test before commit

Before every commit, run the relevant checks based on what changed:

**Backend changes:**

```bash
cd backend && go test ./... && golangci-lint run
```

**Frontend changes:**

```bash
cd frontend && pnpm lint && pnpm test:run
```

**Both changed:** run both. If only one side changed, only run that side's checks.

If anything fails → fix first, then commit. Never commit failing code.

### 5. The commit plan protocol

When multiple files are dirty, the AI **must**:

1. **Inspect dirty state:** `git status --porcelain`
2. **Learn existing style:** `git log --oneline -5`
3. **Classify files:**
   - **AI-edited this session** — files the AI wrote/edited
   - **Unrecognized** — files the AI didn't touch (user edits, other tools)
4. **Draft a commit plan** grouping AI-edited files into logical commits
5. **Present the plan once** for human confirmation:

```
Proposed commits (in order):
  1. feat(auth): add password reset endpoint
     - backend/internal/api/rest/v1/auth.go
     - backend/internal/service/auth/service.go
     - backend/internal/domain/user/repository.go
  2. refactor: simplify env var handling across frontend
     - frontend/src/lib/api/client.ts
     - frontend/src/proxy.ts
     - frontend/.env.local

Unrecognized dirty files (NOT in any commit):
  - .gitignore

Reply 'ok' / '行' to execute. Reply with edits, or '我自己来' / 'manual' to abort.
```

1. **On confirmation:** execute `git add` + `git commit` for each batch in order. No `--amend`. No push.
2. **On rejection:** stop. Do not propose a second plan. Let the user commit manually.

### 6. Special cases

| Case                                                      | Rule                                                                                                 |
| --------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| **Only one file changed**                                 | Skip the plan protocol — commit directly with a descriptive message                                  |
| **Generated files** (swagger.json, next-env.d.ts)         | Bundle with the commit that caused them, or a single `chore` commit if standalone                    |
| **Mixed language edits** (e.g., README.md + README_zh.md) | Keep in one commit if they describe the same change                                                  |
| **Cross-stack feature** (backend + frontend)               | If tightly coupled (API + its consumer), one commit; if independent parts, split by stack            |
| **Swagger/docs regeneration**                             | Regenerate via `swag init` command, bundle with the API change that required it                       |

### 7. What NOT to do

- ❌ **One giant commit** for everything — defeats the purpose
- ❌ **Per-file commits** when files are logically coupled (handler + service = one feature)
- ❌ **Commit with failing tests** — always verify first
- ❌ **`git commit --amend`** — never rewrite history
- ❌ **`git push`** without explicit user request
- ❌ **Include unrecognized dirty files** silently — always list them separately
- ❌ **Placeholder commit messages** like "wip" or "update files"
- ❌ **Edit generated files** (swagger docs, lock files) directly — regenerate them

## Quick reference

```
1. git status --porcelain          → what changed?
2. git log --oneline -5            → what style?
3. Group by logical change          → plan commits
4. Present plan → human confirms   → one shot
5. go test + golangci-lint / pnpm lint + test:run → verify BEFORE each commit
6. git add + git commit            → execute in order
7. Never push, never amend
```
