# IMP-479: Decide and build the D-C9b close-gate reverse lookup

ADR-007 **D-C9b** (target close refuses on an unresolved blocker in an active
review) is a corpus scan over RV `[target].ref` — *"feasible, but built, not
reused"* (ADR-007 Consequences/Negative; R2 standing). No reverse-relation index
exists (ADR-004 stores outbound-only).

Decide whether the scan stays, or a reverse index / materialised status is built,
before review count makes it a cost. Interacts with `IMP-433` (derived-status
reach) and the `ISS-314` empty-ledger change. See RFC-032 `research.md` F6.

## Outcome (2026-09-26) — closed, not needed until measured

RFC-032 decision frontier **D12**, as carried by SL-268: the reverse index is not
built until it is needed.

- **The scan stays.** The D-C9b close gate remains a corpus scan over RV
  `[target].ref` (`review_ledger::gate::unresolved_blockers_for`). It is called
  on the slice closure-seam moves (`audit → reconcile`, `reconcile → done`) and
  costs O(number of reviews) per close. No reverse index and no materialised
  status is built.
- **Recorded in governance.** SPEC-032 (Review ledger) states the scan and its
  cost. REV-064 rewords ADR-007's Consequences sentence to match. REV-064 is
  proposed and applied at reconcile.
- **Reopen trigger.** Reopen this item if a measurement shows the scan costs:
  close latency that grows with the review count, or a caller that needs the
  reverse lookup outside close.
