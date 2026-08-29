---
name: writing-dv-rfcs
description: Write, review, move, or supersede DV-RFC decision records (.agents/dv-rfcs/). Use when a non-trivial change needs its decision recorded, when asked to write, review, backfill, or migrate an RFC, or when the verify_dv_rfc_format gate fails.
argument-hint: 'Topic or change needing a decision record'
---

The workflow for DV-RFC decision records. The contract is `.agents/dv-rfcs/README.md` and the tree's standing orders are `.agents/dv-rfcs/AGENTS.md` — read them; don't re-summarize them. This skill is the judgment layer on top.

1. **Check for an existing record first.** Grep `.agents/dv-rfcs/` for the topic and its synonyms. If a shipped record covers it, update that record's facts in the same change (never the decision); if a new decision replaces an old one, write the new record, cross-link both, and only then retire the old verdict. Never rewrite a shipped decision into a different one.
2. **Pick the lifecycle honestly.** `proposed/` before implementation is real (review happens on the record); `implemented/` only for what actually shipped — verify every path, name, and mechanism against the code before moving a record there; `rejected/` only while the rationale still blocks a tempting mistake, otherwise delete it.
3. **Write to the skeleton and the naming rule.** Date = when the topic was first proposed per git history (`git log --reverse -S "<topic>"`); backfills date to the topic's first appearance, not to today. `## Problem` must stand without the solution.
4. **Record alternatives as they were argued.** Every genuine alternative and why it lost, one bold-led paragraph each — including "do nothing". Alternatives invented after the fact are worse than none.
5. **Pair it.** Write `foo.md` and `foo.zh.md` together: same skeleton, English headings and machine tokens in both, fenced code blocks byte-identical. Update both in the same change, always.
6. **Validate before handing off.** `python scripts/doc_sync.py .agents/dv-rfcs/<lifecycle>/<files>` must pass. A gate failure in a record you just wrote is a bug in the record; if the gate itself is wrong, change it in the same change and say why in the record that owns the rule.
7. **Move records mechanically.** `proposed → implemented` rewrites `## Proposal` into a present-tense `## Decision` and folds `## Acceptance criteria` and `## Risks` into `## Consequences`; `proposed → rejected` only adds the one-line reason to the `Status:` line and freezes the file.
