# IMP-433: Lift RV derived status to a tier engine-side readers can reach

An RV's status is not authored — it is **derived** at read time from its finding
ledger by `review::derived_status_string` (ADR-007 D-C8). `review` is a
`command`-tier module (`.doctrine/adr/001/layering.toml`), so nothing at engine
or leaf tier can ask an RV for its status. `catalog::scan::status_and_title_for`
handles this by special-casing the `RV` prefix — but `catalog::scan` is itself
command-tier and reaches `backlog`, so that special case is unreachable from any
consumer sitting below it.

`SL-238` (`DEC-233`) hit this building the cross-kind status probe for
`backlog list --by sequence`. It resolved it by *declining* to classify: an `RV`
target returns `Unavailable`, classifies `Unrecognised`, and is therefore always
disclosed and never suppressed. That is deliberately conservative and correct as
a failure mode, but it means a cross-kind `needs` onto an `RV` can never say
whether the review is open or concluded.

No backlog item references an `RV` or `REC` target today — all 30 authored
cross-kind `needs`/`after` edges target `QUE` or `SL` — so this is a correctness
edge with no live instance, which is why `SL-238` did not fold it in.

## What would close it

Make an RV's derived status readable from engine tier, so `partition::status_class`
can classify an `RV` target like any other kind and the `Unavailable` arm can
retire. The obstacle is that the derivation reads the authored finding ledger,
which currently lives behind command-tier `review`; the shape of the fix is
whether that ledger read can move down, or whether the derived status should be
materialised where an engine reader can see it.

## Watch for

`DEC-233` pins the derived-status set (`{RV}`) with a test precisely so this
cannot rot silently. A future kind that derives its status and is not added to
that set fails the test rather than degrading quietly — so closing this item
means retiring that test's reason for existing, not deleting it quietly.
