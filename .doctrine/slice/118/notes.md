# Implementation Notes SL-118

## Audit outcome (RV-103, 2026-06-20)

Candidate surface `cand-118-review-001` (merge of `review/118` onto `main` at
`d1fa743b`). Four findings, all aligned (no fix-now, no blocker). Clippy
zero-warning, fmt clean, 1994 tests green.

## Phase harvest

### PHASE-01 — Validation completeness

- `estimate::validate` gained finiteness checks (inf/nan on both bounds).
- `value::validate` added (finite check) and wired into `value::normalise`.
- 7 new tests; 0 pre-existing estimate/value test edits — behaviour-preservation
  gate held clean.
- **Gotcha:** `inf`/`nan` slip past clap's `f64` parser — finiteness must live
  in `validate`, not at the argument parser.

### PHASE-02 — facet_write leaf

- New `src/facet_write.rs` (512 lines, 17 tests). ADR-001 leaf: imports only
  `toml_edit`, `anyhow`, `std`.
- Pure cores: `set_facet` (mutate-in-place, alloc-if-absent, malformed-present
  fail-loud, no-op guard), `clear_facet` (remove-or-no-op).
- IO envelope: `edit_in_place` read→parse→core→write-once-if-changed.
- No clock, no `updated` bump per D1 reversal.
- **Pattern:** The no-op guard accepts integer-form values (`lower = 2`) as
  equal to float repr (`2.0`) — avoids normalising hand-authored integer
  formatting on every no-op pass.

### PHASE-03 — CLI verbs

- `Command::Estimate`/`Command::Value` subcommand groups (each `Set`/`Clear`).
- `exact: Option<f64>` with `conflicts_with_all = ["lower", "upper"]`; handler
  enforces exactly-one-mode.
- Path-from-ref helper (`resolve_entity_path_and_canonical`) factored from
  `resolve_link_path` pattern.
- `allow_hyphen_values = true` on value magnitude (negative values).
- Write class registered: `Write("estimate")`, `Write("value")`.

## Standing risks

- Candidate worktree has a pre-existing e2e failure
  (`relation_rows_of_one_label_are_contiguous`) — ISS-030 corpus-data dirt,
  not SL-118-caused. Reproduces without this slice's code.

## Deferred work

- **IMP-112** — wire estimate/value display onto the `show` path (formatters
  from SL-102 already exist).
- **IDE-013** — estimate/value change history (time-series). SL-118 ships
  history-ready: the edit-preserving writer preserves unknown sub-keys (VT-7).
- **Confidence authoring** — blocked on SL-104 legitimization of confidence
  bounds.
