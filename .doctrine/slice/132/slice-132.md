# Wire estimate/value display in show path

## Context

`SliceDoc` already deserialises `[estimate]` and `[value]` from TOML
(slice.rs:1013,1015) but `format_show` (slice.rs:1126) ignores both fields.
The data is captured but invisible — exactly the consumption-surface gap
RFC-001 names.

Display helpers exist in `src/estimate/display.rs` (`format_estimate_normal`,
`format_estimate_verbose`) but are gated behind a `dead_code` expect that
explicitly says "deferred to IMP-112." `src/value.rs` has no display helpers
at all.

Unit resolution already works: `catalog/hydrate.rs` reads
`[estimation].unit` / `[value].unit` from `doctrine.toml` (defaulting to
`"espresso_shots"` / `"magic_beans"`).

## Scope & Objectives

- Add estimate display (confidence-percentile framing: "80% confident this
  takes 3–5 espresso_shots") to `doctrine slice show` table output
- Add value display to `doctrine slice show` table output
- Ungate the existing `estimate::display` helpers (remove `dead_code` expect)
- Write `value::display` helper (format_value_normal)
- Unit resolution: reuse the existing `resolve_units` helper
- JSON output already includes estimate/value via serde derive — no change

## Design decisions needed (architectural feedback)

Before design proceeds:

- **Where in show output?** Estimate/value should appear as dedicated rows
  in the `slice show` table output.
- **Human formatting:** Confidence-percentile language — "80% confident
  this takes 3–5 espresso_shots" with unit from `doctrine.toml`.
- **JSON shape:** Estimate/value already serialize via `SliceDoc`'s derive.
  Confirm the JSON shape is acceptable or adjust.
- **Backlog-only or all surfaces?** Slice is the only kind carrying
  estimate/value currently. If other kinds gain facets later, the display
  should use the shared `EntityFacets` projection (see SL-133 scope doc).

## Shared facet projection — design phase deliverable

SL-132 and SL-133 both need estimate/value data. A shared `EntityFacets`
projection (estimate, value, risk, tags) must be established before
either slice grows its own facet parser.

**Design phase deliverable:** Define `EntityFacets` struct and its
scan-time hydration. SL-132 design owns this — it is the first consumer.
SL-133 design references the projection shape as a gate.

This slice's `format_show` consumes `EntityFacets`, not `SliceDoc`'s
fields directly. The projection is not a separate slice — it is small
enough to emerge from SL-132 design.

## Non-Goals

- No changes to backlog/governance show paths (those kinds don't carry
  estimate/value facets)
- No changes to `survey`/`next` output (that's SL-133)
- No history tracking (that's IDE-013, deferred)

## Terrain

| File | Change |
|------|--------|
| `src/slice.rs:1126` | `format_show` — add estimate/value rows |
| `src/estimate.rs:50-57` | Remove `dead_code` expect, ungate helpers |
| `src/estimate/display.rs` | Existing — no changes needed (already written) |
| `src/value.rs` | Add `format_value_normal` display helper |
| `src/catalog/hydrate.rs:185-199` | `resolve_units` — reuse, no changes needed |

## Dependencies

- None. `SliceDoc` already carries the fields. `estimate::display` already
  has the helpers. Unit resolution already works.
- The `dead_code` expect on estimate.rs:50-57 is the only gate to remove.
