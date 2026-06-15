# SL-070 Plan rationale

## Single-phase rationale

This slice is a single coherent change — all four colour improvements ride
the same type widening (`ColumnPaint::Fixed` → `DynColors`) and touch the same
10 files. Splitting into multiple phases would create artificial sequencing
where PHASE-01's type change has no visible effect until PHASE-02 wires it,
making independent verification impossible. One phase keeps the inner loop
tight: widen the type, swap the palette, add the maps, wire the alternation,
test it all at once.

## Sequencing

Within the single phase, the order is mechanical:

1. **Type widening** (`listing.rs`) — `ColumnPaint::Fixed(AnsiColors)` →
   `DynColors`, `status_hue` → `Option<DynColors>`, `paint_cell` + `row_index`
   + `Alternate` early return. This is the prerequisite for everything else;
   do it first.

2. **Mechanical call-site updates** (all 9 kind files) — every `Fixed(Cyan)` →
   `Fixed(DynColors::Ansi(Cyan))`. Compile-check to confirm no site was missed.

3. **Gruvbox tag palette** (`listing.rs`) — `TAG_PALETTE` → `Rgb` entries,
   `segment_hue` → `Option<DynColors>`, `paint_tag` white resolve.

4. **Alternating title column** — add `TITLE_EVEN`/`TITLE_ODD` constants,
   wire `ColumnPaint::Alternate([…])` on title columns in 7 column definitions
   (slice, backlog, memory, governance, spec-SPEC, review, knowledge, rec).

5. **Per-value hue maps** (`listing.rs`) — `backlog_kind_hue`, `memory_type_hue`,
   `trust_hue`. Wire `ByValue` on backlog kind, memory type, memory trust columns.

6. **Tests** — red first: write the Alternate/paint_tag/hue-map tests, watch
   them fail. Then green: the implementation above makes them pass. Then
   refactor: check for duplication across the three hue maps, ensure
   `paint_cell` is still clean with the new early-return.

## Boundaries

- **Pure layer only** — all colour logic stays in `listing.rs` (a pure leaf).
  The impure `color: bool` is already injected via `RenderOpts`; no new impurity.
- **No per-kind colour config** — hue maps are kind-blind functions in
  `listing.rs`; kind files only wire the `ByValue` reference.
- **No JSON impact** — colours are table-only; `--json` path is untouched.
- **Behaviour-preservation gate** — existing suites stay green unchanged.
  The type widening is a mechanical substitution, not a logic change.
