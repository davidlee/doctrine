# CHR-074: SL-246 C7 — Facets level reads the full record, not just record-NNN.toml

SL-246's design states at `design.md:737`: "`Facets` reads only
`record-NNN.toml`. This is `C7` discharged."

The implementation does not do this. PHASE-03 reads the whole record via
`read_record`, so the `facets` verbosity level pays to load a prose body it then
discards. **`C7` is not discharged by SL-246**, and no criterion in any of its
six phases carries it — `grep C7 plan.toml` returns nothing.

## Why this was deferred rather than fixed in-slice

Ruled by the capsule-driver seat during SL-246 PHASE-03/04, on three grounds:

1. **Behaviourally invisible.** Pruned records never reach the renderer, so no
   output differs and no test can catch it either way. It is an IO cost, not a
   correctness or output-size property.
2. **It is not where the feature's value lives.** `C7` saves disk reads; the
   feature's measured saving is in *rendered bytes* — `facets` costs 12.3% of
   `full` on the SL-244 specimen (907 / 13,780 / 112,425 bytes at
   skip / facets / full). That saving is already banked without `C7`.
3. **Cheap to discharge later.** The full read is kept behind a **single call
   site**, `read_selected` (`src/knowledge.rs:2562`), so a facet-only path is one
   function's change rather than a refactor.

## Reconciliation input

This is a live design-vs-tree divergence and `/reconcile` must record it as such
rather than let the slice close claiming `C7` discharged. The honest disposition
is that `C7` was descoped with the cost measured and the remedy isolated — not
that it was met.

Surfaced during SL-246 PHASE-03/04 (capsule-driver).
