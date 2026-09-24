# Review RV-383 — reconciliation of SL-262

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-262 (the envelope's derived `forward` edge replacing
`next_obligation`) against its locked design (DEC-290..DEC-294) and plan
PHASE-01..PHASE-04. Surface: `edge` at 84e3f528f, the slice's own commits
(25c23a2d0, dd1de364a, fe1b986f3, e6207d20f). Not dispatched.

Lines of attack:

- **One builder** (DEC-292): `GateFacts` has one production constructor, and
  apply and every read go through it.
- **Forward agrees with advance**: derivation order (divergence → runbook →
  unmet), `ready` only when nothing blocks, fail-closed on a missing runbook.
- **Placement and bounds** (DEC-293): prompt, status, JSON and resume carry
  `forward`; `resume`'s runbook section is gone; the cause cap discloses `(+N more)`.
- **Mechanical conformance**: `slice conformance` leads, each dispositioned.
- **Read cost** (design sec-9 risk, still open in notes).
- **Gate**: `doctrine check gate` green; `slice verify-vt` all PASS.

## Synthesis

The slice delivers what the design locked. `gate_facts` is the only production
constructor of `GateFacts` (`commands/design.rs`); both `apply` and the
shared read path `project()` call it, so `forward` evaluates the facts `apply`
would. `forward()` in `render/envelope.rs` follows the design's derivation
order. `ready` is gated on no divergence, a present and cleared runbook, and
no unmet rows. It fails closed when the runbook is missing, as `advance` does.
The real binary on four live runs (drafting, inquiring, exploring, locked)
rendered consistent `forward` rows, including a `ready` payload with the
skipped check named as `unchecked`. `doctrine check gate` is green, and every
VT across PHASE-01..04 passes.

Findings were minor. One code defect was fixed in audit (F-1, a doc comment
stranded by the PHASE-01 insertion). Three items were handed to reconcile:
the selector registry (F-2), design prose lagging three small, recorded
PHASE-02 departures (F-3), and the spec REVs design sec-6 always intended
(F-4). The read-cost risk was measured and does not materialise (F-5).

Standing risks:

- `forward` is only as bounded as the embedded runbooks. A project runbook
  override (IMP-372) must bring its own admission bound. That is recorded for
  close, not built here (design sec-5).
- A read cannot see `regressed` runbook steps. `unchecked` discloses them, so
  a `ready` row can still be refused on apply if a verify check now fails.
  This was accepted in DEC-294.
- A stage move batched with the acts that satisfy it is admitted, but
  `forward` describes stored state only. PHASE-03 pinned this and recorded it
  as out of scope.

Accepted tradeoff: facts are built even at `locked`, where `forward` is None.
It costs two small reads and keeps `project()` free of a special case.

## Reconciliation Brief

### Per-slice (direct edit)

- **F-2, selector registry** (load-bearing verb):
  `doctrine slice selector add SL-262 src/design_run/fixture.rs src/design_run/prompt.rs --intent design-target`;
  `doctrine slice selector rm SL-262 tests/e2e_design_show_golden.rs tests/e2e_design_projection.rs`
  (the `tests/e2e_design_*.rs` glob already covers them). Mirror both in
  design sec-7's code-impact table.
- **F-3, design.md**: sec-1 mermaid and prose use `gate_facts` / `GateFacts`,
  not `observe` / `Observed`. In sec-2 *Type*, `Divergence` carries the
  rendered `refusal` sentence beside `expected`/`observed`, because the leaf
  cannot render `SL-NNN` and JSON must carry the prompt's text. In sec-2
  *Derivation* step 6, `ready` is `ApplyRequest { stage, ..ApplyRequest::bare(..) }`,
  and `bare` is the exhaustive literal that keeps a new key a compile error.
  In sec-4, the old-snapshot fixture is an absent key plus a string key,
  because TOML has no `null`.

### Governance/spec (REV)

- **F-4** `PRD-019` `REQ-414`: drop "pending obligation" from the held list,
  because it is now derived (`forward`).
- **F-4** `SPEC-029` `REQ-437`: add the forward edge to the named limits, and
  the maximal-forward case to the bounding run.
- **F-4** `SPEC-029` responsibilities: the envelope's forward look is a
  projection of the gate-contract table, beside the refusal and the
  stage-entry receipt.
- No REV for DEC-293 superseding SL-233 `EX-15`'s envelope half, because no
  spec carries EX-15. Reconcile records the supersession only.

### Close-time bookkeeping (not reconcile surfaces, carried for /close)

- IMP-390 closes; IMP-367 gets a note that `next_obligation`'s disposition was
  settled here; IMP-372 gets a note that an override must bring an admission
  bound (design sec-6).

## Reconciliation Outcome

### Direct edits applied
- Selector registry (RV-383 F-2): `slice selector add` `src/design_run/fixture.rs`,
  `src/design_run/prompt.rs` (design-target); `slice selector rm`
  `tests/e2e_design_show_golden.rs`, `tests/e2e_design_projection.rs` (covered by
  the `tests/e2e_design_*.rs` glob). `slice conformance` now reports 0
  undelivered and only the slice's own `notes.md`/`plan.toml` as undeclared.
- design.md (RV-383 F-2, F-3), out of band on the locked run, so the watermark
  divergence is expected: sec-1 names `gate_facts`/`GateFacts`; sec-2
  `Divergence` carries `refusal`; sec-2 step 6 describes `ApplyRequest::bare`;
  sec-4 and sec-8 old-snapshot fixture is absent key + string; sec-7 table gains
  `fixture.rs` and `prompt.rs` and notes the glob-covered tests.

### REVs completed
- REV-061 (`reconcile-sl-262`): done. REQ-414 (the obligation is derived, not
  held), REQ-437 (forward edge among the named limits; maximal-forward bounding
  case), SPEC-029 responsibilities (the forward edge is a projection of the gate
  contract table). Covers RV-383 F-4. Rationale in revision-061.md.

### Withdrawn / tolerated
- None. F-1 was fixed in audit (e8b15c18e); F-5 is aligned.

Reconcile pass complete; hand off to /close.
