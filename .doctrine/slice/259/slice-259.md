# Truthful apply

## Context

`doctrine design apply`'s exit signal cannot be trusted, and every agent retry or
recovery decision rests on it. The `cluster:design-run` friction record shows three
ways it lies:

| lie | items |
|---|---|
| **success for input it ignored** — unknown keys absorbed, revision bumped, receipt written | `ISS-333` (top-level, `#[serde(flatten)]` blocks `deny_unknown_fields`), `ISS-328` (nested in `CreateRecord`), `ISS-327` (keys inert at the subject's state), `ISS-290` (payload `ValueKind` never checked) |
| **rows that under-report** — a material change with no row | `ISS-367` (same-kind act replacement emits no `ActInvalidated`), `ISS-450` (a `needs` edge declared at node creation emits no `needs_added`) |
| **failure reported, state moved** — reported once; code reading traces it to the silent-absorb class, not the parse site | `ISS-361` |

And the read path breaks the other way: one retired `ChangeEvent` variant makes a
whole historical snapshot unreadable (`ISS-315`; `design show 244` fails today).

**Struck during design.** `ISS-362` (*act admissible at the wrong stage silently
no-ops*) was a fourth row here until repro disproved its premise: a `cp-` dispose
and a `sec-` body are both *honoured* at `stage = exploring`, rows emitted. The
witnessed inertness was `ISS-355`'s missing row, which `SL-256` fixed. The
residual question — whether stage should gate those acts at all, given today it
does not — is a state-machine scoping question, not a defect in apply's exit
signal, and is out of scope. Disproof recorded on `ISS-362`.

`SL-256` delivered one leg (a recorded act emits a change row). This slice finishes
the contract. Part of the `RFC-031` (*Design run fitness*) program.

## Scope & Objectives

One invariant, four legs:

1. **Error ⇒ nothing landed.** No revision, no receipt, no change row on any
   refused or failed submission. Establish or rule out `ISS-361`'s reported
   mechanism by repro.
2. **Success ⇒ rows that tell the truth.** Every material change emits its row;
   `ISS-367`'s same-kind replacement emits `ActInvalidated`, and `ISS-450`'s
   creation-time `needs` edge emits `needs_added`.
3. **Unknown or inert input ⇒ typed refusal.** Unknown keys at every nesting level,
   wrong-`ValueKind` payload terms, and keys inert at the subject's state are
   refused, naming what was expected — the keys admitted, and where.
4. **Old snapshots stay readable.** A snapshot written by an older binary parses;
   an unrecognised historical row degrades visibly (`STD-003`: a degraded read is
   disclosed), never fails the whole file.

**`QUE-219` is settled** (*Submission strictness against snapshot
forward-compatibility*), as `DEC-243`. The working hypothesis — strict write path,
tolerant read path, split by type — does not survive `EVD-027`: `Declaration` is
both the wire type and a stored type (`delegation.rs:69` holds a delegate's
declarations verbatim in the snapshot) and already denies unknown fields, so there
is no type seam to split on. The ruling instead moves the axis: **strictness
refuses what was never known; a retired-member roster carries what was known and
then retired**, generalising `SL-256`'s `READABLE`/`EMITTABLE` split from
change-event tokens to wire keys, with an `STD-003` disclosed-degradation floor
beneath it so a forgotten roster entry costs one row's legibility rather than the
file. `DEC-244` settles the mechanism: write-path strictness is driven off
`payload_contract`'s pinned key inventory rather than off `deny_unknown_fields`,
which structurally cannot reach the flatten envelope, the internally-tagged enums,
or untagged `WireFacetValue`. Together they unblock `ISS-333`, `ISS-328`,
`ISS-327`, `ISS-290`.

## Non-Goals

- Contract legibility / discovery (`IMP-390`, `ISS-298`, `ISS-360`) — refusals must
  *name* what was expected, but no new read verbs.
- The section edit loop (`ISS-320`, `ISS-348`, `IMP-430`) — next slice in `RFC-031`.
- Inquiry-map semantics (`QUE-218` batch), `IMP-392` (findings onto RV).
- Migrating stored snapshots in place — tolerance is on the reader.
- Splitting `Declaration` into separate wire and stored types (`DEC-243`'s first
  rejected alternative) — the clean layering repair, deferred as known debt
  because `EVD-027` shows no live snapshot exercises the overlap.
- Whether stage should gate acts it does not gate today — the residual of the
  struck `ISS-362`.

## Affected surface

- `src/design_run/submission.rs` — wire structs, `ApplyRequest` flatten envelope
- `src/design_run/snapshot.rs` — `ChangeEvent` and snapshot parse
- `src/design_run/run.rs` — `admit`, stage admission, `live_acts` / `invalidation_rows`
- `src/commands/design.rs` — `apply` parse → admit → persist → receipt ordering
- design-run e2e suites

## Risks, assumptions, open questions

- **R1 — strictness breaks live runs.** Bound by
  `mem.fact.design-run.snapshot-outlives-the-binary`. **Measured** (`EVD-027`): all
  16 live snapshots were probed, 15 parse, and `SL-244` is the sole casualty. None
  carries a stored proposal (`delegation = []` throughout), so the wire/stored
  overlap is latent rather than live. Still pin compat at `snapshot::parse` over
  literal legacy fragments — the measurement dates fast.
- **R2 — flatten + strictness.** `mem_019fd03e…` (retiring a wire field on a
  flatten-carrying request is a silent no-op). **Resolved by `DEC-244`**: the fix
  neither replaces nor decorates the flatten, it moves the check off serde
  entirely, so the attribute's incompatibilities stop being load-bearing.
- **A1** — `ISS-361` collapses into the silent-absorb class. Unverified; repro
  first. Code reading weakens it: `commands/design.rs:1624` parses **before**
  `run::admit`, so the obvious path cannot reach a receipt. The mint/journal seam
  (`execute_mint` writes journal entries keyed by `submission_id` before the second
  `run::apply`, which can still refuse) is the live suspect — `inq-6`.
- **OQ** — does a tolerant reader *preserve* an unknown row on rewrite, or drop it
  with a disclosure? (Rewrite-loses-history vs. opaque-row carrying.) Open as
  `inq-15`; sharpened by the fact that the change log is append-only history and
  every apply rewrites the whole file, so "drop" means "lose on next write".

## Verification / closure intent

- VT per leg: a refused submission leaves snapshot bytes, revision and receipts
  unchanged; each unknown/inert/wrong-kind input yields a typed refusal whose
  *reason* is pinned (`mem_01a00d4c…`); same-kind replacement emits
  `ActInvalidated` and a creation-time `needs` edge emits `needs_added`; a literal
  legacy snapshot fragment containing a retired `ChangeEvent` parses and discloses.
- VA: `doctrine design show 244` reads.
- Closes, or records partial fulfilment of, every item in the Context table plus
  settles `QUE-219`.

## Summary

## Follow-Ups
