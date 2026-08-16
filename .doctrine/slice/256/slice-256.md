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
- **R2 — vocabulary closure is load-bearing.** `change_log.rs:46` asserts
  `widest(ChangeEvent::ALL) <= DESIGN_EVENT_NAME_BYTES` at compile time, and the
  containment check enumerates `ALL`. A new member must be added to the roster,
  not just the enum, or the build's own proof goes stale. The current widest is
  `section_fingerprint_changed` (27 bytes), so a plausible new token is unlikely
  to move the bound — but "unlikely" is not the check; the assert is.
- **R3 — the sketch says these are not members.** The projection-bounds sketch
  §(d) closes the vocabulary deliberately: "Cursor moves, posture changes,
  receipt eviction and fragment receipts are deliberately **not** members — they
  are state, not delta." Design must show that recording an act is a delta on
  that same test, and not merely useful to print. `AcceptanceAttested`'s
  existing membership is the strongest evidence that it is.
- **R4 — `SL-251` merge.** Textual overlap with the capsule is empty by the
  file list above. The residual is a test in the capsule pinning *current*
  behaviour (an apply asserting an empty row set). `SL-251`'s planned VTs are
  contract-table tests, so this is unlikely — and it would surface as a red test
  on import, not as silent divergence.
- **A1 — no spec requirement is at stake.** SPEC-029's roster (`REQ-428` …
  `REQ-438`) covers schema versioning, CAS, submission replay, id reservation,
  adoption, the single envelope, the watermark, and prompt composition. None
  governs the change log's completeness. This is conformance against the
  `SL-233` design sketch, not against a `REQ` — so design should decide whether
  a requirement is owed.

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
  change row, tested at the engine (`run.rs`'s own test module) and pinned
  through the render path.
- The same holds for a `checkpoint_act` with no disposition.
- The `ChangeEvent` roster, `as_str` table, and `payload_terms` table stay
  set-equal — the existing exhaustive tests carry this and must stay green
  unchanged where they are not about the new member.
- A snapshot fragment written with the new token round-trips through
  `snapshot::parse`, and the pre-existing legacy-fragment tests stay green.
- `doctrine slice conformance` reports no edit outside the fenced surface — in
  particular none inside `SL-251`'s design-targets.
- `ISS-355` closed; the `SL-251` coordination note discharged or handed on.

## Summary

## Follow-Ups
