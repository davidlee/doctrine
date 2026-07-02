# IMP-236: Split-lineage dispatched slice un-closeable: IMP-127 direct-land must record a trunk row

## Context

Surfaced closing SL-190 (dispatched, claude arm). The slice's `review/190` bundle
forked pre-SL-180, so `dispatch candidate create` conflicted and parked
`cand-190-review-001` (`status=conflicted, merge_oid=""`) — the known **IMP-127**
dead-end (no verb ingests a hand-resolved merge). The documented escape
(`mem.pattern.dispatch.split-lineage-close-conflict-direct-land`) is to direct-land
the resolved candidate onto trunk and flip `done`.

That memory claims `done` "waves through" a never-journal-integrated bundle. **It is
now stale.** The SL-126 close-integration gate (`ledger.rs::trunk_integration`)
distinguishes:

- journal **absent / zero rows** → `NotDispatched` (passes), and
- journal **has rows but none target trunk** → `Blocked("no trunk row")`.

A dispatched slice's `dispatch/<N>` journal carries `review/<N>` + `phase/<N>-NN`
rows from the funnel, so the second arm fires. SL-190 direct-landed correctly
(main gate-green, byte-identical to reviewed tip `0eca671a`) yet `reconcile → done`
refused on the missing trunk row.

**Neither integrate path can write that row for a split-lineage slice:**

- `plan_candidate_trunk_row` (candidate workflow active — the conflicted candidate
  row counts) sources the admitted `close_target` OID and **refuses** rather than
  fall back → IMP-127 blocks the admission.
- `plan_trunk_row` (legacy) sources the `phase/<N>-NN` chain tip and requires it to
  **fast-forward trunk** — but split lineage forbids the ff (the exact reason the
  candidate merge existed).

Net: **a split-lineage dispatched slice is un-closeable by any verb** once its phase
rows are journaled.

## Improvement

Give IMP-127's fix (or a sibling) a sanctioned **record-completed-integration**
path: adopt an operator-supplied `(base, source) → landed` merge as the trunk row,
validating `planned_new_oid` is an ancestor of the resolved trunk tip (the true
semantic the gate already checks). Not a `--force` — the integration is real and
still validated; the operator merely performs the merge the auto-path cannot.

SL-190 was closed by **hand-writing** the verified trunk row onto the `dispatch/190`
journal (`source=0eca671a`, `expected_old=7c6dd34c`, `planned=applied=383dad93`,
`status=verified`) — a manual analog of exactly this path. Automate it so the next
split-lineage close does not require raw journal surgery.

## Notes

- Correct the stale memory `mem.pattern.dispatch.split-lineage-close-conflict-direct-land`
  (the "waves through" claim) once this lands.
- Related: IMP-127 (the admit dead-end this compounds).
- Touches `src/dispatch.rs` (integrate / candidate plan), `src/ledger.rs`
  (`trunk_integration`). Coordinate with RFC-005 topology churn.
