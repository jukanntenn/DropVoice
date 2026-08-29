# DV-RFC: Agent harness — dv-rfcs, layered agent instructions, docs tiers, local gates

Status: implemented

## Problem

DropVoice is developed primarily by AI coding agents. The plumbing already exists and stays as-is: prek is the single source of format and lint, and the four tool directories (`.claude/`, `.zcode/`, `.codex/`, `.opencode/`) are thin adapters over it. What is missing is everything above the plumbing, and the gaps are observable today:

1. **Decision rationale has no durable home.** Why signaling relays through a public pairing-server instead of LAN discovery, why device tokens are stored as hashes only, why prek won over per-tool hook logic — these live in commit messages, chat transcripts, and prose buried inside the root `AGENTS.md`. A future change is never forced to confront what a decision beat, so every re-litigation starts from zero. Both reference projects name the failure this invites: a decision recorded without what it beat invites re-litigation.
2. **The root `AGENTS.md` mixes standing orders with reference manuals.** At 3,438 words it loads dev-loop tasks, deploy runbooks, configuration tables, the pairing-flow walkthrough, and conventions into every session — 3–4× the ceiling either reference project found sustainable. Subtree-specific orders (the desktop WebRTC answerer, pairing-server migration rules) have no home closer than the root.
3. **Markdown carries no mechanical contract.** Relative links rot invisibly, nothing bounds document growth, and skills drift silently: `.agents/skills/commit-old` still tells agents to verify with Go toolchains (`go test`, `golangci-lint`) that do not exist in this repository — a stale copy from markpost that survived because nothing checks it.

Two reference repositories informed this decision. deepseek-harness is the mechanism's source (agent notes, skills, layered agent instructions, doc tiers, a TypeScript gate suite). markpost adapted it successfully, and its decision history records the governing lesson: **port pieces only when a trigger signal appears, never wholesale** — the reference project's scale is the ceiling of where this road leads, not the starting point.

## Decision

The harness ships as four components. All new gates are stdlib-Python scripts under `scripts/`, wired through prek (the existing single gate source); CI inherits them with zero workflow changes. The authoritative doc-tier table and slop checklist live in `docs/AGENTS.md`; this record fixes the decisions behind them.

### 1. dv-rfcs — decision records under `.agents/dv-rfcs/`

A DV-RFC records a decision or proposal that affects this codebase: the why, the what-we-gave-up, and the consequences. Placement follows the reference projects: `.agents/` is the directory agent tools treat as their load path.

- **Tree**: `implemented/`, `proposed/`, `rejected/` — flat lifecycles, no class subfolders, no archive. The lifecycle tree is the inventory; there is deliberately no index file to maintain.
- **Naming**: `{lifecycle}/yyyy-mm-dd-topic-title.md`. The date is when the topic was first proposed, per git history; the filename regex `^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*(\.zh)?\.md$` is gate-enforced.
- **Format** (gate-enforced): first line `# DV-RFC: <title>`, third line `Status: proposed | implemented | rejected — <one-line reason>` agreeing with the folder. Body opens with `## Problem` stated to stand without the solution. `## Alternatives considered` is mandatory in every record — one bold-led paragraph per genuine alternative, recorded as argued, never invented after the fact.
- **Skeletons**: `proposed/` carries `## Proposal`, `## Alternatives considered`, `## Acceptance criteria`, `## Risks`. `implemented/` carries `## Decision` (present tense, what shipped), `## Alternatives considered`, `## Consequences`; proposal-era headings are banned there. `rejected/` keeps its proposal-time sections frozen; the verdict lives on the `Status:` line.
- **Lifecycle moves are mechanical**: `proposed → implemented` rewrites Proposal into present-tense Decision and folds Acceptance criteria and Risks into Consequences; `proposed → rejected` only adds the reason to the `Status:` line. Never edit a record into a different decision — supersede it and cross-link both.
- **Bilingual pairs**: every record is `foo.md` + `foo.zh.md`, same skeleton, English headings and machine tokens, updated in the same change. Structural parity (heading sequence, byte-identical fenced blocks) is gate-enforced.
- **Backfills** (2, `implemented/`, dates = first git appearance): `2026-08-14-prek-single-source-of-format-and-lint` and `2026-08-14-webrtc-datachannel-public-pairing-server`. The in-flight desktop Rust-signaling migration is not backfilled — it writes its own record when it lands.
- **Contract files**: `.agents/dv-rfcs/AGENTS.md` (standing orders) + `README.md`/`README.zh.md` (normative contract: naming, lifecycle, skeletons, pairing).
- **Deferred with revival triggers**: class layer (records > 50), frozen archive + hash manifest (superseded backlog), i18n.yaml hash sidecars, postmortem tier (first costly incident).

This record is itself the first DV-RFC; it ships in `implemented/`, rewritten as Decision + Consequences — the rules bound their own introduction.

### 2. Skills — `.agents/skills/` is the source, `.claude/skills/` the mirror

- Source of truth is the tool-neutral path `.agents/skills/` (this repository's primary tooling is not Claude Code); `.claude/skills/` is a byte-identical mirror, gate-enforced like the AGENTS/CLAUDE pairs.
- The stale `commit-old` (a markpost copy referencing Go toolchains) is deleted; a fresh `commit/` skill aligns with this repo's stack (pnpm + cargo + prek verification, dropvoice scopes, never `--no-verify`).
- `writing-dv-rfcs/` carries the RFC workflow (duplicate check, lifecycle choice, skeleton, bilingual pairing, `python scripts/doc_sync.py <files>` validation); a lean `doc-standards/` carries doc placement tiers and how to respond to gate failures.
- `grill-me`, `iterating`, `tauri-icon-generation` stay unchanged.

### 3. Layered AGENTS.md with word budgets

The root file condenses from 3,438 words to 844 words of standing orders — each rule 1–3 lines linking its home. Subtree files carry subtree-specific orders and never repeat the root. Every `AGENTS.md` gets a byte-identical `CLAUDE.md` twin, fixed direction (AGENTS.md is the source; `python scripts/sync_agent_files.py` is the fixer), extending the existing `agents-sync` gate to all eight pairs plus the skills mirror. Budgets are gate-enforced (`verify_doc_budgets.py` + `doc_budgets.manifest.json`; a manifest entry whose file disappears fails, so renames cannot orphan budgets):

| File                            | Ceiling (words) | Shipped |
| ------------------------------- | --------------- | ------- |
| Root `AGENTS.md`                | 1000            | 844     |
| `apps/desktop/AGENTS.md`        | 500             | 283     |
| `apps/mobile/AGENTS.md`         | 300             | 179     |
| `apps/pairing-server/AGENTS.md` | 450             | 284     |
| `apps/landing/AGENTS.md`        | 150             | 105     |
| `packages/AGENTS.md`            | 250             | 171     |
| `docs/AGENTS.md`                | 600             | 497     |
| `.agents/dv-rfcs/AGENTS.md`     | 150             | 139     |

On a red budget: relocate to the owning tier, condense, raise the ceiling last with a justified manifest diff in the same change.

### 4. docs/ — tiered prose, bilingual pairs

The new root `docs/` absorbs the reference material that left the root `AGENTS.md`, so no fact is lost — every relocated fact keeps a link from its old home:

- `docs/development.md` + `.zh.md` — full command/environment detail (dev tasks, doctor, quality gates, code-style examples, migrations, deploy, release).
- `docs/architecture.md` + `.zh.md` — system composition: structure tree, tech stack, pairing flow, signaling supervision, connection-state event bus, ports, configuration single-source table.
- `docs/AGENTS.md` — the documentation standard itself: tier table (one fact, one home), rules each naming its gate, slop checklist, language policy.
- Deployment content stays in its existing home (`apps/pairing-server/devops/` + runbooks) — classified in the tier table, not moved. `specs/` (read-only) and `PRINCIPLES.md` are classified only, untouched.

Bilingual scope (decided): DV-RFCs and `docs/` prose pages are pairs. Exempt: all `AGENTS.md`/`CLAUDE.md`, skills, generated files (openapi), `specs/`, `README.md`. Inside pairs, fenced code blocks are byte-identical — code and its comments stay English in both languages.

### 5. Gates — stdlib Python behind prek

`scripts/doc_sync.py` aggregates, accepting file arguments (staged locally via prek) or running full-corpus (CI via `prek run --all-files`):

1. `verify_dv_rfc_format.py` — filename regex, title, Status-folder agreement, `## Problem` opener, mandatory Alternatives, per-lifecycle skeletons, proposal-speak ban in `implemented/`, `.zh.md` requires its original plus structural parity, stray-file detection.
2. `verify_doc_pairs.py` — pairing completeness both directions, heading-depth parity, byte-identical fenced blocks, over the pairing scope defined by `doc_languages.manifest.json` (patterns + exclusions); agent instructions are categorically single-language.
3. `verify_doc_budgets.py` — the table above.
4. `verify_md_links.py` — relative markdown links and fragment anchors resolve.

prek wiring (root `prek.toml`): the `doc-check` hook (check group, `\.md$`, excluding the four tool-adapter dirs — never `.agents/dv-rfcs/`); `agents-sync` covers all AGENTS↔CLAUDE pairs plus the skills mirror, with `scripts/sync_agent_files.py` as the fixer. The prettier/whitespace exclusion narrows from `^(\.agents|\.claude|\.codex|\.opencode|\.zcode)/` to the four tool dirs only — wholesale-excluding `.agents/` lets gates pass green over real content (markpost's recorded enforcement-parity lesson). `pnpm docs:check` / `docs:fix` aliases.

### 6. Rollout — three batches, harness-only staging

Shipped as three conventional commits (`docs` scope), each staging only harness files: the dv-rfcs mechanism + gates + backfills; the layering + docs + budget/link/pair gates; the skills finish + this record's move to `implemented/`. The ~50 in-flight product files in the worktree (desktop Rust-signaling migration) were never staged into harness commits.

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

## Consequences

- `python scripts/doc_sync.py` exits 0 over the full corpus, and every prek hook (prettier, agents-sync, lockfiles-fresh, doc-check, commitlint) passed on each harness commit. The exhaustive `prek run --all-files` format pass and the pre-push test suites ride on the in-flight work's own commit — the ~50 modified product files own that run, not the harness.
- The root `AGENTS.md` went from 3,438 to 844 words; every layered file ships within budget (see the table above); the word count of every relocated fact is reachable from the root or a subtree file, and `verify_md_links` proves the targets. Two directory-style links were found and fixed during rollout (one in the new architecture pages, one in `README.md`).
- The gates caught real violations on day one, including in the mechanism's own files: `verify_doc_pairs` flagged translated comments inside the zh twin's fenced blocks (blocks are byte-identical by rule — code comments stay English), and a fence-stripping bug shared by three gate scripts was exposed by the same run and fixed in the same change.
- Skills converge to six (`commit`, `doc-standards`, `grill-me`, `iterating`, `tauri-icon-generation`, `writing-dv-rfcs`); `.claude/skills` is a byte-identical mirror; `commit-old` is gone and the rewritten `commit` skill references only commands that exist here.
- CI workflow files are unchanged — the new hooks reach CI through prek, and `docs:check`/`docs:fix` are the human entries. Each gate was negative-tested (bad date, missing twin, stray index file, broken link) before being trusted.
- Bilingual upkeep is structural: parity gates hold the pair together while prose stays free; the pairing scope deliberately excludes the highest-churn files. Full-corpus pairing (README, runbooks) remains a deferred trigger.
- Deferred tiers carry revival triggers: class layer at >50 records, frozen archive + hash manifest on a superseded backlog, i18n.yaml hash sidecars, postmortems on the first costly incident. Everything shipped is exercised by existing flows (commit, push, CI), so the mechanism cannot silently rot the way `commit-old` did.
