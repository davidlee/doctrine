# IDE-003: Revision vehicle: staged draft/approve of requirement+spec-prose deltas, distinct from REC

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced in SL-042 design (SPEC-002 observe substrate), resolving Q3 (REC status).

**The distinction.** A **REC** (SPEC-002 D1) is the *immutable record of a
reconciliation act* — status-less, append-only, "exactly one per act". It is **not**
the place to *draft and approve* the deltas themselves. The vehicle for staged,
reviewable changes against requirement truth and **spec prose** is a separate
**Revision** entity — the spec-driver-heritage concept ADR-003 names as missing
("no tech specs, no revisions, no reconciliation verb").

**Why deferred, not built now.** Human authorship/approval of reconciled truth is
**case-by-case, team-by-team** — it belongs on the ADR-009 conduct axis, not baked
into a fixed REC lifecycle. SL-042 therefore ships REC status-less; a draft→approved
workflow, if a team wants one, lands on a future Revision vehicle, additively.

**Scope when picked up.** A Revision kind/record that stages requirement-status and
spec-prose deltas for review before they land in the authored tier; couples to the
SPEC-002 reconcile writer (Slice B, the "revise → spec-truth write" move) and the
conduct axis. Distinct from REC; distinct from the RV review-ledger (SL-040).

Related: SPEC-002 (D7 revise move), ADR-003 (revisions named-missing), ADR-009
(conduct axis), SL-040 (RV sibling — not this), Slice B (reconcile writer).
