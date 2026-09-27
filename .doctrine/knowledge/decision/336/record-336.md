A recorded act displaced by a same-kind recording emits `act_invalidated` only when the displaced act's death has not already been reported: it was live immediately before the apply, or it was recorded earlier in the same apply. An act already dead by coverage was reported dead at the revision it died; displacing its retained corpse reports nothing.

The second arm covers one request carrying both a `checkpoint_act` of `design-accepted` and the run-level `acceptance` — both recorded through the same seam, the second displacing the first (`RV-405` `F-3`).

Why code, not prose (`RV-367` `F-1`): a delta reader must never see one `(ActKind, DesignId)` slot die twice with no intervening `act_recorded`. With the gate, the two death paths — coverage (derived by `invalidation_rows`) and displacement (emitted at the record seam) — are disjoint across revisions as well as within one, so the disjointness claim in `invalidation_rows`' doc becomes true rather than being retracted.
