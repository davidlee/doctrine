A single `design apply` request may carry both a `checkpoint_act` of kind
`design-accepted` **and** the run-level `acceptance`. `apply` records the
explicit act, then records the acceptance through the same `record_act` seam —
so the second recording **displaces** an act recorded moments earlier in the
same apply. The shipped lock recipe sends exactly this pair.

Consequence for any per-apply reasoning about acts: "live at `prior`" is not
the whole set of acts that can die in this apply. `SL-272` (`ISS-454`,
`DEC-336`) gates the displacement `act_invalidated` row on a `reportable` set
seeded from `live_acts(prior)` and extended by every recording in the apply; a
gate on `prior` alone would drop the first recording's death (`RV-405` `F-3`).
Tests: `a_double_acceptance_reports_the_first_recordings_death` in
`src/design_run/run.rs`.
