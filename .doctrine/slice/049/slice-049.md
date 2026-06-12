# CLI list-surface & input-validation hygiene

## Context

Two open backlog cards report independent papercuts on the CLI surface — one on
the list/render seam, one on input validation — each drifting from the shared
conventions established by `listing.rs` and the entity scaffolders. Each is a
self-contained fix; bundling keeps the per-fix ceremony proportionate while one
design pass reconciles them against the shared seam.

Work intake (the scope is exactly these, no more):

- **IMP-017** — `memory list` hand-rolls its column grid (`memory.rs::format_rows`)
  instead of adopting the shared *column model* (`Column<R>` / `render_columns` /
  `select_columns`) in `listing.rs`. It already rides the shared `retain` /
  `build` / `validate_statuses` / `json_envelope` / `render_table`, so this is the
  last unmigrated seam: the column projection. Adopting it also yields `--columns`
  parity with the other kinds.
- **ISS-004** — `spec req add` aborts with OS error 36 (ENAMETOOLONG) on a long
  title: it derives the on-disk slug from the title via
  `input::resolve_slug → entity::derive_slug`, which is length-unbounded, and the
  `NNN-slug` symlink filename overflows the 255-byte filesystem limit. Two gaps:
  (a) `run_req_add` hardcodes `resolve_slug(.., None)` — no `--slug` escape, unlike
  `spec new`; (b) the shared derivation has no length cap, so the abort is latent
  in every kind, not just `req add`.

ISS-005 (`rec list` empty-corpus header) was triaged out and closed `wont-do`:
its premise is false. Every spine kind suppresses the header on an empty result
by design (SL-025 §5.5 — `render_columns` returns `""` on empty rows); `rec`
already rides the spine and is consistent with `adr`/`slice`. Proof:
`adr list --status superseded` → zero rows → no header, identical to `rec list`.
The report compared an empty `rec` corpus against a populated `adr` corpus.

## Scope & Objectives

1. **IMP-017** — `memory list` renders through the shared column model
   (`Column<Memory>` + `render_columns` + `select_columns`); gains `--columns`.
   No behavioural regression to populated output: the `uid type status trust key
   title` projection, full-uid lead (F-A11), `scrub_line` on free-text cells
   (F-A10), and keyless→`-` all preserved. Goldens stay green or are updated with
   justification.
2. **ISS-004** — `spec req add` no longer aborts on a long title:
   (a) a `--slug` escape is plumbed through `run_req_add` (parity with `spec new`);
   (b) `resolve_slug`/`derive_slug` bounds the derived slug to a safe length in
   the shared path, so the abort is removed for *every* kind. Truncation is
   collision-safe — the numeric `NNN` is identity, the slug in the `NNN-slug`
   symlink is cosmetic — so a bounded slug cannot collide an existing id.

Cross-cutting: the IMP-017 migration honours the shared-model conventions
(pre-materialised row; non-capturing `fn(&R) -> String` cell extractors;
`listing::validate_statuses` already called by `memory list`).

## Non-Goals

- No redesign of `listing.rs`; IMP-017 *adopts* the existing column model.
- No reversal of the SL-025 §5.5 empty-list contract (header-suppressed-on-empty
  stays — that determination closed ISS-005).
- No change to the memory/requirement data models or storage tiers.
- No new list filters or output formats beyond `--columns` falling out of the
  shared model for `memory list`.
- No corpus-wide slug-scheme overhaul — ISS-004 adds a length bound + escape, not
  a new slug grammar.

## Affected Surface

- `src/memory.rs` — IMP-017: replace `format_rows` grid with a `MEMORY_COLUMNS`
  table + `render_columns`/`select_columns`; wire a `--columns` arg on the
  `memory list` command path.
- `src/spec.rs` — ISS-004 (a): add a `--slug` arg to `run_req_add`, pass it to
  `resolve_slug`.
- `src/input.rs` and/or `src/entity.rs` — ISS-004 (b): length bound in the shared
  `resolve_slug` / `derive_slug` path.
- `src/listing.rs` — referenced (column-model adoption target), not redesigned.
- Black-box CLI goldens + unit tests covering the above.

## Risks / Assumptions / Open Questions

- **OQ-1 (open, design)** — Slug length bound: what cap, and truncate on a `-`
  boundary or hard cut? Filesystem limit is 255 bytes for the *symlink filename*
  `NNN-slug` (the `requirement-` / `NNN-` prefix eats a few bytes). Pick a cap
  comfortably under that, on a char boundary; truncate at a `-` if one is near.
- **A-1** — IMP-017 and ISS-004 are file-disjoint (memory.rs vs spec.rs +
  input/entity); phaseable independently. `listing.rs` is read-only for both.
- **A-2** — Capping the shared `derive_slug` changes the derived slug only for
  titles long enough to overflow today (which currently *abort*), so no
  previously-successful slug changes — behaviour-preserving for all existing
  callers.

## Verification / Closure Intent

- IMP-017: a black-box test pins `memory list` shared-model output (header +
  rows unchanged) and `memory list --columns <subset>`; existing memory-list
  goldens stay green or are re-pinned with justification.
- ISS-004: tests pin (a) `spec req add` with `--slug` succeeds and writes the
  given slug; (b) a title long enough to previously abort now succeeds with a
  bounded slug (no ENAMETOOLONG); (c) the bound is collision-safe (two long
  titles sharing a prefix get distinct `NNN`-keyed dirs).
- `just check` green; `cargo clippy` zero warnings.
- IMP-017 and ISS-004 resolved at close with a resolution referencing SL-049.
  ISS-005 already closed (`wont-do`).

## Follow-Ups

- None anticipated. Any `listing.rs` contract change that exceeds adoption →
  separate card, not this slice.
