---
name: doc-standards
description: Place, edit, and repair documentation under the tier system — deciding where a fact lives, which documents pair bilingually, and how to respond to a failing documentation gate (doc_sync, doc pairs, budgets, md links). Use when writing or moving docs, or when verify_doc_* / verify_md_links gates fail.
---

The workflow form of the documentation standard. The standard itself is [docs/AGENTS.md](../../../docs/AGENTS.md) — read it, don't re-summarize it; this skill is how to operate it.

**Placing a fact.** Match the fact to the tier's job: rationale → a DV-RFC; procedures → docs/development.md; system facts → docs/architecture.md; subtree orders → that subtree's AGENTS.md; standing orders → root AGENTS.md (1–3 lines, linking the home). When in doubt, the more specific tier wins and the general one links to it. Never restate a fact that already has a home — link it.

**Writing.** Document current state, not change history: a reader at HEAD with no transcript access can verify every claim. No "previously/now", no PR/commit references, no status annotations. Hunt the slop list in docs/AGENTS.md before finishing: duplicated rules, narrated history, hand-restated catalogs, paragraph walls, emphasis inflation.

**Language.** Prose pages (docs/*.md) and DV-RFC records ship as `foo.md` + `foo.zh.md` — same skeleton, fenced code blocks byte-identical (code and its comments stay English), updated in the same change. Agent instructions (all AGENTS.md/CLAUDE.md, skills) are English-only.

**Responding to a red gate.** `pnpm docs:check` (or `python scripts/doc_sync.py <files>`) names the gate:

- `verify_md_links` — fix the link or the target; directory links get a file; anchors must match a real heading.
- `verify_doc_pairs` — restore the missing twin or restore parity (heading depths, fenced blocks); if the page shouldn't pair, that's a manifest decision (scripts/doc_languages.manifest.json) made in a DV-RFC, not an ad-hoc skip.
- `verify_doc_budgets` — relocate to the owning tier, condense, raise the ceiling last with a justified manifest diff.
- `verify_dv_rfc_format` — see the writing-dv-rfcs skill.

Fix the doc, not the gate. If the gate itself is wrong, change it in the same change and say why in the record that owns the rule (docs/AGENTS.md or the DV-RFC that introduced the gate).

**Mirrors.** After touching any AGENTS.md, CLAUDE.md, or skill: `python scripts/sync_agent_files.py` rebuilds the twins and the `.claude/skills` mirror; the `agents-sync` gate rejects drift either way.
