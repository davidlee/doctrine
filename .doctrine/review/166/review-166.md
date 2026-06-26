# Review RV-166 — reconciliation of SL-157

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation, self-audit). **Surface reviewed:**
solo `/execute` fork `sl-157-phase-01`, single delta `da243b3d` (edge baseline
`42c55624`). Not `/dispatch` — no candidate branch; the fork delta *is* the
evidence. Reviewed from the primary tree (`edge`); gate run in the fork worktree.

**What this probes** — does the delta strip *exactly* the None-leg speculative
post-CAS resync and nothing load-bearing, per design §3–§4 and the PHASE-01
EX/VT/VA criteria?

**Invariants held:**
- **Surgical scope** — only `resync_worktree_hard`, `Disposition::RacedDesync`,
  and the `advance_pure_ref` re-probe are removed; the checked-out leg
  (`advance_checked_out`/`ff_advance_in_worktree`), the M4 dirty pre-gate,
  `AdvancedResynced`, `report_integrate`'s body, and the ADR-012 D4 CAS contract
  stay untouched (EX-3/EX-4, VA-1).
- **Behaviour preservation** — the integrate-safety e2e suite is the proof and
  stays green *unchanged*; only the dead `resync_worktree_hard` unit test is
  removed (EN-2/VT-1, no new test per design §6).
- **Gate clean** — `cargo clippy` zero warnings proves no orphaned fn/variant
  after the removals (VT-2).
- **Governance truth** — D4 CAS contract preserved ⇒ no ADR-012 Revision; the
  one durable-gov touch is SPEC-022 prose (deferred to reconcile, design §5).

**Where bodies could hide:** an over-deletion taking the checked-out leg with it;
a stale doc-comment left naming the dead mechanism; SPEC-022 prose still
describing the resync; IMP-122 (open) hardening a now-deleted mechanism; the
source-delta registry never bound (conformance signal).

## Synthesis

**Closure story.** SL-157 PHASE-01 lands as a single atomic delta (`da243b3d`,
+13/−75) that strips the not-checked-out integrate leg's speculative post-CAS
re-probe/resync and the machinery only it reaches (`resync_worktree_hard` + its
unit test, `Disposition::RacedDesync` + its `label()` arm). Under Doctrine's
dispatch posture the delivery ref (`main`) is never checked out and `edge` is
always already checked out (the safe atomic FF leg), so the guarded None→Some
race the resync defended cannot occur — the guard *was* the sole locus of the
RFC-005 H2 / IMP-122 R1/R3/R4 hazard. Deleting the condition dissolves the
hazard at the mechanism (design's "delete the condition, don't harden the
window").

**Conformance.** Mechanical path-conformance is clean (2 conformant, 0
undeclared, 0 undelivered) after bootstrapping the registry row (F-1). Every
PHASE-01 criterion holds: EX-1..5 confirmed by diff inspection, VT-1 by the
38-test integrate e2e suite green (incl. the checked-out-FF and pure-ref
regressions), VT-2 by `cargo clippy` zero warnings (no orphaned fn/variant),
VA-1 by the surgical-scope review — checked-out leg, M4 dirty pre-gate,
`AdvancedResynced`, `report_integrate` body, and the ADR-012 D4 CAS contract all
untouched; no new test added (design §6: pinning the unsupported
main-checked-out race would re-pin the hazard).

**Standing risks / accepted tradeoffs.**
- **No ADR-012 Revision.** Every advance remains a 3-arg CAS; non-FF still
  refused; no force-push, no auto-resolve — D4 is preserved verbatim, so no
  ADR-012 decision changes. Consciously accepted per design §5.
- **Non-FF trunk auto-merge (RFC-006)** is explicitly out of scope — it would
  *reverse* FF-only and is routed to external review.
- **F-1 process gap** (solo-on-fork did not auto-bind the source-delta) is a
  known property of fork-based solo `/execute`, not a code defect; repaired in
  this audit and worth a durable note.

## Reconciliation Brief

### Per-slice (direct edit)
- **IMP-122** (F-3): close as resolved-by-deletion — the slice deletes the
  None-leg resync wholesale, mooting IMP-122's two hardenings; cite SL-157 in
  the closure note.

### Governance/spec (REV)
- **SPEC-022** (F-2): `modify` REV `--target SPEC-022` striking the advance-leg
  parenthetical *"(with a post-CAS re-probe that resyncs a newly-checked-out
  ref)"* — the not-checked-out advance is now pure CAS-and-done. Apply *after*
  the code lands so the spec never leads the code (design §5). No ADR-012
  Revision (D4 CAS contract unchanged).

### Out of scope (carry separately, not reconcile)
- **RFC-006** — non-FF trunk auto-merge + conflict surgery (the ADR-012 D2/D4
  reversal). External review gates any Revision.
- **R2** — `/close` ISS-030 recovery procedure (independent skill fix).
