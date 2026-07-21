# IMP-304: Let a superseding candidate replace a Failed/Pending trunk journal row

Surfaced by the codex adversarial review of REV-030 (F-4), as a **pre-existing**
dispatch limitation — it predates SL-212.

## The dead-end

`integrate` plans a trunk row only when `fresh` — no journal row yet exists for
that `target_ref`, **status-blind** (`dispatch.rs:2183`). A CAS race *after* the
journal row is written (trunk moved between journal commit and ref mutation) sets
`row.status = Failed` (`dispatch.rs:2263-2264`) and persists it. On the next
`integrate`, `fresh` is false (a row exists), so no replan happens — the stale
`planned_new_oid` is replayed and re-Fails. The documented recovery
(`create --supersedes → admit → integrate`) produces a *new* admitted OID that is
never replanned, so it cannot clear the `Failed` row. Only
`sync --record-integration` (`dispatch.rs:690`) replaces Pending/Failed trunk rows,
and only for an *already-landed* tip.

So the supersede cycle recovers the **plan-time** refusal (trunk moved before the
journal row — the common case) but **dead-ends** on the post-journal CAS race,
forcing `record-integration` or manual journal surgery.

## Direction

Let a superseding `close_target` admission drive a replan that *replaces* a
Pending/Failed trunk row for the same `target_ref` (the row-replacement
`record-integration` already performs, generalised to the not-yet-landed case),
under the same expected-tip CAS. Closes the residual-recovery gap REV-030 accepts.

Relates: REV-030, RFC-006 (shared-trunk-race), SL-211 (record-integration), SPEC-022 FR-005.
