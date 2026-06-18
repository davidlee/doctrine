# REQ-D to REQ-NNN lifecycle — design discovery to close check

The full REQ-D → REQ-NNN lifecycle across five skill touchpoints:

1. **Design** — discovers implied requirements, records them in `design-requirements.toml` as `REQ-DNN` handles (authoritative TOML with handle, statement, kind, home_hint, descends_from); `design.md` `## Implied Requirements` carries one-line summaries only.
2. **Plan** — reads `design-requirements.toml` in sub-step 2a, maps each `REQ-DNN` to verifying phases, records the mapping in `plan.md` `## Requirements verification` as a prose list (not a pipe-table — `plan.toml` `[requirements]` stays empty in v1).
3. **Audit** — reads `design-requirements.toml` in sub-step 4a (orphan survey), surfaces unplaced orphans in the reconciliation brief under `#### Orphaned requirements (REV introduce)` nested under `### Governance/spec (REV)`.
4. **Reconcile** — places orphans via REV step 4f (introduce rows per destination spec; multi-spec placement = sibling REQ-NNNs with traced lineage, not shared REQ-NNN). Records `REQ-DNN → REQ-NNN` mappings in `revision-NNN.md` `### Orphan placements`.
5. **Close** — reads `plan.md` → `review-NNN.md` `## Reconciliation Outcome` → `revision-NNN.md` `### Orphan placements` to confirm every `REQ-DNN` has a `→ REQ-NNN` mapping. Advisory check (not binary-enforced — RV-ledger enforcement is a follow-up IMP).
