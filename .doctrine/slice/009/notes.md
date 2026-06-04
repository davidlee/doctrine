# Notes SL-009: Slice status rollup

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Realization deviations from the authored plan

- **`render_table` replaced the round-1 `measure_meta_columns`/`format_meta_row`
  sketch** (committed in `2bd527e`, design § 5.1/D2). A whole-row meta formatter
  can't host slice's *middle* `phases` column and the `status_suffix` arg would be
  dead in `meta`. `meta::render_table(rows: &[Vec<String>])` is now the single
  layout authority for **every** list surface (slice + adr); markers (`⚠`/`!N`/`?N`)
  are baked into cell strings by the caller, so `meta` stays phase-blind.

- **`is_terminal_status` lives in `slice.rs`, not `state.rs`** (plan PHASE-02 EX-3
  said state.rs). Moved for cohesion: it is slice-*authored*-status vocabulary,
  belongs beside `is_divergent` and the future slice lifecycle-transition verb —
  not in the phase-runtime-state module. It is the single terminal-token source
  (`{"done"}`, provisional); **the deferred lifecycle verb must reuse it**, never
  re-hardcode.

- **PHASE-02 and PHASE-03 landed in one commit** (`0c2a1ab`). The rollup core has
  no production consumer until the list wiring, so the two phases cannot each pass
  `-D dead-code` as separate commits. Phase boundary kept in the plan as a logical
  split; execution collapsed it. (Lesson for future plans: a "pure core" phase
  whose only consumer is the next phase can't end green alone under `-D unused`.)

## Seams the deferred follow-ups ride

- **Slice lifecycle-transition verb** (the open CLI gap): reuse
  `slice::is_terminal_status`; it would *reconcile* the divergence this slice only
  *reveals* (`⚠`). Derived data never overwrites authored — that rule is the reason
  divergence is shown, not auto-fixed.
- **`PhaseTracking` enum** (design Open Q5): untracked vs empty both fold to `None`
  → `—` today. Promote `phase_rollup`'s return to an enum only when a consumer (the
  `slice status <ID>` detail view) needs to tell them apart. `PhaseRollup` already
  carries `planned`/`in_progress` for that view; no reshape needed.
- **`--format=tsv`**: `slice list` is human-only output (header + the `phases`
  column make it structurally distinct from `adr list`). Machine consumers wait for
  this flag; it pairs with the same need on `adr list`.

## Verification at close

`just check` green — 318 tests (was 306). Dogfooded: `./target/debug/doctrine slice
list` renders the rollup on this repo (009 → `1/3` mid-build; `—` for the
phase-trackingless 001–004); `adr list` output unchanged.
