# RFC: Retiring the legacy documentation standard for the hdsh harness

Status: implemented

English | [中文](2026-10-07-retiring-the-legacy-documentation-standard.zh.md)

## Problem

DropVoice ran its own documentation governance: a `doc-check` prek gate aggregating four verifier scripts (dv-rfcs format, doc pairs, word budgets, Markdown links), two manifests (`scripts/doc_budgets.manifest.json`, `scripts/doc_languages.manifest.json`), a bespoke `.agents/dv-rfcs/` decision-record tree with its own header format, and two repo-owned skills (`doc-standards`, `writing-dv-rfcs`) documenting that machinery. Adopting [the hdsh harness](2026-10-07-adopting-the-hdsh-harness.md) installs a second, opinionated standard — pairing records, RFC format and archive gates, docs wrap/links/budgets — and hdsh adopt does not run a dual standard beside a repository's own gates. Two parallel gate sets would double every documentation change's cost and drift apart exactly the way the old gates were built to prevent.

## Decision

The legacy standard is retired in the same adoption, per the hdsh adoption manual's retirement policy:

- All seven dv-rfcs records moved into `.agents/rfcs/` as complete bilingual triplets (`md` + `zh.md` + `.i18n.yaml`), classified into the harness class folders (`process`, `architecture`, `bug-fix`); the `DV-RFC:` title prefix became `RFC:`, and each record gained the language switcher the pairing contract requires. The `.agents/dv-rfcs/` tree, its README pair, and its AGENTS/CLAUDE mirrors are deleted.
- The word ceilings in `scripts/doc_budgets.manifest.json` folded into `.hdsh/docs.manifest.json` `docBudgets`; the subtree `AGENTS.md` budgets carry over unchanged and the retired `.agents/dv-rfcs/AGENTS.md` budget drops.
- The bilingual corpus moved under the pairing contract: `docs/architecture.md`, `docs/development.md`, and every migrated record gained switcher lines and recorded consistency sidecars; the Chinese-first load-test README flipped (`README.zh.md` keeps the original bytes); the root README and the icons README gained Chinese counterparts; `docs/CLAUDE.md` — a regenerated agent mirror — is excluded in `.hdsh/pairing.manifest.json`.
- The `doc-check` hook, the five gate scripts it ran, and the two manifests are deleted; the `hdsh` prek group owns documentation gating. `pnpm docs:check` is removed from `package.json`.
- The `doc-standards` and `writing-dv-rfcs` skills are deleted; [documenting](../../../skills/documenting/SKILL.md), [editing-prose](../../../skills/editing-prose/SKILL.md), and [archiving-rfcs](../../../skills/archiving-rfcs/SKILL.md) own the same workflows under the new standard, and [docs/AGENTS.md](../../../../docs/AGENTS.md) restates the tier taxonomy against it.

## Alternatives considered

**Run both standards side by side.** Keep `doc-check` for the old corpus and the hdsh gates for the installed one. It lost: every in-scope document answers to two pairing/budget/link rules that disagree in detail (switchers, sidecar format, wrap), and the hdsh adopt manual explicitly refuses the dual standard.

**Port the legacy gates to output hdsh-compatible records.** Rewrite the four verifiers to emit `.i18n.yaml` sidecars and the RFC header format. It lost: it forks every rule hdsh already ships — the exact duplication the harness exists to remove — and locks the repo out of upstream fixes.

**Defer retirement until the adoption settles.** Adopt the harness, retire the old gates in a later change. It lost: until retirement, both gate sets run red against each other's conventions (wrap rules, switcher requirements, record formats), and the adoption PR cannot land green.

## Consequences

- Documentation governance has one source: the `hdsh` prek group plus [docs/AGENTS.md](../../../../docs/AGENTS.md) and the [pairing contract](../../../../docs/i18n/README.md). Gate behavior changes by rerunning `hdsh adopt apply` under a newer ref or by redirecting changes upstream — never by forking the installed files.
- Decision-record URLs change: inbound links to `.agents/dv-rfcs/...` were rewritten in the same change, and the two specs' references aside, the migration is link-clean per `hdsh docs links`.
- The AGENTS.md word-budget gate is now `hdsh docs budgets` over `.hdsh/docs.manifest.json`; the ceiling values carried over, so no document had to shrink.
- Repo-local skills no longer document documentation workflow; agents follow the harness skills mirrored into every tool directory by `scripts/sync_agent_files.py`.
