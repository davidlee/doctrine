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

1. **Derive the forward edge** (`DEC-290`) — a derived `forward` field on
   `TurnEnvelope`: the run's single outbound edge (`gate::Advance::from_stage`),
   its outstanding runbook steps in order, then its unmet conditions of
   `cumulative_conditions(to)` with causes and `Contract::remedy()`. The first
   row is the next act; with nothing outstanding it reads `ready` and carries a
   serialised `StageDeclaration`. `None` only at `locked`.
2. **Delete `next_obligation`** from `RunHeader` and `TurnEnvelope`; envelope
   version → 2 under the compatibility rule of `DEC-291`, which also narrows
   `DEC-124`'s envelope clause to "no contract prose".
3. **One observed-facts builder** (`DEC-292`) — `Observed` (authored
   fingerprint, observed facts, observed review, runbook) built once in the shell
   for apply and every envelope read; `satisfied` / `forward_unmet` take it;
   `advance` delegates to `forward_unmet`.
4. **Placement** (`DEC-293`) — `forward` is no-drop with a derived named limit;
   JSON, `show --format prompt`, status (one line) and `resume` carry it; it
   replaces `resume`'s out-of-envelope `runbook_section`.
5. **Disclosure** (`DEC-294`) — the forward line names the runbook checks a
   read did not run (`STD-003`).
6. Guidance — one line in `install/hymns/stage/design.md`, and the design
   skill's activation step, point at `forward`.

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
- `src/design_run/run.rs` — `DerivedInput` / `Observed`; `src/design_run/bounds.rs` — forward limit
- `install/hymns/stage/design.md`, `install/design-prompts/**`
- `tests/e2e_design_*.rs`

## Risks, assumptions, open questions

- **Checked:** every gate condition is evaluable without a write — `satisfied`
  reads only the snapshot plus `authored_fingerprint`, `observed_facts`,
  `observed_review` (`gate.rs:1382`, `:1442`, `:1609`); runbook standing reads
  asset digests. The forward look is pure over `Observed` (`DEC-292`).
- Settled: placement and elision class — `DEC-293`.
- **Risk:** snapshot schema change for a persisted field — in-flight runs carry
  `next_obligation: null`; removal must deserialize old snapshots.

## Verification / closure intent

- e2e: a run whose runbook is discharged and conditions met reads `ready` with
  the stage payload; an outstanding step and an unmet forward condition render
  with their discharging acts; a locked run carries no `forward`; the exploring
  edge names its skipped check.
- Envelope golden at version 2 without `next_obligation`; bounding fixture
  covers a maximal forward set; `resume` carries no `runbook_section`.
- Old snapshots carrying `next_obligation: null` still deserialise.
- No stored field without a writer remains for `next_obligation`.
- Closes `IMP-390` (remaining faces); `IMP-367` updated for the disposition.

## Summary

## Follow-Ups
