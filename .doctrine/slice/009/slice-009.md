# Slice status rollup

## Context

Phase progress is invisible without reading the per-phase state tomls by hand
(`CLAUDE.md` known gap). `doctrine slice list` shows only the hand-edited
`slice-nnn.toml` `status` field — a human-authored lifecycle label that is **not
derived** from phase completion. So a slice can read `proposed` in `slice list`
while five of its six phases sit `completed` in the runtime state tree, and
nothing surfaces the divergence. The `X/Y complete` rollup lives in
`.doctrine/state/slice/nnn/phases/phase-NN.toml` and is never aggregated.

This slice closes that gap with a **read-only derivation**: enumerate a slice's
phase tracking tomls, fold their `status` into an `X/Y complete` rollup, and
surface it. It writes nothing, mutates nothing, adds no state — the lowest-risk
shape doctrine has (the inverse of SL-007/008, which add schema and a git seam).
It is independent of the memory work in flight: it rides the slice/phase read
surface (`src/state.rs`, `src/meta.rs`, `src/slice.rs`), never memory's schema or
`src/git.rs`.

The rollup is **derived, never stored** (the storage rule): computed at query
time from the disposable, gitignored state tree, displayed, never written back
into the authored `slice-nnn.toml`. The hand-edited `status` stays the authored
lifecycle label; the rollup is its runtime counterpart, and surfacing both is
what makes their divergence visible.

## Scope & Objectives

- **Pure phase-status reader (`src/state.rs`).** `state.rs` today owns only phase
  *writers* (`init_phases`, `set_phase_status`) and the stem enumerator
  (`existing_phase_stems`). Add the missing *reader*: enumerate a slice's
  `phase-NN.toml` files, parse each `status`, and return a rollup value — counts
  per status (`completed`, `in_progress`, `blocked`, `planned`) and the total.
  IO (read the dir + files) sits in the thin shell; the fold/count is pure and
  unit-tested off an in-memory status list. Absent/unmaterialised phases (a slice
  with no state tree — e.g. SL-004) yield an explicit *no phases* rollup, never a
  zero-of-zero that reads as "done".

- **Rollup surfaced in `doctrine slice list`.** Add an `X/Y` (or
  `done/total`) column to the slice listing. Critically, `meta::format_list` is
  **shared with `adr list`** and ADRs have no phases — so the rollup column is
  layered in the slice-specific list path (`slice::run_list`), and the neutral
  `meta` formatter stays untouched (no phase concept bleeds into the governance
  entity). The exact composition (extend `Meta`, wrap it, or a slice-local row
  type) is a design.md decision; the constraint — *keep `meta` neutral* — is
  fixed here.

- **Divergence surfaced, not silently reconciled.** When the authored
  `slice-nnn.toml` `status` disagrees with the derived rollup (e.g. `status =
  "done"` but phases incomplete, or all phases `completed` while `status =
  "in_progress"`), the listing makes both legible. v1 *shows* the mismatch; it
  does **not** auto-edit the authored status (that is the deferred lifecycle-
  transition verb, a separate known gap). The rule: derived data never overwrites
  authored data.

- **Behaviour-preservation.** The engine, state writers, and `meta`/`adr` list
  paths are untouched in contract; their existing suites stay green unchanged. The
  new reader and the slice-list column arrive with their own tests.

End state: `doctrine slice list` shows, per slice, both the authored lifecycle
status and the derived `X/Y` phase rollup, with no-phases rendered explicitly and
divergence visible. doctrine gains its first *derived* status surface, read-only,
under the storage rule, with the shared `meta` formatter kept neutral.

## Non-Goals

- **Lifecycle-transition verb.** Moving a slice `proposed→…→done`, or auto-
  syncing the authored `status` to the rollup, is the separate
  `slice status <ID> --status S` gap (`CLAUDE.md`). This slice only *reveals* the
  divergence the transition verb would later resolve; it writes nothing. (ADR's
  authored-status `toml_edit` primitive from SL-006 is the reuse seam when that
  lands.)

- **A standalone `slice status <ID>` detail view.** A per-slice expansion (phase-
  by-phase breakdown, the read analogue of a `show`) is a natural follow-up. v1
  delivers the aggregate column in `list`; the drill-down waits for demand.

- **`--format=tsv` / machine output.** Scriptable output is deferred (it pairs
  naturally with the same need on `adr list` — `CLAUDE.md` F2 there). v1 is the
  human listing.

- **Phase-state schema change.** The `phase-NN.toml` shape is read as-is; no new
  field, no migration. Pure consumer of the existing tracking format.

- **Rollup persistence / caching.** No derived index, no cache file. Recomputed
  per invocation from the state tree — cheap at v1 scale, and avoids a stale-cache
  failure mode.

## Summary

A read-only `X/Y complete` phase rollup for slices. A new pure phase-status
**reader** in `src/state.rs` (which today has only writers) enumerates and folds
each slice's `phase-NN.toml` status; `doctrine slice list` gains a rollup column
layered in the slice-specific path so the shared `meta` formatter (used by
`adr list`, which has no phases) stays neutral. Divergence between the authored
`slice-nnn.toml` status and the derived rollup is surfaced, never silently
reconciled — derived data does not overwrite authored data (the lifecycle-
transition verb that *would* reconcile it is a separate deferred gap). No writes,
no new state, no schema change: the lowest-risk slice, fully independent of the
SL-007/008 memory work.

The reader's return shape, the `slice list` column composition (keeping `meta`
neutral), the no-phases rendering, and the divergence-display format are the
design.md decisions — authored next, pending adversarial review per the
slice-002/003/004 rhythm.

## Follow-Ups

- **`slice status <ID> --status S` (lifecycle transition).** The authored-status
  `toml_edit` verb slice still lacks; reuses SL-006's ADR authored-status
  primitive. Resolves the divergence this slice only reveals.
- **`slice status <ID>` detail view.** Per-phase breakdown for one slice — the
  read analogue of a `show`.
- **`--format=tsv` on `slice list` (and `adr list`).** Machine-readable output;
  shared need across the two list paths.
- **CLAUDE.md.** Drop the "no slice status rollup" known-gap note and document the
  new column when this lands.
