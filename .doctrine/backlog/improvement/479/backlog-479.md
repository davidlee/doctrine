# IMP-479: Decide and build the D-C9b close-gate reverse lookup

ADR-007 **D-C9b** (target close refuses on an unresolved blocker in an active
review) is a corpus scan over RV `[target].ref` — *"feasible, but built, not
reused"* (ADR-007 Consequences/Negative; R2 standing). No reverse-relation index
exists (ADR-004 stores outbound-only).

Decide whether the scan stays, or a reverse index / materialised status is built,
before review count makes it a cost. Interacts with `IMP-433` (derived-status
reach) and the `ISS-314` empty-ledger change. See RFC-032 `research.md` F6.
