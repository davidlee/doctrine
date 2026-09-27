# Review RV-407 — reconciliation of SL-272

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Mode: conformance. Surface: `edge` at `828444c96` (solo `/execute`, not
dispatched — no candidate branch). Code commits `d21b571a8`, `94e655d92`
(PHASE-01), `5d1bbd29d`, `3dd526fcb` (PHASE-02).

Lines of attack:

- Every design sec-4 path delivered; every touched path declared
  (`slice conformance`).
- `SL-259`'s exit-signal contract holds on each item: `ISS-482` refused on
  both the direct and proposal routes through `Batch::validate`, after
  `inert_at_state`; `IMP-499`'s remedy never tells a caller to resubmit
  blind; `ISS-454` reports each death once, per the sec-3 four-case table
  (`DEC-336`); `ISS-488` emits only on changed text, never at creation
  (`DEC-335`).
- VT-1..VT-6 exist under their plan names and each was red before its fix.
- Test-suite totality claims (roster, term matrix, exemplar census) still
  hold after the new `ChangeEvent` and `Presence` members.
- Implementation-time deviations recorded in `notes.md` match the design or
  are routed to reconcile.
- `doctrine check gate` green.

## Synthesis

The four `RFC-031` clump-1 defects are closed as designed, and `SL-259`'s
exit-signal contract now holds at each:

- `ISS-482` — `Declaration::finding_blocking_null` runs in `Batch::validate`
  after `inert_at_state`, the one seam the direct apply and the proposal
  rehearsal both cross; the proposal is refused before storage (VT-2 of
  PHASE-01). The payload contract publishes it as `OptionalNullRefused`.
- `IMP-499` — `SubmissionExpired` directs a check against current run state
  and a resubmit of only what is absent; it never suggests a blind retry.
- `ISS-454` — the `reportable` set (live at `prior` ∪ recorded earlier in the
  apply) gates the displacement row, covering all four rows of design
  sec-3's table and the double-acceptance lock recipe; the `ISS-367` positive
  control stays green.
- `ISS-488` — `node_question_changed` is term-free, emitted only when the
  resolved text differs; creation is covered structurally by the seeded
  prior. The roster fixture drives it.

Evidence: `doctrine slice verify-vt 272` passes all seven criteria;
`doctrine check gate` green at `828444c96`; conformance 8 conformant,
0 undelivered, 1 undeclared (F-1).

Findings: F-1 and F-2 are design prose that fell behind correct
implementation choices — delegated to reconcile. F-3 was a real test-suite
regression of a totality claim (the exemplar census), fixed in audit.

Standing risks, accepted in design sec-6 and unchanged: older binaries show
`node_question_changed` as a disclosed unreadable row (`DEC-249`); a revived
act (coverage restored) is the one pre-existing exception to `DEC-336`'s
invariant; `IDE-057` remains `REQ-478`'s open case.

## Reconciliation Brief

### Per-slice (direct edit)

- F-1 — `doctrine slice selector add` a `design-target` selector for
  `src/design_run/tests.rs` (load-bearing: `slice conformance` reads the
  registry); mirror it with a design.md sec-4 row: "`a_term_constructed_at_an_undeclared_kind_is_refused`
  admits a term-free event as one cell (`DEC-335`)", and "the eight paths"
  → "the nine paths".
- F-2 — design.md sec-2: the `OptionalNullRefused` annotation reads
  `(omit means absent · null refused)` (one note per `Presence` variant),
  not `(omit → non-blocking · null refused)`.

### Governance/spec (REV)

- None.

## Reconciliation Outcome

### Direct edits applied
- F-1: `doctrine slice selector add --intent design-target 272
  src/design_run/tests.rs` (load-bearing; `slice conformance` now 9
  conformant, 0 undeclared, 0 undelivered). Mirrored in design.md sec-4: a
  `tests.rs` row, and "the eight paths" → "the nine paths".
- F-2: design.md sec-2 — the `OptionalNullRefused` annotation now reads
  `(omit means absent · null refused)`, with why (one note per presence
  state).

Both edits land out of band on the locked design run; the fingerprint
divergence is expected at reconcile. User agreed to both in session.

### REVs completed
- None — the brief carried no governance/spec items.

### Withdrawn / tolerated
- None. F-3 was fixed in audit (`b0d8f9963`).

Reconcile pass complete — handoff to /close.
