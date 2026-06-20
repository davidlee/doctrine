# Review RV-108 — reconciliation of SL-123

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Review surface (dispatched slice).** Reviewed the candidate interaction branch
`candidate/123/review-001` (created from the `review/123` impl-bundle merged onto
`main`), **not** the raw `review/*`/`phase/*` evidence refs — per the dispatched-slice
audit rule. Surface tip at audit start: `6f19ec1c`; after the audit fix-now (VT-4
tests), `b8c80e8c`.

**Lines of attack.**

1. **Conformance to plan exit criteria.** PHASE-01 (EX-1..5, VT-1..5) and PHASE-02
   (EX-1..4, VT-1, VA-1, VH-1) — does each delivered artifact meet its authored
   criterion? Particular focus on the *test* criteria (VT-4 integration coverage of
   the two new belts) and the read-class survival criterion (EX-4/VT-5).
2. **Design ↔ implementation fidelity (§5.2).** Does `classify_worker_verify` match
   the design's precond order (head→isolated→marker→base→branch), tokens, and the
   `head_is_branch_tip`-true-when-no-`--branch` skip? Does the shell gather reuse
   `is_linked_worktree` with no new git plumbing?
3. **Behaviour-preservation gate.** Existing `verify-worker` verdicts unchanged for
   all isolated cases (the signature ripple must not change a verdict).
4. **Honest-residual claims.** Are the §5.1/§5.2 residuals (mid-run clobber via the
   funnel import belt; properties-not-provenance) accurately scoped, not overclaimed?
5. **Closure obligations.** ISS-034 Defect A and the IMP-052 overlap — routed to the
   reconciliation brief, not silently dropped.

**Invariants held to:** the four primed invariants (behaviour-preservation, ADR-006
orchestrator-sole-writer, ADR-001 pure/imperative split, belt-set fails-closed).

## Synthesis

**Closure story.** SL-123 delivers what the locked design specified: the two
orchestrator-side belts (`not-isolated` ordered #2, `branch-mismatch` last) land in
`classify_worker_verify` with the exact precond order and tokens of design §5.2; the
impure `run_verify_worker` shell gathers isolation via the already-trusted
`is_linked_worktree` (no new git plumbing) and the `--branch` coherence read only when
`--branch` is supplied; the CLI surface (`--branch Option`, executor threading, the
five-token doc-comment) is complete and `VerifyWorker` stays in the Read arm
(EX-4/VT-5 satisfied by inspection). The PHASE-02 skill belt carries the four-check
BASE GUARD, the pre-funnel footer-abort + five-token verify cadence, and the Red Flags
— with the budget cap bumped to exactly 78 and content pinned by presence asserts
(`base-guard`/`not-isolated`/`branch-mismatch`/`worktreePath`). Behaviour-preservation
holds: every pre-existing `verify-worker` verdict is unchanged, proven by the updated
goldens and the green e2e suite.

**The one real gap (F-1), now closed.** PHASE-01 was flipped completed with its VT-4
exit criterion unmet — the design-promised `run_verify_worker` *integration* tests for
the two new belts were never written, leaving the impure shell wiring (the isolation
gather, the `--branch` rev-parse, the CLI plumbing) covered only by the pure
classifier. This is the gap an audit exists to catch: pure logic green is not the same
as the wired belt firing end-to-end. Fixed-now within audit scope (test-only): three
e2e cases added on the candidate (`b8c80e8c`), all seven green.

**Standing risks (consciously accepted).** The belt-set is fail-closed but
*scenario-specific*, not "each belt sufficient" — the design is honest about this and
the §5.1 manifestation→belt map is accurate. Two residuals remain explicitly out of
scope and correctly named: (a) the mid-run-clobber / orphan-fork case is contained by
the *existing harness-identical funnel import belt* (`classify_import` `S^==B`), not by
`verify-worker`; (b) the properties-not-provenance remainder — a coherent footer naming
a wholly-valid unrelated tree is indistinguishable from a correct result and therefore
harmless to import; provenance binding is deferred to IMP-072. Neither is overclaimed.
The dirtied-primary-checkout blast radius (a clobbered worker can disrupt a concurrent
human/agent in the main tree) is named, not solved — that is true pre-worker isolation
(IMP-072), outside this slice's accepted ADR-011 D6 "loud-and-late" class.

**Trivia.** F-4 (a redundant HEAD rev-parse) tolerated — negligible and faithful to the
design pseudocode.

**Human gate — SIGNED.** VH-1 (human sign-off that the base-guard wording is unambiguous
and the footer-gate cadence is followable) was the only verification not closable by the
auditor. Signed off PASS by the user (2026-06-20), recorded as F-5 — prose clear,
complete, followable; the budget bump did not compress the safety prose (R3). Two
distill-time orchestrator obligations (fill `<B>`/`<seams>`) noted as prompt-fill duties,
not prose defects.

## Reconciliation Brief

### Per-slice (direct edit)
- None. The single per-slice gap (F-1, missing VT-4 tests) was resolved fix-now on the
  candidate branch during audit; design ↔ implementation are otherwise faithful, so no
  design-text correction is owed.

### Governance/spec (backlog reconciliation)
- **ISS-034 Defect A** (F-2): the claude-arm fail-closed remedy is delivered and proven
  → reconcile the backlog item to resolved/promoted (delivered-by SL-123).
- **IMP-052** (F-3): the §5.4b pre-funnel footer-abort cadence partly delivers IMP-052's
  "abort an un-isolated/unstamped worker" intent → add a delivered-overlap note citing
  SL-123; **do NOT auto-close** (design §9).
- **IMP-072** (follow-up context): remains open with SL-123 as trigger context for true
  pre-worker isolation (OQ-2/§5.2 residual) — confirm it still carries that pointer.

### Candidate admission
- Admit `candidate/123/review-001` (tip `b8c80e8c`, includes the audit fix-now) against
  RV-108 as the reviewed surface.

## Reconciliation Outcome

### Direct edits applied
- None to per-slice artefacts. The single per-slice gap (F-1, missing VT-4 tests) was
  resolved fix-now on the candidate during audit (`b8c80e8c`); design ↔ implementation
  are otherwise faithful, so no design-text correction was owed.

### Backlog reconciliation (direct backlog writes — no REV; no governance/spec truth touched)
- **ISS-034** (Defect A; covers RV-108 F-2): transitioned `resolved · mitigated`.
  SL-123 delivered remedies #2 (base-guard template) + #3 (`verify-worker` belts +
  footer gate), proven by test — wrong/moving base now fails closed loudly. Resolution
  is **mitigated** not fixed: the isolation race persists by design; true pre-worker
  elimination is deferred to IMP-072. Rationale recorded in backlog-034.md.
- **IMP-052** (covers RV-108 F-3): left **open**; added a delivered-overlap note —
  SL-123 §5.4b pre-funnel footer gate partly delivers the abort-un-isolated-worker
  intent (claude rung, import-time), but the spawn-time / cross-arm orchestrator gate
  remains. NOT auto-closed (design §9). Note in backlog-052.md.
- **IMP-072** (follow-up context): left **open**; added an SL-123 trigger-context note —
  SL-123 hardened the post-run §8.4 `verify-worker` belt this item defers against;
  IMP-072 remains the deferred pre-run (`WorktreeCreate`) upgrade. Note in backlog-072.md.

### Withdrawn / tolerated
- RV-108 F-4: tolerated — redundant HEAD rev-parse; negligible and faithful to design
  pseudocode. Rationale in the finding disposition.
- RV-108 F-1, F-5: verified (F-1 fixed-now during audit; F-5 = VH-1 human sign-off, PASS).

Reconcile pass complete — handoff to /close.
