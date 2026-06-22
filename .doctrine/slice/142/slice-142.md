# Wire tag coefficients into priority scoring

## Context

ADR-015 defines the multi-dimensional priority score. The formula includes a
`× tag_multiplier` term in `value_dim` (the delta-from-default form per REV-009),
but the code currently omits it:
tags are stored on entities (SL-136), `tag_coeff()` exists in `config.rs` as
dead code, and the data path (`read_facets` → `EntityFacets` → `base_score`)
has no slot for tags. RFC-002 next-actions item A captures this.

## Scope & Objectives

**Objective:** Wire entity tags through the scoring pipeline so the tag
coefficient term contributes to `value_dim`.

### In scope

1. **Data path** — add `tags: Vec<String>` to:
   - `EntityFacets` (`src/facet.rs`) — shared projection struct
   - `read_facets` return tuple (`src/catalog/scan.rs`)
   - `ScannedEntity` (`src/catalog/scan.rs`)
   - `build_from` call site constructing `EntityFacets` (`src/priority/graph.rs`)

2. **Formula** — in `base_score`, multiply `tag_multiplier =
   max(0.0, 1.0 + Σ(cfg.tag_coeff(tag) − 1.0))` into `value_dim` (ADR-015 §1 per
   REV-009). Remove `#[expect(dead_code)]` on `tag_coeff()`.

3. **Tests** — add `base_score` tests proving tags shift the score.

### Out of scope

- `tag::normalize_tag` is NOT called in the scoring path — tags are already
  normalized at rest by SL-136. `read_facets` reads raw table values and
  passes them through unmodified.
- IMP-109 (consolidate TOML parsing) is adjacent but not part of this slice.
- `doctrine.toml` seed values for `[priority.tag_coefficients]` are item B of
  RFC-002 and will be done separately.
- No changes to SL-136 (tag storage is unified and correct).
- No changes to `kind_weight` or other config surface.
- No `next`/`survey` display changes beyond what golden tests capture.

## Risks & Assumptions

- Tags on entities are already normalized (lowercased, `[a-z0-9_:-]`).
  `read_facets` reads `table.get("tags")` and emits them as raw `Vec<String>`
  without re-normalizing (SL-136 normalizes at rest).
- Tag term formula: `tag_term = 1.0 + Σ(coeff - 1.0)`. An empty or
  all-default-coefficient tag list yields `tag_term = 1.0` (×1.0 identity).
  Each configured tag pushes the multiplier away from 1.0 by its excess
  over the default. Multiple demoting tags (each coeff < 1.0) sum to values
  below 1.0, correctly reducing `value_dim`.
- `read_facets` already parses the TOML table; extracting tags is a
  `table.get("tags").and_then(|v| v.as_array())` call — no extra I/O.
- IMP-109 (double parse) is a known concern but orthogonal to this change.

## Verification / closure

- `base_score` tests with tag-bearing entities prove the coefficient term is
  wired: empty tags ⇒ identity (×1.0); single configured tag ⇒ multiplier
  reflects `1.0 + (coeff - 1.0)`; demoting coeff (< 1.0) reduces `value_dim`.
- `just gate` passes with zero warnings.
- `#[expect(dead_code)]` on `tag_coeff()` removed.
- Golden tests (survey/next/explain) reflect tag-driven ordering shifts in
  any entity that carries tags in the test corpus.

## Follow-Ups

- RFC-002 item B: seed `[priority.tag_coefficients]` in `doctrine.toml`.
- IMP-109: consolidate TOML parsing in the scan path.
