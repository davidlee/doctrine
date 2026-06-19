# Estimate facet authoring CLI verb

## Context

SL-101–103 built the estimate/value facets (model, parse/validate, unit
resolution, catalog/graph projection); SL-102 added the pure display formatters
(`src/estimate/display.rs`). SL-104 hardens and legitimizes. Across all of it the
facet is **unauthorable from the CLI** — the only way to attach an `[estimate]`
table to an entity is hand-editing its `*.toml`. No `doctrine` verb writes facet
values; the only authored-TOML write seams today are edges (`link`/`needs`/`after`,
via the edit-preserving append in `src/dep_seq.rs`).

This slice closes that gap: CLI verbs that author / edit / clear the `[estimate]`
**and `[value]`** facets on an entity through the existing parse/validate matrix and
an edit-preserving write. Paired with IMP-112 (wire display onto the `show` path),
it is what makes the facets usable by a human rather than a contract proven only in
tests. Full design: `design.md`.

## Scope & Objectives

- `estimate` / `value` subcommand groups (`set` / `clear`) — see `design.md` §3 for
  the surface (`estimate set <ID> <lower> <upper>` / `-x <N>` point / `clear`;
  `value set <ID> <magnitude>` / `clear`).
- All writes route through the **existing** pure parse/validate (estimate matrix:
  finite, `lower >= 0`, `upper >= lower`; value: finite) — the CLI rejects exactly
  what parse rejects; no second validation implementation. A thin `value::validate`
  is added for symmetry.
- **Edit-preserving** `toml_edit` write in a new ADR-001 leaf `src/facet_write.rs`
  (one core generic over table-name + scalar fields, serving both facets); preserve
  unrelated tables/formatting; alloc-the-table-if-absent (safe — a whole-table
  append is position-independent).
- A **clear/unset** path removing the facet table cleanly (absent facet parses clean
  per SPEC-020); clear-when-absent is a friendly no-op.
- Target resolution via the same canonical-ref seam the edge verbs use
  (`SL-NNN`, `ADR-NNN`, …) — kind-agnostic, matching the facet's kind-agnostic seam.

## Non-Goals

- **Display / `show` wiring** — that is IMP-112; formatters already exist.
- **Confidence authoring** — `lower_confidence`/`upper_confidence` are unspec'd
  until SL-104's confidence legitimization lands; this verb does not author them.
- **Change history / time-series** — IDE-013. This slice is history-*ready* (the
  edit-preserving writer preserves unknown facet sub-keys; a VT pins it), not
  history-*bearing*.
- New validation semantics, aggregation, gating — none. Pure authoring of existing
  model.

## Affected Surface

- `src/estimate.rs` / `src/estimate/` — the writer (new), reusing the existing
  model + validate.
- CLI dispatch (top-level verb registration) + arg parsing.
- `src/dep_seq.rs` — reference pattern for the edit-preserving append (reuse, don't
  duplicate).

## Resolved Decisions (was Open Questions — see design.md §3/§7)

- **O1 — verb shape** → subcommand groups (`estimate`/`value` × `set`/`clear`),
  positional bounds, `-x` point flag.
- **O2 — write seam** → new ADR-001 leaf `src/facet_write.rs` (cohesion + naming
  honesty); no code duplicated (cores are operation-specific).
- **O3 — value facet** → folded in (symmetric, one generic core).
- **O4 — confidence** → out (blocked on SL-104).
- **O5 — partial update** → `set` is a full-facet replace (both bounds always);
  validation needs both anyway.

## Verification / Closure Intent

- Round-trip: `set` then catalog scan reads back the normalized facet; `clear` then
  scan reads absent.
- CLI rejects the full invalid matrix (missing bound, negative, `upper < lower`,
  non-finite) with the same verdicts as parse.
- Edit-preserving: unrelated tables/relations on the target TOML survive a `set`.
- Dogfood: author an estimate + value on a live entity via the verb, not by hand.

## Follow-Ups

- **IDE-013** — estimate/value change history (time-series). Deferred from this
  slice; SL-118 ships history-ready (forward-compat preserving writer + VT-7).
- **IMP-112** — wire estimate display onto the `show` path (the rendering pair).
