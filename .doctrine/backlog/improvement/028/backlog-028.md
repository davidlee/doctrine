# IMP-028: retire backlog order; fold ordering into list as a flag (--order)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`backlog order` and `backlog list` are divergent surfaces over the same spine.
`list` carries the full grammar — `--kind/--filter/--regexp/--status/--tag/--all/
--format/--json/--columns` (the `listing.rs` shared column model). `order` carries
only `--path`. To see *ordered* output you must use `order`, which then refuses
every filter/format/column the user already knows from `list`.

Discovered while inspecting a soft `after` cycle (RSK-001 ⟷ ISS-003): `list` never
shows relations or sequence, `order` does — but you cannot filter the ordered view
by kind/status or reshape its columns.

## Want

Don't teach `order` the list grammar — **retire `order` and fold ordering into
`list`**. One verb, one surface. `list` gains a flag (`--order` / `--ordered`)
that switches the row sequence from the default (id/created) to the composed
`after`/`needs` order; everything else — `--kind/--filter/--status/--tag/
--format/--columns` — already works and now composes with ordering for free.

Rationale: `order` is `list` with a different sort and an extra diagnostic block.
A whole second verb for "same rows, different order" is grammar duplication. Merge
beats teaching the duplicate to match (the IMP-028-v1 framing, now rejected).

## Open questions

- **Cycle diagnostics.** The `dropped (soft cycle)` block is `order`-specific
  output with no `list` analogue. Under the merge it becomes conditional stderr/
  footer emitted only when `--order` is set (the shared grammar governs rows/
  columns; the diagnostic rides alongside, never a column).
- **`needs` hard-cycle error.** `order` hard-errors on a `needs` dependency cycle
  (EX-3). `list` is infallible today. `list --order` must inherit that failure
  mode *only* when ordering is requested — plain `list` stays total.
- **Default sort.** Does `--order` flip the comparator, or is composed order the
  new default with `--no-order`/`--by id` to opt out? Lean: explicit `--order`,
  keep id-sort the zero-surprise default.

## Notes

- Sits in the shared-grammar family with [[IMP-017]] (memory list adopts the column
  model) and [[IMP-018]] (columns flag conformance matrix) — same push to make
  every spine surface ride `listing.rs` instead of hand-rolling its own.
- Retiring a shipped verb is a surface break; pairs naturally with [[IMP-006]]
  (uniform destructive + lifecycle-transition verbs) if a deprecation cadence
  exists, else a clean cut since backlog is internal tooling.
