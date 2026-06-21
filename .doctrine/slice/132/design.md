# SL-132 Design — Wire estimate/value display in show path

## Current behavior

`doctrine slice show SL-NNN` renders identity, dates, relationships, and body.
`SliceDoc` deserialises `estimate` and `value` from TOML (slice.rs:1013,1015)
but `format_show` (slice.rs:1126) ignores both fields.

`estimate::display` helpers (`format_estimate_normal`, `format_estimate_verbose`)
are implemented and tested but gated behind `#[expect(dead_code)]` on `mod
display` (estimate.rs:26). Confidence constants and `resolve_confidence` are
similarly gated (estimate.rs:40,48,86). `value.rs` has no display helpers.

## Target behavior

```
SL-132 — Wire estimate/value display in show path
wire-estimate-value-display-in-show-path · proposed
conduct: self/auto
created 2026-06-21 · updated 2026-06-21
estimate: 80% confident this takes 3–5 espresso_shots
value: 5 magic_beans

relationships:
  ...

# body text
```

- **Estimate row**: confidence-framed percentile bounds from config
  (`lower_confidence` / `upper_confidence`), displayed with resolved unit
- **Value row**: magnitude + resolved unit (`"Value: {magnitude} {unit}"`)
- **Absent facets → no row**. An unauthored slice's output is byte-identical
  to the current output. No "none recorded" placeholder.
- **JSON output unchanged**. Facets already serialise via `SliceDoc`'s
  `serde::Serialize` derive.

## Code impact

| File | Change | Detail |
|------|--------|--------|
| `src/slice.rs:1062-1070` | `run_show` — resolve units, pass to `format_show` | Add `let units = crate::catalog::hydrate::resolve_units(&root)?` (or inline equivalent); pass `&units` |
| `src/slice.rs:1126` | `format_show` — accept `units: &Units`, render estimate/value rows | Two `parts.push(...)` after dates line, before relationships |
| `src/estimate.rs:26-28` | Remove `#[expect(dead_code)]` on `mod display` | Display helpers gain a live call site |
| `src/estimate.rs:40-49` | Remove `#[expect(dead_code)]` on confidence constants | Consumed by confidence display |
| `src/estimate.rs:86-90` | Remove `#[expect(dead_code)]` on `resolve_confidence` | Called by `format_show` to pick percentile band |
| `src/estimate/display.rs` | Add `format_estimate_confidence` | New function: bounds + confidence → "80% confident this takes 3–5 unit" |
| `src/value.rs` | Add `format_value_normal` | New function: magnitude + unit → "Value: 5 magic_beans" |
| `src/slice.rs` tests | Add `format_show` cases with facets present + absent | Both present, estimate-only, value-only, neither (golden) |

## Design decisions

### D1 — Units pass-through

`run_show` resolves `Units` from `doctrine.toml` and passes `&Units` to
`format_show`. `format_show` stays pure — receives resolved strings, no disk
access. Follows ADR-001 (pure/impure split).

`run_show` already loads `doctrine.toml` for conduct config; unit resolution
is one additional call to `crate::estimate::resolve_unit` /
`crate::value::resolve_unit` on the same `DoctrineToml`.

### D2 — Confidence display formula

`format_estimate_confidence(facet, lower_pct, upper_pct, unit)`:

```
lower_bound = facet.lower + lower_pct × (facet.upper - facet.lower)
upper_bound = facet.lower + upper_pct × (facet.upper - facet.lower)
```

Output: `"{:.0}% confident this takes {:.1}–{:.1} {unit}"`

Defaults: `lower_pct = 0.1`, `upper_pct = 0.9` → 80% band.
Configurable via `doctrine.toml` `[estimation].lower_confidence` /
`[estimation].upper_confidence`.

Example: `EstimateFacet { lower: 2.0, upper: 8.0 }` with (0.1, 0.9) →
`"80% confident this takes 2.6–7.4 espresso_shots"`

### D3 — Absent facets → no row

If `doc.estimate` is `None`, no estimate row appears. Same for value.
Keeps output strictly additive — an unauthored slice's `show` output is
byte-identical to pre-change.

### D4 — No value confidence framing

Value is a single `f64` magnitude — no range, so no percentile selection
applies. Display is `"Value: {magnitude} {unit}"`.

### D5 — EntityFacets deferred

`format_show` receives individual `Option<&EstimateFacet>` /
`Option<&ValueFacet>` fields, not an `EntityFacets` struct. The shared
projection is created when SL-133 design needs it — at that point,
`src/facet.rs` defines `EntityFacets { estimate, value, risk, tags }`
and the call site is refactored to construct and pass it.

Memory stub `mem.trigger.entity-facets-struct` records the trigger
condition and handoff.

### D6 — Risk display deferred

Risk facet not touched by this slice. `EntityFacets` will carry
`risk: Option<RiskFacet>` when created (SL-133 or later).

## Verification

| ID | What | How |
|----|------|-----|
| VT-1 | Estimate present → confidence row rendered | Unit test: `format_show` with `estimate = Some(...)` produces confidence-framed row |
| VT-2 | Estimate absent → no row | Unit test: no estimate line in output |
| VT-3 | Value present → row rendered | Unit test: value row with correct unit |
| VT-4 | Value absent → no row | Unit test: no value line |
| VT-5 | Both absent → output matches pre-change | Golden test: byte-identical to current `format_show` output |
| VT-6 | No `dead_code` warnings post-change | `cargo build` passes clean |
| VT-7 | JSON unchanged | Existing `show_json` tests pass unchanged |
| VT-8 | Gate zero warnings | `just check` passes; `just gate` passes |

## Non-goals

- No changes to backlog/governance show paths
- No risk facet display
- No EntityFacets struct (deferred to SL-133 trigger)
- No `survey`/`next` output changes (SL-133)
- No history tracking (IDE-013)
