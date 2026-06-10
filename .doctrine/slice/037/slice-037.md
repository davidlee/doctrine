# Shared list column model: --columns projection + slug-free defaults

## Context

Three backlog items converge on the `list` render surface:

- **IMP-009** — the noisy `slug` column dominates every `list` table, while the
  durable identity is the prefixed id and the slug is *never authoritative*.
  Default tables should drop it; a flag should reveal it.
- **IMP-013** — `slice.rs` and `spec.rs` carry parallel list/show/JSON shapes.
  Marked *deferred-until-condition*: it fires "when a phase next reshapes the
  list or show rendering" in either file. **This slice's slug edit is that
  trigger** — so the shared lift is now cheaper than re-diverging.
- **IMP-014** — `listing.rs`'s cross-verb render surface has no golden harness;
  the column churn this slice causes is exactly what such a harness must pin.

The shared read spine (SL-025, `src/listing.rs`) is a pure leaf (ADR-001; no
clap — A-3) owning the *invariant* axes: filter (`FilterFields`), `Format`,
`render_table`, the JSON envelope. The *variant* axis — column projection — is
today hand-rolled in each kind's `format_rows` (grid) and `json_rows`. The four
list tables and their slug columns:

| verb | default columns | title col? |
|---|---|---|
| backlog | `id kind status slug title` | yes |
| slice | `id status[?][⚠] phases slug title` | yes |
| spec | `id status slug #members` | **no — slug is the label** |
| governance (adr/policy/standard) | `id status slug title` | yes |

## Scope & Objectives

Lift the variant axis into a shared **column model** on the `listing.rs` seam and
deliver the slug-free defaults on top:

1. **Column model** — a per-kind ordered set of columns (name + a pure
   row→string extractor) with a declared *default visible set*. Lives in
   `listing.rs`; each kind declares its own columns (the lift IMP-013 deferred).
2. **`--columns` projection API** — `CommonListArgs` gains `--columns a,b,c`
   selecting/ordering visible table columns by name, validated against the
   kind's available set. The long-term presentation API (supersedes a one-off
   `--slug` boolean).
3. **Slug-free defaults** — each kind's default visible set omits `slug`; `spec`
   swaps `slug → title` so it keeps a human label. `--columns …,slug,…` restores
   it.
4. **Cross-verb golden harness (IMP-014)** — pin every list verb's table + JSON
   output as the regression net for the column churn.

Applies uniformly to all four list verbs (backlog, slice, spec, governance).

## Non-Goals

- **JSON stays faithful/full** — `--columns` is a *table* projection only; JSON
  rows keep every field incl. slug (SL-025 D7, "JSON is data, not
  presentation"). [decision — see design, open to review]
- No change to filter semantics — slug stays searchable via substr/regex
  (`FilterFields` unchanged).
- No change to single-entity `show` — slug still shown there.
- Memory `find`/`list` (keyed, no slug column) is out of scope.
- Not lifting JSON's *typed* per-kind row structs into stringly columns — that
  would regress type fidelity (#members numeric, resolution nullable). The lift
  targets the table column model + the `list_rows` control flow. [decision]

## Affected Surface

- `src/listing.rs` — the column model (`Column`, projection, validation,
  column-aware table render) + `ListArgs` carries the requested columns.
- `src/main.rs` — `CommonListArgs` gains `--columns`; `into_list_args` lowers it.
- per-kind renderers declare columns + default set, drop bespoke `format_rows`:
  `src/backlog.rs`, `src/slice.rs`, `src/spec.rs`, `src/governance.rs`.
- tests — a shared cross-verb golden harness (IMP-014).

## Risks, Assumptions, Open Questions

- RISK: the lift is the exact config-surface IMP-013 warned could exceed the
  duplication it removes (slice decorates status with drift/phase markers; spec
  dispatches subtypes + `#members`). The column model must absorb these as
  per-kind extractors without a baroque config. Watch cohesion.
- RISK: cross-verb table/JSON churn is wide; the IMP-014 golden harness is the
  mitigation (mem `conformance-asserts-surface`, `black-box-cli-golden`).
- ASSUMPTION: slug hidden from default table, not removed — persists in JSON,
  filter, and `show`.
- ASSUMPTION: `--columns` is table-only; JSON faithful-full (Non-Goals). Open to
  adversarial review.
- OQ-1: validation/error shape for an unknown `--columns` name (reuse the
  `validate_statuses` one-line-error pattern, A-2).
- OQ-2: does `--columns` ordering allow duplicates / arbitrary order, or just a
  visibility subset in canonical order? (design)

## Verification / Closure Intent

Each verb's default `list` table omits slug (spec shows title); `--columns`
selects/orders columns and restores slug; an unknown column errors cleanly; JSON
output unchanged across all verbs; filter still matches on slug. Cross-verb
golden harness green. `cargo clippy` clean, `just check` green. Backlog **IMP-009,
IMP-013, IMP-014** all reconciled to terminal at `/close`.

## Summary

Lift the per-kind list column projection into a shared column model on
`listing.rs`, expose it as a `--columns` API, and ship slug-free defaults (spec
swaps slug→title) with a cross-verb golden net. One coherent change that
resolves IMP-009 (UX), IMP-013 (the lift, now triggered), and IMP-014 (goldens).

## Follow-Ups
