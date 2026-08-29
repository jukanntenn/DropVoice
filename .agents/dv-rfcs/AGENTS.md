# DV-RFC standing orders

Records here are DropVoice's decision records: the why, the alternatives that lost, the consequences. The normative contract — naming, lifecycle, skeletons, bilingual pairing — is [README.md](README.md); the gate is `python scripts/verify_dv_rfc_format.py`.

- Before writing a new record, grep this tree for the topic: supersede and cross-link, never restate or rewrite a shipped decision.
- Lifecycle moves are mechanical — `proposed → implemented` rewrites Proposal into a present-tense Decision and folds Acceptance criteria and Risks into Consequences; `rejected` freezes the file and puts the verdict on the `Status:` line.
- English and `.zh.md` twins update in the same change; headings and machine tokens stay English in both.
- A non-trivial change adds or updates a record in the same change. Only purely mechanical edits with no change to behavior, contracts, structure, process, or rationale are exempt.
