# IMP-112: Wire estimate/value display in show path

## Research summary

`SliceDoc` already deserialises `[estimate]` and `[value]` from TOML
(slice.rs:1013,1015) but `format_show` (slice.rs:1126) ignores both fields.
The data is captured but invisible.

Display helpers exist:
- `src/estimate/display.rs` — `format_estimate_normal` with confidence
  percentile framing ("80% confident this takes 3–5 espresso_shots"). Gated
  behind a `dead_code` expect that explicitly says "deferred to IMP-112."
- `src/value.rs` — no display helpers yet. Needs `format_value_normal`.

JSON output already includes estimate/value via serde derive — no change
needed there. Unit resolution already works via `catalog/hydrate.rs`
(`[estimation].unit` / `[value].unit` from `doctrine.toml`).

Backlog and governance show paths don't carry estimate/value — no changes
needed outside slice.

## Scope (SL-132)

- Add estimate/value display rows to `format_show` in `src/slice.rs:1126`
- Ungate `estimate::display` helpers (remove `dead_code` expect)
- Write `value::display` helper (`format_value_normal`)
- Reuse existing `resolve_units` helper

## Design decisions (architectural feedback)

Before SL-132 design:

- **Where in show?** Estimate/value as dedicated rows in `slice show`
  table output (before or after relations?)
- **Human formatting:** "80% confident this takes 3–5 espresso_shots"
  using unit from `doctrine.toml` `[estimation].unit`
- **JSON shape:** Already serialized via `SliceDoc` derive — confirm or
  adjust the JSON representation
- **Scope of surfaces:** Slice-only (other kinds don't carry
  estimate/value). If expanded later, consume shared `EntityFacets`
  projection (see SL-133 scope doc)
- **Risk facet display:** Deferred — risk lacks show-path display entirely.
  Analogous to estimate/value but out of scope for this slice.
