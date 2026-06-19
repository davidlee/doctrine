# Estimate facet authoring CLI verb

## Context

SL-101–103 built the estimate/value facets (model, parse/validate, unit
resolution, catalog/graph projection); SL-102 added the pure display formatters
(`src/estimate/display.rs`). SL-104 hardens and legitimizes. Across all of it the
facet is **unauthorable from the CLI** — the only way to attach an `[estimate]`
table to an entity is hand-editing its `*.toml`. No `doctrine` verb writes facet
values; the only authored-TOML write seams today are edges (`link`/`needs`/`after`,
via the edit-preserving append in `src/dep_seq.rs`).

This slice closes that gap: a CLI verb that authors / edits / clears the
`[estimate]` facet on an entity through the existing parse/validate matrix and an
edit-preserving write. Paired with IMP-112 (wire display onto the `show` path), it
is what makes the facet usable by a human rather than a contract proven only in
tests.

## Scope & Objectives

- A CLI write verb that **sets** an entity's `[estimate]` bounds (`lower`, `upper`)
  on its identity TOML, allocating the table if absent and updating it if present.
- All writes route through the **existing** `EstimateFacet` normalization +
  validation matrix (finite, `lower >= 0`, `upper >= lower`) — the CLI rejects
  exactly what parse rejects; no second validation implementation.
- **Edit-preserving** write — ride the `toml_edit` seam pattern in `src/dep_seq.rs`
  (preserve unrelated tables/formatting); do not rewrite the whole file.
- A **clear/unset** path that removes the `[estimate]` table cleanly (absent facet
  parses clean per SPEC-020).
- Target resolution via the same canonical-ref seam the edge verbs use
  (`SL-NNN`, `ADR-NNN`, …) — kind-agnostic, matching the facet's kind-agnostic seam.

## Non-Goals

- **Display / `show` wiring** — that is IMP-112; formatters already exist.
- **Value facet authoring** — symmetric but separate; see open question O3. Default
  out unless design folds it in cheaply.
- **Confidence authoring** — `lower_confidence`/`upper_confidence` are unspec'd
  until SL-104's confidence legitimization lands; this verb does not author them
  (see O4).
- New validation semantics, aggregation, gating — none. Pure authoring of existing
  model.

## Affected Surface

- `src/estimate.rs` / `src/estimate/` — the writer (new), reusing the existing
  model + validate.
- CLI dispatch (top-level verb registration) + arg parsing.
- `src/dep_seq.rs` — reference pattern for the edit-preserving append (reuse, don't
  duplicate).

## Open Questions (for /design)

- **O1 — verb shape.** `doctrine estimate set <ID> --lower N --upper N` (facet-named
  verb) vs a generic `doctrine facet set <ID> estimate ...` seam anticipating value
  / future facets. Tension: YAGNI vs the symmetric value facet arriving next.
- **O2 — write-seam reuse.** Generalize the `dep_seq.rs` edit-preserving append into
  a shared facet-table writer, or a bespoke estimate writer alongside it? Honour
  "no parallel implementation."
- **O3 — value facet.** Fold `[value]` authoring into the same verb now (symmetric,
  cheap) or defer to a sibling slice?
- **O4 — confidence.** Once SL-104 legitimizes confidence, does this verb grow
  `--lower-confidence`/`--upper-confidence`, or is that a later increment?
- **O5 — partial update.** Does `set` require both bounds every call, or allow
  updating one (re-validating the merged pair)?

## Verification / Closure Intent

- Round-trip: `set` then catalog scan reads back the normalized facet; `clear` then
  scan reads absent.
- CLI rejects the full invalid matrix (missing bound, negative, `upper < lower`,
  non-finite) with the same verdicts as parse.
- Edit-preserving: unrelated tables/relations on the target TOML survive a `set`.
- Dogfood: author an estimate on a live entity via the verb, not by hand.

## Follow-Ups
