# Implementation Plan SL-009: Slice status rollup

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

A read-only `X/Y complete` phase rollup column for `doctrine slice list`,
derived from the gitignored phase-tracking tree, surfaced beside the authored
status with a coarse divergence hint. No writes, no schema change, no cache —
the lowest-risk slice on the board. The design ([design.md](design.md)) is
locked after a codex adversarial round (5 MAJOR folded; § 10).

Three phases, one per seam the design touches:

1. **`meta.rs`** — extract the shared row-formatting helpers (behaviour-preserving).
2. **`state.rs`** — the rollup derivation (pure fold + buckets + terminal-set +
   IO reader).
3. **`slice.rs`** — presentation, wiring, and close-out docs.

## Sequencing & Rationale

**Why this order.** PHASE-01 and PHASE-02 are independent (a `meta` refactor vs a
new `state` reader); PHASE-03 depends on both. PHASE-01 leads because it touches
**shared machinery** — the `meta` module `adr list` also calls — so it runs first
under the behaviour-preservation gate while the tree is otherwise untouched,
keeping the byte-identical proof clean. PHASE-02 then adds an entirely new,
isolated derivation surface that nothing yet consumes (safe to land green on its
own tests). PHASE-03 is the only phase that changes observable `slice list`
output, and it lands last, composing the two finished seams.

**PHASE-01 — keep `meta` neutral, don't fork it.** The rollup column is
slice-only, but the layout (measure, align, gap, newline) is shared. The
re-review (R-F1) rejected copying that layout into `slice.rs` — it would fork the
two list surfaces the moment spacing changes. So `format_list` is refactored over
a neutral `render_table(rows: &[Vec<String>])` and reimplemented byte-unchanged;
`slice.rs` later calls the same renderer with its own cells, including the middle
`phases` column and the header row. (This refines the round-1
`measure_meta_columns`/`format_meta_row` sketch — a whole-row meta formatter
can't host a *middle* column, and a `status_suffix` arg would be dead in `meta`;
a cell-grid renderer composes for any column set.) Markers (`⚠`/`!N`/`?N`) are
baked into cell strings by the caller, so `meta` never learns what a phase is.

**PHASE-02 — derive truthfully, hide nothing.** The re-review tightened three
things this phase encodes: the phase set comes from the module's own
`existing_phase_stems` so the rollup agrees with `init_phases` and a `.md`-only
crash-partial never silently vanishes from the total (R-F4/D8); unknown statuses
and missing-toml stems land in explicit buckets surfaced as `?N`, never folded
into "incomplete" (R-F3/D5); and `is_terminal_status` is the single named home of
the terminal token so the deferred lifecycle-transition verb reuses it instead of
re-hardcoding `"done"` (R-F2/D3). `PhaseRollup` carries every bucket now (D7) so
the detail-view and `--format=tsv` follow-ups need no reshape.

**PHASE-03 — the only behaviour change, fully composed.** Divergence is computed
but conservative: keyed on the terminal-set, and suppressed whenever anomalies are
present (a corrupt slice is not a lifecycle mismatch). The header and the new
column make `slice list` human-only output — machine consumers wait for the
deferred `--format=tsv`, and the phase restates the `adr list` byte-identical
gate so the shared `meta` refactor stays honest end-to-end. Close-out drops the
"no slice status rollup" gap note from CLAUDE.md.

## Notes

- **No new CLI surface.** The existing `slice list` arm is enriched; no `main.rs`
  subcommand is added. `--status` continues to filter the *authored* status.
- **Deferred (design § 6, § Follow-Ups):** the `PhaseTracking` enum distinguishing
  untracked-vs-empty (built when the detail view needs it), the `in_progress`
  column, `--format=tsv`, the `slice status <ID>` detail view, and the lifecycle-
  transition verb that would *reconcile* the divergence this slice only reveals.
- **Behaviour-preservation gate** spans PHASE-01 and PHASE-03: `adr list` (and
  `slice list` until PHASE-03) output stays byte-identical; the existing `meta` /
  `entity` / state-writer suites stay green unchanged.
