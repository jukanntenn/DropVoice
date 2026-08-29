# DV-RFC: Agent harness — dv-rfcs, layered agent instructions, docs tiers, local gates

Status: proposed

## Problem

DropVoice is developed primarily by AI coding agents. The plumbing already exists and stays as-is: prek is the single source of format and lint, and the four tool directories (`.claude/`, `.zcode/`, `.codex/`, `.opencode/`) are thin adapters over it. What is missing is everything above the plumbing, and the gaps are observable today:

1. **Decision rationale has no durable home.** Why signaling relays through a public pairing-server instead of LAN discovery, why device tokens are stored as hashes only, why prek won over per-tool hook logic — these live in commit messages, chat transcripts, and prose buried inside the root `AGENTS.md`. A future change is never forced to confront what a decision beat, so every re-litigation starts from zero. Both reference projects name the failure this invites: a decision recorded without what it beat invites re-litigation.
2. **The root `AGENTS.md` mixes standing orders with reference manuals.** At 3,438 words it loads dev-loop tasks, deploy runbooks, configuration tables, the pairing-flow walkthrough, and conventions into every session — 3–4× the ceiling either reference project found sustainable. Subtree-specific orders (the desktop WebRTC answerer, pairing-server migration rules) have no home closer than the root.
3. **Markdown carries no mechanical contract.** Relative links rot invisibly, nothing bounds document growth, and skills drift silently: `.agents/skills/commit-old` still tells agents to verify with Go toolchains (`go test`, `golangci-lint`) that do not exist in this repository — a stale copy from markpost that survived because nothing checks it.

Two reference repositories inform this proposal. deepseek-harness is the mechanism's source (agent notes, skills, layered agent instructions, doc tiers, a TypeScript gate suite). markpost adapted it successfully, and its decision history records the governing lesson: **port pieces only when a trigger signal appears, never wholesale** — the reference project's scale is the ceiling of where this road leads, not the starting point.

## Proposal

Introduce the harness in four components. All new gates are stdlib-Python scripts under `scripts/`, wired through prek (the existing single gate source); CI inherits them with zero workflow changes. The authoritative doc-tier table and slop checklist will live in `docs/AGENTS.md`; this record fixes the decisions behind them.

### 1. dv-rfcs — decision records under `.agents/dv-rfcs/`

A DV-RFC records a decision or proposal that affects this codebase: the why, the what-we-gave-up, and the consequences. Placement follows the reference projects: `.agents/` is the directory agent tools treat as their load path.

- **Tree**: `implemented/`, `proposed/`, `rejected/` — flat lifecycles, no class subfolders, no archive. The lifecycle tree is the inventory; there is deliberately no index file to maintain.
- **Naming**: `{lifecycle}/yyyy-mm-dd-topic-title.md`. The date is when the topic was first proposed, per git history; the filename regex `^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*(\.zh)?\.md$` is gate-enforced.
- **Format** (gate-enforced): first line `# DV-RFC: <title>`, third line `Status: proposed | implemented | rejected — <one-line reason>` agreeing with the folder. Body opens with `## Problem` stated to stand without the solution. `## Alternatives considered` is mandatory in every record — one bold-led paragraph per genuine alternative, recorded as argued, never invented after the fact.
- **Skeletons**: `proposed/` carries `## Proposal`, `## Alternatives considered`, `## Acceptance criteria`, `## Risks`. `implemented/` carries `## Decision` (present tense, what shipped), `## Alternatives considered`, `## Consequences`; proposal-era headings are banned there. `rejected/` keeps its proposal-time sections frozen; the verdict lives on the `Status:` line.
- **Lifecycle moves are mechanical**: `proposed → implemented` rewrites Proposal into present-tense Decision and folds Acceptance criteria and Risks into Consequences; `proposed → rejected` only adds the reason to the `Status:` line. Never edit a record into a different decision — supersede it and cross-link both.
- **Bilingual pairs**: every record is `foo.md` + `foo.zh.md`, same skeleton, English headings and machine tokens, updated in the same change. Structural parity (heading sequence, byte-identical fenced blocks) is gate-enforced.
- **Backfills** (2, `implemented/`, dates = first git appearance): `2026-08-14-prek-single-source-of-format-and-lint` and `2026-08-14-webrtc-datachannel-public-pairing-server`. The in-flight desktop Rust-signaling migration is not backfilled — it writes its own record when it lands.
- **Contract files**: `.agents/dv-rfcs/AGENTS.md` (standing orders, ~100 words) + `README.md`/`README.zh.md` (normative contract: naming, lifecycle, skeletons, pairing).
- **Deferred with revival triggers**: class layer (records > 50), frozen archive + hash manifest (superseded backlog), i18n.yaml hash sidecars, postmortem tier (first costly incident).

This record is itself the first DV-RFC. It moves to `implemented/` at the end of rollout, rewritten into Decision + Consequences — the rules bind their own introduction.

### 2. Skills — `.agents/skills/` is the source, `.claude/skills/` the mirror

- Source of truth is the tool-neutral path `.agents/skills/` (this repository's primary tooling is not Claude Code); `.claude/skills/` is a byte-identical mirror, gate-enforced like the AGENTS/CLAUDE pairs.
- Delete `commit-old` and write a fresh `commit/` skill aligned with this repo's stack (pnpm + cargo + prek verification, dropvoice scopes, never `--no-verify`); the current one is a stale markpost copy referencing Go toolchains.
- Add `writing-dv-rfcs/` (the RFC workflow: duplicate check, lifecycle choice, skeleton, bilingual pairing, `python scripts/doc_sync.py <files>` validation) and a lean `doc-standards/` (doc placement tiers + how to respond to gate failures).
- Keep `grill-me`, `iterating`, `tauri-icon-generation` unchanged.

### 3. Layered AGENTS.md with word budgets

The root file condenses from 3,438 words to ~1,000 words of standing orders — each rule 1–3 lines linking its home. Subtree files carry subtree-specific orders and never repeat the root. Every `AGENTS.md` gets a byte-identical `CLAUDE.md` twin, fixed direction (AGENTS.md is the source, `cp` is the fix), extending the existing `agents-sync` gate to all pairs. Budgets are gate-enforced (`verify_doc_budgets.py` + `doc_budgets.manifest.json`; a manifest entry whose file disappears fails, so renames cannot orphan budgets):

| File                            | Ceiling (words) |
| ------------------------------- | --------------- |
| Root `AGENTS.md`                | 1000            |
| `apps/desktop/AGENTS.md`        | 500             |
| `apps/mobile/AGENTS.md`         | 300             |
| `apps/pairing-server/AGENTS.md` | 450             |
| `apps/landing/AGENTS.md`        | 150             |
| `packages/AGENTS.md`            | 250             |
| `docs/AGENTS.md`                | 600             |
| `.agents/dv-rfcs/AGENTS.md`     | 150             |

On a red budget: relocate to the owning tier, condense, raise the ceiling last with a justified manifest diff in the same change.

### 4. docs/ — tiered prose, bilingual pairs

New root `docs/` absorbs the reference material leaving the root `AGENTS.md`, so no fact is lost — every relocated fact keeps a link from its old home:

- `docs/development.md` + `.zh.md` — full command/environment detail (dev tasks, doctor, quality gates, migrations).
- `docs/architecture.md` + `.zh.md` — system composition: pairing flow, signaling supervision, connection-state event bus, ports, configuration single-source table.
- `docs/AGENTS.md` — the documentation standard itself: tier table (one fact, one home), rules each naming its gate, slop checklist.
- Deployment content stays in its existing home (`apps/pairing-server/devops/` + runbooks) — classified in the tier table, not moved. `specs/` (read-only) and `PRINCIPLES.md` are classified only, untouched.

Bilingual scope (decided): DV-RFCs and `docs/` prose pages are pairs. Exempt: all `AGENTS.md`/`CLAUDE.md`, skills, generated files (openapi), `specs/`, `README.md`.

### 5. Gates — stdlib Python behind prek

`scripts/doc_sync.py` aggregates, accepting file arguments (staged locally via prek) or running full-corpus (CI via `prek run --all-files`):

1. `verify_dv_rfc_format.py` — filename regex, title, Status-folder agreement, `## Problem` opener, mandatory Alternatives, per-lifecycle skeletons, proposal-speak ban in `implemented/`, `.zh.md` requires its original plus structural parity, stray-file detection.
2. `verify_doc_pairs.py` — pairing completeness both directions, heading-sequence parity, byte-identical fenced blocks, over the pairing scope with an exclusions manifest.
3. `verify_doc_budgets.py` — the table above.
4. `verify_md_links.py` — relative markdown links resolve.

prek wiring (root `prek.toml`): new `doc-check` hook (check group, `\.md$`, excluding generated files and the four tool-adapter dirs — never `.agents/dv-rfcs/`); `agents-sync` extended to all AGENTS↔CLAUDE pairs plus the skills mirror; a fixer script rebuilds twins and the mirror. The prettier/whitespace exclusion narrows from `^(\.agents|\.claude|\.codex|\.opencode|\.zcode)/` to the four tool dirs only — wholesale-excluding `.agents/` lets gates pass green over real content (markpost's recorded enforcement-parity lesson). `pnpm docs:check` / `docs:fix` aliases.

### 6. Rollout — three batches, harness-only staging

1. **dv-rfcs mechanism**: tree + contract files + format gate + `doc_sync` + prek wiring + exclusion narrowing + `writing-dv-rfcs` skill + both backfills.
2. **Layering and docs**: subtree AGENTS.md files + docs pairs + root condense + `sync-agents` extension + budget/link/pair gates.
3. **Finish**: `commit` skill rewrite, `doc-standards` skill, delete `commit-old`, rebuild mirror, `prek run --all-files` + `doc_sync` green, move this record to `implemented/`.

Each batch is its own conventional commit (`docs`/`process` scope) staging only harness files; the ~50 in-flight product files in the worktree are never staged into harness commits.

## Alternatives considered

**Port the deepseek-harness wholesale.** It is the proven source, but its corpus (600+ notes, 40+ subsystem pages, ~5,300 lines of gate TypeScript) is the ceiling of the road, not the starting point. It lost: DropVoice starts with an empty record tree and a handful of docs; the class layer, frozen archive, i18n triplets, and doc-site projection would tax every edit while buying nothing at this size. Pieces are ported on trigger signals, each recorded with its revival condition.

**The reference repo's TypeScript gate suite and scheduler.** A dependency-graph runner over ~25 gates serves a scale DropVoice does not have, and the repo convention is cross-platform stdlib Python (`scripts/doctor.py`, `smoke.py`, the existing drift gates). It lost: four-plus sequential gates over a small Markdown corpus finish in seconds with zero new toolchain dependencies.

**Symlink mirrors for CLAUDE.md.** Zero drift by construction. It lost: a Windows checkout without `core.symlinks=true` materializes `CLAUDE.md` as a regular file containing nine bytes of target path — the first Windows clone trades drift risk for a broken mirror. Byte-identical copies with a gate are the repo's existing, working mechanism.

**Direction-free mirrors (markpost's git-HEAD mechanism).** Resolving mirror direction against git HEAD handles the "both sides edited" case more gracefully. It lost: DropVoice already has a working fixed-direction gate (`agents-sync`, AGENTS.md is the documented source) and has not hit the failure it solves; the reconciliation machinery stays unported until a real both-sides conflict hurts.

**Skills source at `.claude/skills/` (markpost's direction).** markpost made the Claude path primary because Claude Code is its main tool. It lost here: DropVoice runs four agents with ZCode primary; the tool-neutral `.agents/` path is the honest source, with `.claude/skills/` as the gated mirror.

**Class subfolders and a frozen archive from day one.** Six classes ease browsing at 600 records. It lost: with an empty tree they are empty directories; the flat lifecycle tree carries the inventory. Revival triggers are recorded above.

**Gradual AGENTS.md migration (contract first, content later).** Lower-risk, but it leaves a root file 3× over its own budget as the standing example of the exception. It lost: the content already exists and is current — migration is relocation with links, not authoring — and markpost's precedent shows the one-pass normalize is a bounded, reviewable change. Decided: one pass.

**Bilingual agent instructions and skills.** Consistency argues for pairing everything. It lost: agent instructions are machine-consumed standing orders; doubling them taxes every future rule edit for no reader. Decided scope: DV-RFCs and docs prose pair; instructions, skills, specs, README stay single-language.

**A centralized RFC index.** It lost: the lifecycle tree is the inventory, and generated indexes are a predictable merge hotspot with little discovery value beyond browsing or searching — the source project's recorded reason, confirmed by markpost.

**Do nothing.** The current state demonstrably drifts: a commit skill instructing Go verification survived months in a Rust/pnpm repo because nothing checks it. Prose conventions do not survive agent turnover; enforced gates do.

## Acceptance criteria

1. `python scripts/doc_sync.py` exits 0 over the full corpus; `prek run --all-files` and `prek run --stage pre-push --all-files` exit 0.
2. `verify_dv_rfc_format.py` passes on a tree containing this record (moved to `implemented/`, rewritten as Decision + Consequences) and both backfills — each a bilingual pair.
3. Root `AGENTS.md` ≤ 1,000 words; every layered file ≤ its budget; no orphaned manifest entries.
4. Every fact moved out of the root `AGENTS.md` is reachable by a link from the root or a subtree file; `verify_md_links.py` proves targets resolve.
5. `.claude/skills/` byte-identical to `.agents/skills/`; `commit-old` gone; the rewritten `commit` skill references only commands that exist in this repo.
6. GitHub workflow files unchanged; no in-flight worktree files staged into harness commits.

## Risks

**Fact loss while condensing the root AGENTS.md.** Mitigation: relocation-only with per-section review during rollout; the link gate proves every target exists; the reviewer spot-checks that each removed section has a linked new home.

**Gate false positives on legacy Markdown.** The existing corpus was never held to these rules. Mitigation: run each gate over the full corpus before wiring it into prek; fix the doc, not the gate — if a gate is wrong, change it in the same change and say why here.

**Bilingual upkeep tax.** Every RFC and docs page now edits in two languages. Mitigation: parity is structural (heading sequence, fenced blocks), the scope excludes the highest-churn files (instructions, skills), and full-corpus pairing stays a deferred trigger.

**Wrong budget ceilings.** Ceilings sized by judgment may misfit a file's real need. Mitigation: the raise-with-justification rule makes adjusting cheap and visible; ceilings are guardrails, not reduction targets.

**Collision with in-flight work.** The worktree carries ~50 modified product files (desktop Rust-signaling migration). Mitigation: harness commits stage only harness files; batches are independent of product changes.

**Over-engineering for a small team.** Every deferred tier carries an explicit revival trigger, and everything shipped is exercised by existing flows (commits, pushes, CI), so the mechanism cannot silently rot the way `commit-old` did.
