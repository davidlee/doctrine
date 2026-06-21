# IMP-147: Priority config: per-field isolation for wrong-type [priority] values

Surfaced in SL-133 PHASE-03 (finding F-3a). `priority::config::load` does
`priority.try_into::<PriorityConfig>()` and, on ANY deserialize error, returns the
whole-section `PriorityConfig::default()`. So a single *wrong-type* coefficient
(e.g. `value = "abc"`) silently discards every sibling field's valid value, not
just the offending one.

This is coarser than design §5.2, which specifies per-field "clamped, not fatal"
isolation for malformed values. The numeric clamps (NaN/±∞ → default, negative →
0.0, over-`COEFF_MAX` → max) ARE per-field today — those parse as `f64` and are
clamped individually. Only the wrong-TYPE path is whole-section. Behaviour is safe
(no panic, no hard-error — the advisory-config invariant holds), so this was
tolerated at PHASE-03, not blocked.

**Fix sketch:** deserialise each `[priority]` coefficient through a tolerant
`Option<f64>`-style layer (mirroring the scan's `parse_facet` per-facet isolation),
clamping/defaulting field-by-field, so one typo cannot reset unrelated tuning.

Documented by the existing `non_numeric_value_clamps_returns_defaults` test in
`src/priority/config.rs` — which asserts the current whole-reset behaviour and
should be updated when this lands.
