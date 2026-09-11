# Truthful apply

## Context

`doctrine design apply`'s exit signal cannot be trusted, and every agent retry or
recovery decision rests on it. The `cluster:design-run` friction record shows four
ways it lies:

| lie | items |
|---|---|
| **success for input it ignored** — unknown keys absorbed, revision bumped, receipt written | `ISS-333` (top-level, `#[serde(flatten)]` blocks `deny_unknown_fields`), `ISS-328` (nested in `CreateRecord`), `ISS-327` (keys inert at the subject's state), `ISS-290` (payload `ValueKind` never checked) |
| **success for an act that did nothing** — act admissible in form, inert at the current stage | `ISS-362` |
| **rows that under-report** — same-kind act replacement emits no `ActInvalidated` | `ISS-367` |
| **failure reported, state moved** — reported once; code reading traces it to the silent-absorb class, not the parse site | `ISS-361` |

And the read path breaks the other way: one retired `ChangeEvent` variant makes a
whole historical snapshot unreadable (`ISS-315`; `design show 244` fails today).

`SL-256` delivered one leg (a recorded act emits a change row). This slice finishes
the contract. Part of the `RFC-031` (*Design run fitness*) program.

## Scope & Objectives

One invariant, four legs:

1. **Error ⇒ nothing landed.** No revision, no receipt, no change row on any
   refused or failed submission. Establish or rule out `ISS-361`'s reported
   mechanism by repro.
2. **Success ⇒ rows that tell the truth.** Every material change emits its row;
   `ISS-367`'s same-kind replacement emits `ActInvalidated`.
3. **Unknown or inert input ⇒ typed refusal.** Unknown keys at every nesting level,
   wrong-`ValueKind` payload terms, and acts inert at the current stage are refused,
   naming what was expected (the stage an act needs; the keys admitted).
4. **Old snapshots stay readable.** A snapshot written by an older binary parses;
   an unrecognised historical row degrades visibly (`STD-003`: a degraded read is
   disclosed), never fails the whole file.

**First design decision: settle `QUE-219`** (*Submission strictness against snapshot
forward-compatibility*). Working hypothesis: strict on the write path, tolerant on
the read path — the two are different types or different deserialisation modes, so
tightening a submission never tightens a stored declaration. Settling it unblocks
`ISS-333`, `ISS-328`, `ISS-327`, `ISS-290`.

## Non-Goals

- Contract legibility / discovery (`IMP-390`, `ISS-298`, `ISS-360`) — refusals must
  *name* what was expected, but no new read verbs.
- The section edit loop (`ISS-320`, `ISS-348`, `IMP-430`) — next slice in `RFC-031`.
- Inquiry-map semantics (`QUE-218` batch), `IMP-392` (findings onto RV).
- Migrating stored snapshots in place — tolerance is on the reader.

## Affected surface

- `src/design_run/submission.rs` — wire structs, `ApplyRequest` flatten envelope
- `src/design_run/snapshot.rs` — `ChangeEvent` and snapshot parse
- `src/design_run/run.rs` — `admit`, stage admission, `live_acts` / `invalidation_rows`
- `src/commands/design.rs` — `apply` parse → admit → persist → receipt ordering
- design-run e2e suites

## Risks, assumptions, open questions

- **R1 — strictness breaks live runs.** Bound by
  `mem.fact.design-run.snapshot-outlives-the-binary`. Probe the five live snapshots
  (`SL-243`, `244`, `246`, `251`, `256`, `258`) for every token touched; pin compat
  at `snapshot::parse` over literal legacy fragments.
- **R2 — flatten + strictness.** `mem_019fd03e…` (retiring a wire field on a
  flatten-carrying request is a silent no-op): the fix likely replaces the flatten,
  not decorates it.
- **A1** — `ISS-361` collapses into the silent-absorb class. Unverified; repro first.
- **OQ** — does a tolerant reader *preserve* an unknown row on rewrite, or drop it
  with a disclosure? (Rewrite-loses-history vs. opaque-row carrying.)

## Verification / closure intent

- VT per leg: a refused submission leaves snapshot bytes, revision and receipts
  unchanged; each unknown/inert/wrong-kind input yields a typed refusal whose
  *reason* is pinned (`mem_01a00d4c…`); same-kind replacement emits
  `ActInvalidated`; a literal legacy snapshot fragment containing a retired
  `ChangeEvent` parses and discloses.
- VA: `doctrine design show 244` reads.
- Closes, or records partial fulfilment of, every item in the Context table plus
  settles `QUE-219`.

## Summary

## Follow-Ups
