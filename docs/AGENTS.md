# Documentation standard

The rules for where a fact lives and in which language. Rules bind their gate: each entry names the script that mechanically enforces it. Placement doubts are resolved by the tier table — when two tiers seem to own a fact, the more specific one wins and the other links to it.

## Tiers

| Tier                                        | Job                                                                       |
| ------------------------------------------- | ------------------------------------------------------------------------- |
| Root `AGENTS.md`                            | Standing orders: rules every agent needs every session, 1–3 lines each    |
| Subtree `AGENTS.md` (`apps/*`, `packages/`) | Orders specific to that subtree; never repeats the root                   |
| [DV-RFCs](../.agents/dv-rfcs/README.md)     | Decision records: the why, the alternatives that lost, the consequences   |
| [architecture.md](architecture.md)          | Current-state system facts: composition, stack, config sources, flows     |
| [development.md](development.md)            | Procedures: commands, gates usage, migrations, deploy, release            |
| `specs/`                                    | Approved frozen design specs — read-only; changes proposed via discussion |
| `PRINCIPLES.md`                             | Agent behavioral constraints (values) — frozen archive                    |
| `apps/pairing-server/devops/runbooks/`      | Operational acceptance runbooks                                           |
| `README.md`                                 | Repo front page                                                           |
| Skills (`.agents/skills/`)                  | Reusable workflows; not product or runtime contracts                      |
| Generated files (`openapi.*`, `theme.css`)  | Regenerated, never hand-edited; guarded by drift gates                    |

Placement rule: rationale → DV-RFCs; procedures → development.md; system facts → architecture.md; standing orders → root `AGENTS.md` linking its home; bugs/incidents → a DV-RFC or a future postmortem tier (deferred).

## Rules

1. **One fact, one home.** Each fact lives in the tier whose job it is; everywhere else links there. A rule restated in two homes has already diverged. _(gate: verify_md_links keeps the links honest; duplication itself is review's to hunt)_
2. **Current state, not change history.** No "previously/now" narration, no PR or commit references in prose; a reader at HEAD, with no transcript access, can verify every claim. Change stories belong in commits and DV-RFCs. _(review)_
3. **Bilingual pairs where humans read prose.** DV-RFC records and `docs/*.md` prose ship as `foo.md` + `foo.zh.md` — same skeleton, fenced code blocks byte-identical, updated in the same change. _(gate: verify_doc_pairs, verify_dv_rfc_format)_
4. **English for machine-consumed instructions.** All `AGENTS.md`/`CLAUDE.md`, skills, and `docs/AGENTS.md` stay English-only — doubling them taxes every rule edit for no reader. `README.md` stays as-is. _(gate: verify_doc_pairs exclusion manifest)_
5. **Word budgets guard instructions.** `AGENTS.md` files carry ceilings (`scripts/doc_budgets.manifest.json`); on red: relocate → condense → raise the ceiling last, with a justified manifest diff. A missing budgeted file fails (renames cannot orphan budgets). _(gate: verify_doc_budgets)_
6. **Links must resolve.** Cross-reference with machine-checkable relative links, never free prose ("see the architecture doc" ✗, `[architecture.md](architecture.md)` ✓). _(gate: verify_md_links)_
7. **Fix the doc, not the gate.** If a gate is wrong, change it in the same change and say why in the record that owns the rule. _(review)_

## Slop to hunt

Duplicated rules in two homes; narrated history; implementation-status annotations ("done", "TODO", "planned"); hand-restated catalogs of what code or generated files already state; paragraph walls; emphasis inflation; spec-speak in implemented DV-RFCs; indexes of things the tree already lists.
