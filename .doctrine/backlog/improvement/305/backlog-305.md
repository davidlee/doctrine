# IMP-305: Candidate staging crash-resume and restage verb

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by the codex adversarial review of **SL-212** (design, pass 2). SL-212
narrows the candidate-create crash window — it writes the `Conflicted` row
*before* the worktree and makes `ledger::store` atomic (temp+rename) so a
crash cannot corrupt the manifest — but it does **not** close the full
crash≡resume contract (PRD-015):

- **Orphan ref (pre-existing in `create` today):** a crash after the branch CAS
  but before the row write leaves a candidate ref with no row; re-running `create`
  loses the zero-OID CAS and refuses.
- **Half-staged worktree:** a crash after the row write but before / during
  materialisation (`read-tree` / `update-index --index-info` / `MERGE_HEAD`)
  leaves a `Conflicted` row + base ref but no resolvable staged conflict; `ingest`
  refuses (`R == base_oid`) and `create` cannot rerun under the same label.

**Ask:** a `dispatch candidate restage <id>` (or `create --resume`) verb that,
given a recorded `Conflicted` row, re-materialises merge-tree's output into the
(re-created if absent) worktree idempotently, and a reconciliation path for an
orphan ref with no row. Mechanically decidable → an in-verb move, not operator
archaeology (RFC-016 §B/§D).

Depends on nothing SL-212-specific; can land before or after. Governed by
ADR-012 (candidate lifecycle) and PRD-015 (crash≡resume). Not a gate on SL-212 —
SL-212's own staging is atomic-durable and rolls back on the failure it controls;
this covers the process-death boundaries it cannot.

Related: [[SL-212]], IMP-303 (audit-OID binding), IMP-304 (supersede clears a
`Failed`/`Pending` trunk row) — the same "invariants-into-verbs, lineage-recorded"
family (RFC-016).
