# SL-132 Design — Wire estimate/value display in show path

## Current behavior

`doctrine slice show SL-NNN` renders identity, dates, relationships, and body.
`SliceDoc` deserialises `estimate` and `value` from TOML (slice.rs:1013,1015)
but `format_show` (slice.rs:1126) ignores both fields.

`estimate::display` helpers (`format_estimate_normal`, `format_estimate_verbose`)
are implemented and tested but gated behind `#[expect(dead_code)]` on `mod
display` (estimate.rs:26). Confidence constants and `resolve_confidence` are
similarly gated (estimate.rs:40,48,86). `value.rs` has no display helpers.

Estimate/value data already flows through two independent parse paths:
- **Catalog path**: `scan.rs::read_facets` → `ScannedEntity.estimate/.value` →
  `CatalogEntity.estimate/.value` (for survey/next/graph)
- **Show path**: `SliceDoc` serde deserialise (for per-entity display)

These predate SL-132 and are not changed by it.

## Target behavior

```
SL-132 — Wire estimate/value display in show path
wire-estimate-value-display-in-show-path · proposed
conduct: self/auto
created 2026-06-21 · updated 2026-06-21
estimate: 3–5 espresso_shots (80% confidence)
value: 5 magic_beans

relationships:
  ...

# body text
```

- **Estimate row**: confidence-framed bounds with parenthetical confidence level
  (`"3–5 espresso_shots (80% confidence)"`), using config-driven percentile
  bounds and resolved unit
- **Value row**: magnitude + resolved unit (`"Value: {magnitude} {unit}"`)
- **Absent facets → no row**. An unauthored slice's output is byte-identical
  to the current output. No "none recorded" placeholder.
- **JSON output unchanged**. Facets already serialise via `SliceDoc`'s
  `serde::Serialize` derive.

## Code impact

| File | Change | Detail |
|------|--------|--------|
| `src/facet.rs` (new) | Define `EntityFacets` struct | `{ estimate: Option<EstimateFacet>, value: Option<ValueFacet> }` — pure data struct, no parsing. Wraps already-parsed fields from `SliceDoc` (show path) or `ScannedEntity` (catalog path). Extended with risk/tags when SL-133/136 need them. ADR-001: engine tier — no disk/clock/rng. |
| `src/slice.rs:1062-1070` | `run_show` — resolve units inline, construct `EntityFacets`, pass to `format_show` | Resolve `estimation_unit`/`value_unit` from the already-loaded `DoctrineToml cfg` via `crate::estimate::resolve_unit` / `crate::value::resolve_unit`. Construct `EntityFacets { estimate: doc.estimate, value: doc.value }`. No catalog coupling — uses the same `cfg` the conduct load already read. |
| `src/slice.rs:1126` | `format_show` — accept `facets: &EntityFacets`, `estimation_unit: &str`, `value_unit: &str`, render rows | Two `parts.push(...)` after dates line, before relationships |
| `src/estimate.rs:26-28` | Remove `#[expect(dead_code)]` on `mod display` | `format_estimate_confidence` lives in this module and now has a call site |
| `src/estimate.rs:40-49` | Remove `#[expect(dead_code)]` on confidence constants | Consumed by confidence display |
| `src/estimate.rs:86-90` | Remove `#[expect(dead_code)]` on `resolve_confidence` | Called by `format_show` to pick percentile band |
| `src/estimate/display.rs` | Add `format_estimate_confidence` | Signature: `fn format_estimate_confidence(facet: &EstimateFacet, lower_pct: f64, upper_pct: f64, unit: &str) -> String` → `"2.6–7.4 espresso_shots (80% confidence)"` |
| `src/value.rs` | Add `format_value_normal` | Signature: `fn format_value_normal(facet: &ValueFacet, unit: &str) -> String` → `"Value: 5 magic_beans"` |
| `src/slice.rs` tests | Add unit + integration tests | Unit: `format_show` with facets present/absent/custom-bounds/zero-width. Integration: `run_show` against fixture (slice TOML with estimate + doctrine.toml with units). Malformed doctrine.toml → error propagated. |

## Design decisions

### D1 — Units resolved inline, no catalog coupling

`run_show` already loads `DoctrineToml` for conduct config (line 1068:
`load_conduct(&root)`). Unit resolution reuses the same `cfg`:

```rust
let estimation_unit = crate::estimate::resolve_unit(&cfg.estimation);
let value_unit = crate::value::resolve_unit(&cfg.value);
```

Passed to `format_show` as `&str`. No import of `catalog::hydrate`, no
`Units` struct needed in the signature. Follows ADR-001: shell resolves,
engine receives pure values.

### D2 — Confidence display formula

`format_estimate_confidence(facet, lower_pct, upper_pct, unit)`:

```
lower_bound = facet.lower + lower_pct × (facet.upper - facet.lower)
upper_bound = facet.lower + upper_pct × (facet.upper - facet.lower)
```

Output: `"{:.1}–{:.1} {unit} ({:.0}% confidence)"`

Defaults: `lower_pct = 0.1`, `upper_pct = 0.9` → 80% band.
Configurable via `doctrine.toml` `[estimation].lower_confidence` /
`[estimation].upper_confidence`.

Example: `EstimateFacet { lower: 2.0, upper: 8.0 }` with (0.1, 0.9) →
`"2.6–7.4 espresso_shots (80% confidence)"`

### D3 — Absent facets → no row

If `facets.estimate` is `None`, no estimate row appears. Same for value.
Keeps output strictly additive — an unauthored slice's `show` output is
byte-identical to pre-change.

### D4 — No value confidence framing

Value is a single `f64` magnitude — no range, so no percentile selection
applies. Display is `"Value: {magnitude} {unit}"`.

### D5 — EntityFacets wraps existing data, no new parse

`EntityFacets` is a pure aggregation struct. It does NOT introduce a new
TOML parse path — it wraps already-parsed fields:

- **Show path**: constructed from `SliceDoc.estimate`/`SliceDoc.value`
  (serde-deserialised, already validated)
- **Catalog path** (future SL-133): constructed from
  `ScannedEntity.estimate`/`ScannedEntity.value` (scanned via
  `read_facets`, already validated)

The two existing parse paths (`SliceDoc` serde + `scan::read_facets`)
predate SL-132. `EntityFacets` gives them a shared consumption contract
so SL-132 and SL-133 don't grow independent readers.

### D6 — EntityFacets extended by later slices

`src/facet.rs` is created in this slice with `{ estimate, value }`.
When SL-133 needs risk and SL-136 needs tags, they add fields to the
struct and the call sites populate them. `format_show` ignores fields
it doesn't render — no refactoring needed.

### D7 — Existing `format_estimate_normal` / `format_estimate_verbose` kept gated

These helpers remain behind `#[expect(dead_code)]` (still on `mod
display` — the ungating applies only to the new `format_estimate_confidence`
call site within the module). They are preserved for future verbose
display modes (e.g. `slice show --detail`). They do not diverge — they
render different aspects (simple bounds vs confidence-framed).

### D8 — Risk display deferred

Risk facet not touched by this slice. `EntityFacets` will carry
`risk: Option<RiskFacet>` when SL-133 adds it.

### D9 — ADR-001 tier: `src/facet.rs` is engine tier

`EntityFacets` is a pure data struct — no disk I/O, no clock, no rng,
no git. It sits in the engine tier alongside `src/estimate.rs` and
`src/value.rs`. No `layering.toml` change needed (bins-only crate;
layering enforcement is architectural convention, not tool-enforced).

## Verification

| ID | What | How |
|----|------|-----|
| VT-1 | Estimate present → confidence row rendered | Unit: `format_show` with `facets.estimate = Some(...)` produces `"estimate: 3–5 espresso_shots (80% confidence)"` |
| VT-2 | Estimate absent → no row | Unit: no `estimate:` line in output |
| VT-3 | Value present → row rendered | Unit: `"value: 5 magic_beans"` appears |
| VT-4 | Value absent → no row | Unit: no `value:` line |
| VT-5 | Both absent → byte-identical to pre-change | Golden: capture current `format_show` output for a fixture slice with no estimate/value; assert output unchanged |
| VT-6 | No `dead_code` warnings | `cargo build` passes clean |
| VT-7 | JSON unchanged | Existing `show_json` tests pass unchanged |
| VT-8 | Gate zero warnings | `just check` passes; `just gate` passes |
| VT-9 | Custom confidence bounds | Unit: `lower_confidence=0.25, upper_confidence=0.75` → `"X–Y unit (50% confidence)"` |
| VT-10 | Zero-width estimate (lower==upper) | Unit: `{lower:5, upper:5}` + (0.1,0.9) → `"5.0–5.0 espresso_shots (80% confidence)"` |
| VT-11 | Shell integration: fixture slice with estimate | Integration: `run_show` against slice TOML fixture with `[estimate]` + `doctrine.toml` with `[estimation]` → output contains confidence row |
| VT-12 | Malformed doctrine.toml propagates error | Integration: `run_show` with unparseable `doctrine.toml` → `Err` returned, not silently defaulted |

## Non-goals

- No changes to backlog/governance show paths
- No risk facet display
- No `survey`/`next` output changes (SL-133)
- No history tracking (IDE-013)
- No verbose estimate display mode (existing helpers preserved for future use)
