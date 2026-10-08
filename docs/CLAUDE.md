# AGENTS.md — The documentation standard

This file defines the document tiers, writing rules, and the documentation-gate budgets; [docs/i18n/README.md](i18n/README.md) owns the bilingual pairing contract itself. Use [documenting](../.agents/skills/documenting/SKILL.md) for placement and validation, and [editing-prose](../.agents/skills/editing-prose/SKILL.md) for required coverage and editorial judgment.

## Document structure

A document's subject and tree position fix its scope: describe its own subject at appropriate detail and direct children only by purpose, responsibility, and high-level behavior; link to the owning descendant for lower-level detail. Document type does not widen that scope. A reference may be exhaustive only about its own subject.

Classify every in-scope document as a tutorial or a reference. Tutorials follow an ordered path to an outcome and introduce only what each step needs. References define a lookup scope and current behavior without a teaching sequence. Separate substantial tutorial and reference content; label a section when either part is small.

## The tier taxonomy: one home per fact

Each fact has one home: the tier whose job it is; elsewhere, link there.

| Tier                                                                  | Job                                                                                                                                                 | Does NOT belong there                                                                                     |
| --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Root `AGENTS.md`                                                      | Standing orders an agent needs in context in every session, linking its home                                                                        | Stories, worked examples, situational procedures, anything restated from a linked home                    |
| Subtree `AGENTS.md` (`docs/`, `apps/*`, `packages/`, `.agents/rfcs/`) | Orders specific to that subtree                                                                                                                     | Repo-wide rules the root file already carries                                                             |
| [architecture.md](architecture.md)                                    | Current-state system facts: composition, stack, config single-sources, flows; read before changing the source tree                                  | Per-domain detail (→ the owning document), decision rationale (→ RFCs), implementation-status annotations |
| [development.md](development.md)                                      | Contributor setup, daily workflow, gates usage, migrations, deploy, and a summary of CI; a bilingual pair under the [i18n contract](i18n/README.md) | Runtime/version rationale (→ RFCs), check-by-check lists that drift from the command inventory            |
| [cookbook/](cookbook/responding-to-pr-review-on-a-stack.md)           | Step-by-step how-tos with observable verify steps                                                                                                   | Design rationale (→ the RFC each guide links)                                                             |
| `.agents/rfcs/**`                                                     | Decision records: the why, what was given up, required verification ([rules](../.agents/rfcs/README.md))                                            | Current-state contracts (→ docs), procedures (→ cookbooks)                                                |
| Skills (`.agents/skills/`)                                            | Reusable workflow instructions                                                                                                                      | Product and runtime contracts (→ docs or source)                                                          |
| `specs/`                                                              | Approved frozen design specs — read-only; changes proposed via discussion                                                                           | Anything else (the tree never changes in place)                                                           |
| `apps/pairing-server/devops/runbooks/`                                | Operational acceptance runbooks                                                                                                                     | Development procedures (→ development.md)                                                                 |
| `README.md`                                                           | Repo front page, a bilingual pair                                                                                                                   | Standing orders, tier detail                                                                              |
| Generated files (`openapi.*`, `theme.css`)                            | Regenerated, never hand-edited; guarded by drift gates                                                                                              | —                                                                                                         |

Placement: rationale → RFCs; procedures → development.md or cookbooks; contracts → the owning reference; standing orders → root `AGENTS.md` with a rationale link.

## Pairing rules

- Every in-scope document is an English + Simplified Chinese pair — `foo.md`, `foo.zh.md`, and the `foo.i18n.yaml` record in the same directory — under the [pairing contract](i18n/README.md), enforced by `hdsh pairing verify`.
- Instruction files are English-only and exempt from pairing: every `AGENTS.md`/`CLAUDE.md` mirror and everything under `.agents/skills/`; never create a `.zh.md` or `.i18n.yaml` for them.
- Pairs update together in one PR: a PR that changes either side updates the other side terminology-guided and re-records with `hdsh pairing record <pair>`.
- Archived RFC triplets are frozen and outside the pairing corpus ([archive policy](../.agents/rfcs/archived/AGENTS.md)).

## Writing rules

- Current-state prose: document what is, not change history; put change stories in RFCs and commits.
- Concise titles that name their subject.
- Write directly: name actors and facts instead of metaphorical placeholders. Name the exact check, command, type, or behavior instead of a generic label.
- One physical line per prose paragraph, enforced by `hdsh docs wrap`; use editor soft-wrap. Code blocks, tables, and list structure keep their formatting.
- Cross-reference documents by relative Markdown link, never bare prose or a number, so `hdsh docs links` can prove every target and `#fragment` resolves ([rationale](../.agents/rfcs/implemented/process/2026-09-07-document-corpus-gates.md)).
- Standing docs stay under the `wc -w` ceilings in `.hdsh/docs.manifest.json` (`hdsh docs budgets --list`); relocate or condense instead of raising a ceiling.
- Files end with exactly one trailing newline.

## Wordcount budgets

`.hdsh/docs.manifest.json` sets standing-doc ceilings; `hdsh docs budgets` rejects excess or missing files.

When the gate goes red:

1. **Relocate** content that belongs in another tier; leave a one-line link if needed.
2. **Condense** content that belongs here but can be shorter.
3. **Raise** the ceiling only when the words need the space; justify the manifest diff in the PR. A too-low ceiling is a budget bug.

Ceilings are guardrails, not reduction targets. At or below target, retain at least 5% headroom; above target, freeze the ceiling until relocation or condensation brings the document under target. Review governs unbudgeted documents.

## The slop checklist

Hunt these in any document:

- The same rule stated in more than one home. Grep a distinctive phrase; keep one home and link the rest.
- Narrated history or war stories: "previously", "now", "no longer", "used to", "renamed", "was moved", PRs, or commits. State the current fact; link an RFC when needed.
- Implementation-status annotations in prose or diagrams ("implemented!", "future: …"). Status rots; the repo layout and the command inventory carry it.
- Hand-restated catalogs, inventories of files, gates, or tests when source or a gate output is authoritative.
- Reasoning transcripts: step-by-step implementation narration, proof of obvious branches, test walkthroughs, or rejected local alternatives. Keep the resulting contract or durable rationale; delete the path used to derive it.
- Rationale repeated beside sibling methods instead of once at the owning capability.
- Paragraph walls: one paragraph carrying several rules and parenthetical asides. Split it or demote the detail to its home.
- Emphasis inflation: bold, CAPS, or "critically" everywhere means nothing stands out. Reserve emphasis for the clause that changes behavior.
- Spec-speak in implemented RFCs: "should", migration plans, acceptance checklists. An implemented RFC describes what is.
