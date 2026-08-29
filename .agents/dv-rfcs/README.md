# DV-RFCs — DropVoice decision records

One kind of design doc lives here. A **DV-RFC** records a decision or proposal that affects this codebase — the _why_, the _what we gave up_, the parts code, specs, and `AGENTS.md` cannot carry. They are RFCs written by agents: durable proposals and decision records that preserve rationale, alternatives, and consequences.

## The tree

```
.agents/dv-rfcs/
├── AGENTS.md        standing orders for this tree
├── README.md        this contract (normative)
├── README.zh.md     Chinese twin of this contract
├── implemented/     decisions that shipped, kept current with what shipped
├── proposed/        proposals reviewed before implementation
└── rejected/        declines, kept only while their rationale blocks a tempting mistake
```

The lifecycle tree is the inventory — browse it or grep the repository. There is deliberately **no index file** to maintain: a generated index is a merge hotspot that adds no discovery value.

## Naming

`{lifecycle}/yyyy-mm-dd-topic-title.md` — the date is when the topic was **first proposed**, per git history (a backfilled record dates to the topic's first appearance in the tree, not to when the record was written). The filename grammar is gate-enforced: `^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*(\.zh)?\.md$`.

## The header

The first three lines of every record are exactly:

```markdown
# DV-RFC: <title>

Status: <status>
```

`Status:` agrees with the folder: `proposed`, `implemented`, or `rejected — <why, in one line>`. The status carries no dates and no parentheticals — the filename holds the first-proposed date, git holds everything else. The rejection reason is the one status with content, because a rejected record's verdict is the fact readers come for.

## The skeletons

Each lifecycle has a fixed section skeleton (`…` marks bespoke sections):

```markdown
#### proposed/

## Problem

## Proposal

…

## Alternatives considered

## Acceptance criteria

## Risks

#### implemented/

## Problem

## Decision

…

## Alternatives considered

## Consequences

#### rejected/

(keeps whatever proposal-time sections it had, frozen; the verdict lives on the Status: line)
```

`## Problem` opens the body and is written to stand without the solution. Proposal-era headings (`## Proposal`, `## Plan`, `## Migration plan`, `## Acceptance criteria`) are banned in `implemented/` — the format gate rejects them.

## Alternatives considered is mandatory

Every record carries an `## Alternatives considered` section: one bold-led paragraph per genuine alternative and why it lost (a `### Why not <X>?` subsection is fine for contested ones). Alternatives are recorded as they were argued, never invented after the fact. A decision recorded without what it beat invites re-litigation — the failure DV-RFCs exist to prevent.

## Lifecycle moves

- `proposed → implemented`: rewrite `## Proposal` into a present-tense `## Decision`, fold `## Acceptance criteria` and `## Risks` into `## Consequences`, and verify every path, name, and mechanism against what actually shipped.
- `proposed → rejected`: add the reason to the `Status:` line and freeze the file.
- Never edit a record into a different decision — supersede it and cross-link both.

Implemented records stay current: when later code moves a file, renames a package, or changes a key/default, the record is updated in the same change (facts only, never the decision itself).

## Bilingual pairs

Every record is `foo.md` + `foo.zh.md`: same skeleton, English headings and machine tokens in both, updated in the same change. The zh twin uses ASCII anchors (`<a id="…">`) when it needs fragment links, so links resolve in both languages. Pair parity — identical heading sequences and byte-identical fenced code blocks — is gate-enforced.

## When to write one

Every non-trivial change adds or updates at least one record in the same change. Only a purely mechanical or local edit with no change to behavior, contracts, structure, process, or rationale is exempt. Rationale goes here; procedures go in `docs/`; current-state facts go in `docs/architecture.md` or `AGENTS.md`; approved frozen designs live in `specs/`.

## Gating

`python scripts/verify_dv_rfc_format.py [files…]` enforces this contract (filename grammar, header, Status-folder agreement, skeletons, mandatory Alternatives, proposal-speak ban, pair parity, stray-file detection). `python scripts/doc_sync.py` aggregates it with the other documentation gates; prek runs both on Markdown changes. Fix the doc, not the gate — if the gate is wrong, change it in the same change and say why in the record that owns the rule.
