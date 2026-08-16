# Implementation Plan SL-256: Recording an act emits a change row

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Three phases, all engine-side. The design (`design.md`) is unusually concrete —
it names the sum, the seam's signature, the call sites, every check and every
line it expects to move — so this plan adds no design content. What it adds is
the one thing the design does not settle: **an order in which the tree is green
at every boundary.**

That order is not free. Three edits in this slice are mutually entangled:

1. `ActRecorded` has to be *emitted* before it can be *required*.
2. Deleting the `AcceptanceAttested` push makes a member of `ChangeEvent::ALL`
   undriven — and `every_material_event_kind_persists_a_change_row` fails the
   moment a member of the roster it enumerates is no longer written. So the push
   deletion and the roster split are one phase, not two.
3. Renaming the variant while `#[serde(rename_all = "snake_case")]` still derives
   its token would drift the wire spelling away from `as_str` — the exact defect
   `sec-2` documents. So the token single-sourcing must precede the rename.

The phases fall straight out of those three constraints.

## Sequencing & Rationale

### PHASE-01 — the shared record seam emits `ActRecorded`

The repair proper: `ActRecord`, `admit_and_record`, the deletion of
`admit_against`, and the three call sites collapsed onto one route (`sec-3`,
`DEC-238`). `ActRecorded` joins the vocabulary in the same phase because the
seam's return type is what forces it into existence.

`ISS-355`'s report is `VT-1` and it is the red this slice starts from. `VT-2` is
its sibling — the `record_act` path with no disposition, found in scoping rather
than reported — and `VT-3` is the ordered pair, which exists because `sec-3`
names the way an implementor gets it wrong: construction order runs opposite to
vector order, since the disposition row is built before `admit_and_record` is
called while the mandatory `ActRecorded` row does not exist until it returns.

The phase deliberately leaves `AcceptanceAttested` alone. For the duration of one
phase the acceptance arm emits two rows — `ActRecorded` and `AcceptanceAttested`
— which is redundant but green: every member of the single roster is still
driven. Doing it the other way round, deleting the push here, would leave the
tree red between phases with no way to end the phase honestly.

`VA-1` exists because a keyword floor can assert presence but not absence, and
the whole point of `sec-3`'s deletion is that no second admission route survives.

### PHASE-02 — split the roster, delete the push

`READABLE` and `EMITTABLE` (`DEC-239`), the `const fn is_subset` proof, and the
push deletion together. They arrive together for the reason given above: the
roster split is what makes an undriven member legal, so it must land in the same
commit that stops driving it.

The subset relation is proved at compile time rather than asserted at runtime,
in the idiom `change_log.rs` already documents beside `widest`. `sec-4` records a
second reason the assert must not later be deleted as redundant: after the split
it is the **only** reader of `EMITTABLE` in `src/`, both roster tests living in a
separate compilation unit, and the crate denies `unused` — so dropping it passes
`cargo check` and fails `cargo test --bin doctrine`. Anyone tempted to tidy it
should read that paragraph first.

`VT-3` (`no_row_the_ladder_produces_carries_a_non_emittable_event`) is carried as
**defence in depth and not as discharge of any `REQ-478` criterion** — `RV-360`
`F-2`'s residual. `Pending::about` and `Pending::run_wide` accept any
`ChangeEvent`, so no roster array can prove that nothing outside it is written;
what this check proves is narrower and worth having, and the plan states its size
rather than letting a criterion absorb it.

The Rust identifier does **not** move here. That is the third constraint, and it
is the whole reason there is a PHASE-03 at all.

### PHASE-03 — one source for the token, then the rename

Single-source first (`try_from`/`into` over `as_str`, `Refusal::UnknownChangeEvent`,
the one named `LEGACY_ACT_INVALIDATED` literal), rename second. With `as_str` the
only source, `AcceptanceAttested` → `LegacyAcceptanceAttested` moves the Rust
identifier and nothing else: same wire token, same rendered token, same run-wide
term-free payload. `STD-001` is why the cheaper `#[serde(rename = …)]` pin was
refused — a drift guard detects divergence, it does not single-source anything
(`RV-360` `F-4`).

`VT-2`'s legacy-fragment test is pinned at the whole-file tier deliberately:
`ChangeEvent` deserialises strictly, so one unrecognised token fails an entire
snapshot rather than one row. That is `ISS-315`'s defect class, and the live
population is real — `event = "acceptance_attested"` across
`.doctrine/state/slice/*/design.toml` measured 7 rows over 6 live snapshots at
design time, and it is a floor rather than a constant because live runs accrue
rows. (SL-256's own run has accrued one more since; re-run the predicate rather
than trusting the number.)

`VA-2` closes the loop `sec-4` opened: the `REQ-478` coverage cell binds a
runnable positive control naming one test, not `--mode VT` alone — which records
no check at all, takes the attestation branch, and stores a `Verified` status
with no test behind it (`RV-360` `F-3`). It is recorded at the end of this phase
because the checks exist by then; `REQ-478` itself moves `pending` → `active` at
close.

## Notes

- **Fence.** The selector list is already current — the `render/**` glob was
  narrowed to `render/mod.rs` + `render/change_row.rs` and `refusal.rs` added, at
  `RV-360` integration. `sec-4` expects both render files to come out of the
  slice untouched; they are fenced so that a change to either reads as a
  conformance signal rather than a silent edit. `src/design_run/tests.rs` and
  `src/design_run/fixture.rs` sit inside the blast radius and outside the fence
  on purpose (they are `SL-251` design-targets) — if either turns out to need an
  edit, that is a scope question and a selector widening recorded at plan time,
  never a quiet edit.
- **No test file outside the fence should move.** `sec-4` argues this per fixture
  rather than by generalisation, having had two absolutes falsified (`RV-360`
  `F-7`, `F-8`). The residual is stated there: if the suite disagrees at
  execution, the remedy is a recorded selector widening and a conformance re-run.
- **`ISS-367` is not this slice's.** `live_acts` is blind to same-kind
  replacement, so `ActInvalidated` under-reports. It is sequenced `after SL-256`
  and both `sec-1` and `sec-3` state the boundary, so nothing here should grow to
  meet it.
- **`SL-251` owes a three-site touch at its own reconcile** — its `design.md`
  ¶ 422–428, its ledger row at 2289, and `payload_contract.rs:501`. This slice
  does not edit `SL-251`'s artefacts; the coordination is tracked in the scope's
  Follow-Ups.
- **Research advisory is stale** (`doctrine slice research 256` reports drift from
  `slice-256.md` and the added `design.md`). The design supersedes it for
  planning purposes — every anchor in `sec-3` and `sec-4` was re-resolved against
  the current tree at plan time and all resolve. Restamping the research artefact
  is not a precondition for execution.
- **`plan.toml` carries no spec / requirement keys.** `SPEC-029` and `REQ-478`
  are already on the slice as relations (`references(implements)`, and `REQ-478`
  named throughout `design.md` `sec-4`). The v1 plan reader models neither, so
  restating them in the TOML would be a second source with nothing consuming it.
- **`is_subset` has one const-safe spelling.** `str` equality is not const, and
  a byte-index loop trips `clippy::indexing_slicing` — the same lint `sec-4`
  names for a different shortcut. The spelling that works is the module's own
  slice-recursion idiom, the one `widest` (`change_log.rs:31-40`) already uses,
  recursing over `as_bytes()` with a slice pattern. Worth knowing before
  PHASE-02 starts rather than after the lint gate says so.
