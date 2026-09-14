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

1. **Error ⇒ nothing landed**, stated precisely (`DEC-250`). On any refused or
   failed submission: no revision, no receipt, no change row. The **authored**
   tier is unmoved too, except across one named late-check window that
   `SPEC-029` specifies and whose recovery is the journal — every check that
   *can* be hoisted ahead of the mints is, **on principle rather than against a
   witness**: `RV-365` `F-2` found no reachable refusal that clears pass 1 and
   fails pass 2, so the hoist buys construction-truth for a claim the code
   already makes, and the carve-out rests on the late watermark check alone. `ISS-361`'s reported mechanism is
   ruled out by repro (`EVD-028`); the item stays open against that residual.
2. **Success ⇒ rows that tell the truth.** Every material change emits its row;
   `ISS-367`'s same-kind replacement emits `ActInvalidated`, and `ISS-450`'s
   creation-time `needs` edge emits `needs_added`.
3. **Unknown or inert input ⇒ typed refusal.** Unknown keys at every nesting level,
   wrong-`ValueKind` payload terms, and keys inert at the subject's state are
   refused, naming what was expected — the keys admitted, and where. Refusal is
   a property of the *request* against schema and subject state, never of
   whether anything changed (`DEC-245`). `Outcome`'s declared kind moves to
   `Label` (`DEC-247`).
4. **Old snapshots stay readable**, on one line (`DEC-249`): **degrade what is
   history, refuse what is state.** A change-log row carrying an unrecognised
   token is *retained* — opaquely, at the row, never by widening `ChangeEvent`
   (`DEC-251`) — and disclosed as unreadable (`STD-003`), rather than failing
   the file. `Stage`, `IdKind` and `ActKind` keep strict parse: a run that
   cannot read its own stage is not a degraded row.

**`QUE-219` is settled** (*Submission strictness against snapshot
forward-compatibility*), as `DEC-243`. The working hypothesis — strict write path,
tolerant read path, split by type — does not survive `EVD-027`: `Declaration` is
both the wire type and a stored type (`delegation.rs:69` holds a delegate's
declarations verbatim in the snapshot) and already denies unknown fields, so there
is no type seam to split on. The ruling instead moves the axis: **strictness
refuses what was never known; a retired-member roster carries what was known and
then retired**, generalising `SL-256`'s `READABLE`/`EMITTABLE` split from
change-event tokens to wire keys, with an `STD-003` disclosed-degradation floor
beneath it **for change-log rows only**, so a forgotten roster entry there costs
one row's legibility rather than the file. `RV-365` `F-3` established the limit:
the floor cannot reach a stored `Declaration` inside `Proposal`, and `DEC-249`
forbids widening it there (degrade history, refuse state). On that one path the
roster is the sole guard — stated in the design's `sec-2`, repair tracked as
`IMP-446`. `DEC-244` settles the mechanism: write-path strictness is driven off
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
  because `EVD-027` shows no live snapshot exercises the overlap. Tracked as
  `IMP-446`, triggered on the first live non-empty `delegation`.
- Whether stage should gate acts it does not gate today — the residual of the
  struck `ISS-362`.

## Affected surface

- `src/design_run/payload_contract.rs` — the pinned key inventory `DEC-244`
  drives refusal from; nine `UnknownKeys::SilentlyDropped` rows become true
  statements rather than disclosures of a defect
- `src/design_run/submission.rs` — wire structs, `ApplyRequest`'s flatten envelope
- `src/design_run/change_log.rs` — `ChangeEvent`'s rosters and const proofs
  (untouched by `DEC-251`, deliberately), `ChangeLog`'s four-method surface plus
  its externally-read `floor` field, `RawRow`'s classified reason (`RV-365`
  `F-4`), `PayloadTerm::admit` for `DEC-247`
- `src/design_run/snapshot.rs` — snapshot parse; the legacy-fragment compat
  pins; **both** retain-by-kind act stores, `CheckpointActGroup::record` and
  `AgentDeclarationGroup::record` (`RV-365` `F-1`)
- `src/design_run/run.rs` — `declare_node`'s create/update split, `live_acts` /
  `invalidation_rows`, the `StepDischarged` term construction
- `src/design_run/render/envelope.rs` — the one production consumer of the log,
  via `since()`; gains the degraded-row disclosure
- `src/commands/design.rs` — `apply`'s parse → admit → mint → persist ordering
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
- **A1 — refuted.** `ISS-361` does *not* collapse into the silent-absorb class.
  `commands/design.rs:1624` parses before `run::admit`, and repro confirms a
  malformed payload leaves the revision untouched and does not consume its
  submission id (`EVD-028`). The surviving suspect is the late-check window
  `DEC-250` names, and `SPEC-029` prescribes most of that ordering — so what is
  left is narrower than the report. `ISS-361` stays open against it.
- **OQ — closed** as `DEC-251`: preserve, at the row. Two facts closed it. The
  change log is a bounded **32-revision window** (`CHANGE_LOG_REVISIONS`), not
  permanent history, so an opaque row's cost is bounded — though not short: the
  floor advances as `current - 32 + 1`, so `SL-244`'s revision-88 row survives to
  119 (`RV-365` `F-5`). And `ChangeLog` is encapsulated where it counts —
  `record` / `retain_window` / `covers` / `since`, and **no production read of
  `.rows` outside the type at all** (`RV-365` `F-6`) — so a row-level fallback
  leaves every `ChangeEvent` pin standing,
  where an `Opaque(String)` variant would break `Copy`, `const fn as_str`, and
  both compile-time proofs.
- **R3 — new.** Do not "clean up" `change_log.rs`'s two `const _: ()` proofs.
  The `is_subset(&EMITTABLE, &READABLE)` assert is the only reader of
  `EMITTABLE` in `src/`, so deleting it passes `cargo check` and fails
  `cargo test --bin doctrine`
  (`mem.pattern.rust.expect-dead-code-is-per-compilation-unit`).

## Verification / closure intent

- VT per leg: a refused submission leaves snapshot bytes, revision and receipts
  unchanged; each unknown/inert/wrong-kind input yields a typed refusal whose
  *reason* is pinned (`mem_01a00d4c…`); same-kind replacement emits
  `ActInvalidated` and a creation-time `needs` edge emits `needs_added`; a literal
  legacy snapshot fragment containing a retired `ChangeEvent` parses and discloses.
- VA: `doctrine design show 244` reads.
- Closes, or records partial fulfilment of, every item in the Context table.
  `QUE-219` is **settled** (`DEC-243`); `ISS-362` is **struck** on a disproved
  premise; `ISS-450` **joined** during design; `ISS-361` stays **open** against
  `DEC-250`'s residual window rather than being closed by this slice.

## Summary

## Follow-Ups
