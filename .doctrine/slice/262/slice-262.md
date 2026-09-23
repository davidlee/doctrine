# Envelope names the next move

## Context

`IMP-390` (the turn envelope reports state, not what to do next) had four
candidate remedies. `SL-244` delivered *refusals name the remedy* plus the
gate-condition contracts; `SL-251` delivered *the payload contract is
fetchable*. Two remain, and both are this slice:

1. **`next_obligation` has no writer.** The field sits on the snapshot
   (`src/design_run/snapshot.rs:71`), is constructed `None`
   (`snapshot.rs:586`), and is projected and rendered by
   `src/design_run/render/envelope.rs` — permanently reading *none recorded*.
   In `SL-243` a run sat in `exploring` from revision 6 with its runbook
   cleared and nothing proposing the stage advance; `Locked` is reachable only
   through that move.
2. **Unmet conditions for the next stage are not rendered.** The condition
   contracts exist (`gate::cumulative_conditions`, the `SL-244` table,
   `design-prompts/conditions/`), but the envelope delivers them as a receipt at
   stage *entry*, never as a forward look. An agent learns what the next advance
   requires by being refused.

## Scope & Objectives

Make the envelope carry the turn, not only the state:

1. **Derive the next move** from run state — runbook step outstanding at the
   cursor, else the forward stage advance, else nothing (locked). Name the act
   that discharges it.
2. **Render the forward edge's unmet conditions** — for the run's single
   outbound forward edge (`gate::Advance::from_stage`), which conditions of
   `cumulative_conditions(to)` are not yet satisfied, each with its discharging
   act from the `SL-244` contract table.
3. **Settle `next_obligation`'s fate.** It is stored-but-derivable, which the
   storage rule argues against (derived data is computed, not persisted). The
   likely shape is: derive at render time and delete the stored field — design
   decides. A closed vocabulary where the envelope today renders elided prose is
   an envelope wire change under `DEC-064` (one structured turn envelope),
   recorded as such (`notes.md:2164` of `SL-233`).
4. Guidance (hymn, stage fragments) points at the new envelope rows rather than
   at source.

## Non-Goals

- Deriving the traversal cursor (`IMP-389`) — blocked on `QUE-218`, and a
  different question (which node, not which act).
- `IMP-367`'s other reader-less accessors, beyond deciding who owns
  `next_obligation`'s disposition (recorded against both items at close).
- Adoption / `adopt_authored` — `SL-261`.
- Per-field semantics in the payload contract.

## Affected surface

- `src/design_run/snapshot.rs` — `next_obligation`
- `src/design_run/render/envelope.rs` — projection and render
- `src/design_run/gate.rs` — condition evaluation for the forward edge
- `src/commands/design.rs` — envelope assembly for `show --format prompt` (`DEC-261`) and `resume`
- `install/hymns/stage/design.md`, `install/design-prompts/**`
- `tests/e2e_design_*.rs`

## Risks, assumptions, open questions

- **Assumption:** every gate condition is evaluable without a write, so a
  forward look is a pure function of the snapshot plus observed facts. Check
  against the conditions that bind observed-fact fingerprints.
- **Open:** does a next-move row belong in every envelope projection (compact /
  `--full` / json), and what is its elision class under `DEC-064`'s byte budget?
- **Risk:** snapshot schema change for a persisted field — in-flight runs carry
  `next_obligation: null`; removal must deserialize old snapshots.

## Verification / closure intent

- e2e: a run whose runbook is discharged names the stage advance as the next
  move; an unmet forward condition renders with its discharging act; a locked
  run names nothing.
- No stored field without a writer remains for `next_obligation`.
- Closes `IMP-390` (remaining faces); `IMP-367` updated for the disposition.

## Summary

## Follow-Ups
