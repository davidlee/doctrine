# `dead_code` staging does not cascade to callees

When staging not-yet-wired code behind
`#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, do **not** predict
that a downstream task retires the *inner* items first.

rustc's dead-code pass does not treat a `dead_code`-suppressed item as a **live
root** for the things it reaches. So a staged function that reads a staged
struct's fields, or constructs a staged enum variant, leaves *both* staged.
The suppression is not "this item is alive"; it is "do not tell me this item is
dead" — reachability is unaffected.

**Consequence.** A staged chain (`type → pure fn → shell fn`) goes live **all at
once**, on the task that adds the first *genuinely reachable* production caller.
Intermediate tasks retire nothing.

Observed on SL-249 PHASE-04: `apply_facet_edits` (T3) reads `FacetEdit`'s fields
and constructs `KeyPosture::RequirePresent`, yet all three `expect`s stayed
fulfilled, because `apply_facet_edits` was itself staged. Both the phase sheet
and the author had predicted a T3 retirement.

**Why this costs nothing when you use `expect`.** `expect` errors when the
suppression becomes *unnecessary*, so the build adjudicates the retirement point
and a wrong prediction in a `reason` string is self-correcting on the next
`cargo clippy`. With `allow` the same mistake is silent and the suppression
outlives its purpose. Prefer `expect` for any deliberately temporary suppression,
and write the `reason` as a claim you expect the compiler to check.
