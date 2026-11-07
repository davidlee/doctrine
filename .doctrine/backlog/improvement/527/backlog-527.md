# IMP-527: Mechanical acyclicity check over commitment-ledger needs

`SL-287` (`RV-429` F-3) adds an optional `needs` field to commitment-ledger
rows: correctness dependencies between commitments, used to construct a plan's
sequencing, gates and parallel candidates. Acyclicity — and the "never points
forward across phases" rule — are checked by hand in the shipped guidance.

Raised by the user, 2026-11-08: apply cordage over the ledger to guarantee no
cycles.

## Shape, if earned

`RFC-027` `H9`'s reopening condition already names the build shape: graft
slice-local nodes into the existing `cordage` graph for the focus slice through
a different loader and store nothing new (`NodeId(u32)` is opaque and
`Projection<K>` is generic — a widened key plus a loader arm). A ledger reader
would be the first engine consumer of `commitment-ledger.toml`, so it should
land together with, or after, a decision that the ledger earns a reader at all
(`SL-287`'s follow-up trial, tracked in `RFC-029` § 9; `SL-258`; `IMP-409`).

## Not before

Evidence that hand-checking misses cycles or forward edges in practice. The
check itself is cheap; the reader it implies is the commitment.
