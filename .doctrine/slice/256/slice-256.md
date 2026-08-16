# Recording an act emits a change row

## Context

`ISS-355` reports that a successful `agent_declaration` apply exits 0 and prints
only `revision N stage <stage>` — no change row. Confirming the declaration
landed costs a grep of raw runtime state, which the guardrails tell agents not
to read.

The mechanism is `record_declaration` (`src/design_run/run.rs:539`): it validates
the basis, resolves the fingerprint, admits the record against its gate rule, and
calls `next.declarations.record(record)` — then returns `Ok(())`. Nothing is
pushed onto `pending`, so nothing reaches `Applied::rows`, so the shell's
existing render at `src/commands/design.rs:1649` has nothing to render.

**The reported instance is one of two.** Its sibling `record_act`
(`run.rs:577`) emits a row only when the act carries a review disposition
(`ChangeEvent::ReviewDisposed`); a `checkpoint_act` without one records just as
silently. The run-level `acceptance` path (`run.rs:407`) — which builds a
`CheckpointAct` of kind `DesignAccepted` through the same constructor — *does*
push an `AcceptanceAttested` row. So the change log already treats one recorded
act as a material change and two others as invisible, with no stated reason for
the split. That asymmetry is the defect; `agent_declaration` is where it was
noticed.

A recorded act is only ever seen indirectly today, and only on its death:
`invalidation_rows` (`run.rs:1842`) emits `ActInvalidated` when a new act
displaces a live one. The log therefore reports acts ceasing to bind but never
acts being made — a first act of any kind leaves no trace at all.

### Why this matters beyond tidiness

`ISS-346` and `ISS-333` describe the silent-absorption class: an unknown payload
key is discarded, the revision bumps, a receipt is written, and the command exits
0 with the same output. `ISS-355`'s point is that the *success* output is
byte-identical to that failure, so the one observable that could separate
*landed* from *discarded* is absent exactly where it is needed. The two defects
compound.

This slice repairs the response half only. It does not touch the absorption
mechanism, which is `ISS-333`/`ISS-346`'s and is gated on `QUE-219`.

### Relationship to `SL-251`, in flight

`SL-251` (*Acts carry their own payload contract*) is being implemented in an
isolated capsule. Its design load-bears on this defect, explicitly
(`.doctrine/slice/251/design.md:422`):

> **And the missing change row is not a signal, which is why the disclosure has
> to be in the contract rather than in the response.** A *correct* submission
> prints no change row either (`ISS-355`) […] Repairing the response is
> `ISS-355`'s, and this design does not touch it.

`SL-251`'s conclusion survives this repair. Its hole is a misspelt key *inside*
an otherwise-valid struct — its worked example is `cursor` inside
`TraversalDeclaration`, one of the nine wire structs without
`deny_unknown_fields`. A row confirming that an act was recorded gives no
observable for a key dropped within that act, so buying discoverability before
submission is still the right purchase. What changes is that the paragraph's
unqualified form stops being true.

Coordination owed, tracked here rather than left to be discovered: `SL-251`'s
design.md ¶ at 422–428 and its ledger row at 2289 (*"`ISS-355` — not a test and
not repaired here"*) need a one-clause touch at that slice's reconcile. This
slice does not edit `SL-251`'s artefacts.

## Scope & Objectives

Make the recording of an act a member of the material-change vocabulary, so that
a submission which records one says so.

1. Decide the vocabulary shape — whether one event covers both recorded-act
   kinds or each takes its own — and add the member(s) to `ChangeEvent`
   (`src/design_run/change_log.rs:56`), with `ALL`, `as_str`, and
   `payload_terms` kept in step.
2. Emit the row from `record_declaration`, and from `record_act` on the path
   that currently emits nothing.
3. Settle the `AcceptanceAttested` asymmetry — either it becomes an instance of
   the new event or the design states why it stays distinct. Two spellings for
   one thing is what the no-parallel-implementation rule is about.
4. Pin the behaviour where it broke: an apply that records an act renders a row
   through the existing `commands/design.rs:1649` path.

### Affected surface

| path | why |
|---|---|
| `src/design_run/change_log.rs` | the closed vocabulary, its `ALL` roster, `as_str`, `payload_terms` |
| `src/design_run/run.rs` | `record_declaration`, `record_act`, the `acceptance` arm; tests in its own `#[cfg(test)]` module (line 1869) |
| `src/design_run/render/mod.rs` | `render_row` (line 358) and its tests, if the new terms need rendering work |
| `src/design_run/render/change_row.rs` | row-render internals, if reached |
| `src/design_run/bounds.rs` | only if a new event name outgrows `DESIGN_EVENT_NAME_BYTES` |
| `src/design_run/snapshot.rs` | the literal-fragment compat pin (test module) |
| `tests/e2e_design_state.rs` | **mandatory** — `every_material_event_kind_persists_a_change_row` (`:1081`) enumerates `ChangeEvent::ALL` and reds on a member the `every_event_fixture` ladder (`:838`) does not drive. This is where change rows are asserted; `run.rs`'s own test module asserts only on snapshot state. Named nowhere in `SL-251`. |

`src/commands/design.rs` is expected to need **no** change: line 1649 already
extends the output with every row in `Applied::rows`.

## Non-Goals

- **Repairing the absorption mechanism.** Unknown-key refusal is `ISS-333` /
  `ISS-346` / `ISS-327` / `ISS-328`, gated on `QUE-219`. A row saying an act was
  recorded is not a claim about the keys inside it, and the design must not let
  it read as one.
- **`SL-251`'s design-targets.** Deliberately out of bounds:
  `src/design_run/submission.rs`, `src/design_run/attestation.rs`,
  `src/design_run/tests.rs`, `src/design_run/mod.rs`,
  `src/design_run/render/envelope.rs`, `src/design_run/payload_contract.rs`,
  `src/commands/design.rs`, `src/commands/guard.rs`, `src/main.rs`. The
  disjointness is recorded so `doctrine slice conformance` catches drift into
  the capsule's files rather than a merge discovering it.
- **Making the envelope name the next act.** That is `IMP-390`, sequenced after
  `SL-251`.
- **Retiring or renaming any existing `ChangeEvent` member.**

## Risks & Assumptions

- **R1 — stored vocabulary.** `ChangeEvent` deserialises strictly: one
  unrecognised `event` token fails the whole snapshot, not one row. That is how
  `ISS-315` happened. Adding a member is the safe direction (a new binary reads
  every old token), but a snapshot written by this binary is unreadable to an
  older one. `mem.fact.design-run.snapshot-outlives-the-binary` holds the rule:
  additive-and-defaulted is safe, a rename needs an alias, a retirement needs a
  decision. Design must state which of those this is and pin the compat at
  `snapshot::parse` over a literal fragment, not a unit round-trip.
- **R2 — vocabulary closure discharges a requirement.** `change_log.rs:46`
  asserts `widest(ChangeEvent::ALL) <= DESIGN_EVENT_NAME_BYTES` at compile time.
  Research upgrades this from prudence to conformance: it is how **`REQ-437`**
  (SPEC-029 NF-002, *hold named projection limits against a large run*) is
  discharged, and that requirement's acceptance criteria demand named constants
  in one place with assertions against them. `DESIGN_EVENT_NAME_BYTES = 32`
  against a current widest of 27, so there are **five bytes of headroom** — a
  hard constraint on naming, not a matter of taste. `ALL` also carries a
  hardcoded arity (`[ChangeEvent; 22]`) that must move with the enum.
- **R3 — retired by research.** *Original concern: the projection-bounds sketch
  closes the vocabulary, so design must show a recorded act passes its
  state-vs-delta test rather than merely being useful to print.* The round
  answers it: the sketch (`.doctrine/slice/233/sketches/projection-bounds.md`) is
  a slice design artefact, not governance; its §(d) table (`:428-439`) never
  considered acts at all; and ten members have been admitted past that table
  since — including `AcceptanceAttested` itself. Design applies the sketch's own
  two-part test (delta, and not ceremony-exceeding-value) and cites the
  precedent. The burden dropped. See `research.md` Thread 1 §3.
- **R5 — a const whose name would become false.** `WIDEST_PAYLOAD_EVENT`
  (`src/design_run/render/mod.rs:198`) hardcodes `ChangeEvent::StageMoved` as the
  widest-payload exemplar and drives `WIDEST_PAYLOAD_SEPARATORS` / the
  `SKETCH_WIDEST_ROW_BYTES` pin. A new member with **more than three terms**
  leaves that const wrong while its compile-time assert stays green, because the
  assert only tests `StageMoved`'s shape. Either hold the new row to ≤3 terms or
  make the site programmatic. Bears directly on `OQ-3`.
- **R4 — `SL-251` merge.** Textual overlap with the capsule is empty by the
  file list above. The residual is a test in the capsule pinning *current*
  behaviour (an apply asserting an empty row set). `SL-251`'s planned VTs are
  contract-table tests, so this is unlikely — and it would surface as a red test
  on import, not as silent divergence.
- **A1 — no spec requirement is at stake.** SPEC-029's roster (`REQ-428` …
  `REQ-438`) covers schema versioning, CAS, submission replay, id reservation,
  adoption, the single envelope, the watermark, and prompt composition. None
  governs the change log's completeness — `spec-029.md` contains no occurrence
  of "change log", "change row", "material change", or "delta". Research
  confirms: **no REV is required**, against either SPEC-029 or an ADR. If design
  concludes that *recording an act is a material change* deserves durable
  standing, that is a **new REQ minted under SPEC-029** — additive authoring,
  still not a REV. Only retiring or renaming an existing member would force one,
  and that is a non-goal. Note also that **STD-003** (*no silent skip*) misses by
  its wording — it governs degraded *reads* of authored data, not silent
  *emission* — while being the closest thing in the corpus to a principle that
  covers `ISS-355`. Design decides whether the durable statement owed is a new
  REQ or a widening of STD-003.

## Open Questions

- **OQ-1** — one event or two? A single `ActRecorded` carrying the act kind as a
  term, versus `DeclarationRecorded` + `CheckpointActRecorded`. The former keeps
  the roster small and matches `ActInvalidated`, which already reports by
  subject with the act as a term; the latter renders more legibly.
- **OQ-2** — does `AcceptanceAttested` fold into the new member, stay as a
  special case, or get restated as one? Folding it changes an existing token's
  meaning, which R1 makes non-free.
- **OQ-3** — should `record_declaration`'s row carry the declared act's kind,
  its basis, its fingerprint, or the coverage it claimed? The terms decide what
  an agent can confirm without reading the snapshot, which is the whole point of
  `ISS-355`.

## Verification & Closure Intent

- A `design apply` carrying only an `agent_declaration` emits at least one
  change row, asserted in `tests/e2e_design_state.rs` in the idiom of
  `a_waived_disposition_row_names_its_arm_and_carries_its_reason` (`:1110`) —
  row found by event, subject id and ordered terms asserted.
- The same holds for a `checkpoint_act` with no disposition.
- `every_material_event_kind_persists_a_change_row`
  (`tests/e2e_design_state.rs:1081`) stays green, which requires the
  `every_event_fixture` ladder (`:838`) to drive the new member — the test that
  makes an unwired member fail loudly rather than be quietly absent.
- `rendered_payload_fits_its_cap_for_every_event_kind` (`:1219`) stays green,
  which is the real guard on R5.
- A snapshot fragment written with the new token round-trips through
  `snapshot::parse`, and the pre-existing legacy-fragment tests stay green.
- `doctrine slice conformance` reports no edit outside the fenced surface — in
  particular none inside `SL-251`'s design-targets.
- `ISS-355` closed; the `SL-251` coordination note discharged or handed on.

## Summary

## Follow-Ups
