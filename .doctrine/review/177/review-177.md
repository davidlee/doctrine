# Review RV-177 — reconciliation of SL-165

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface.** Dispatched slice; worktree gc'd. Phase commits landed on
journal refs, folded into the impl bundle `refs/heads/review/165`
(tip `d7de2605`, single squashed commit on `refs/heads/main`, touching exactly
the 3 declared files). No candidate interaction branch was ever created
(`dispatch candidate status --slice 165` → "candidates (none recorded)"). Audit
ran against the bundle, built in a detached worktree at `review/165` with the
gitignored `web/map/dist/` embed copied in.

**Lines of attack.**
1. Conformance algebra — `slice conformance 165`: clean (3/3 conformant, 0
   undeclared, 0 undelivered). Path-scope holds.
2. Gate logic vs design §5.1/§5.5 invariants — INV-1 (root preserved), INV-2/D3
   (role/kind scope), INV-3 (Created-only), INV-4 (bounded budget), INV-5
   (count-exact), INV-6/D5 (lineage binding). Read in source; all present and
   faithful.
3. Verification completeness — landed tests vs plan PHASE-02 VT-1 (the refuse
   matrix + EX-3 moved-ref) and PHASE-03 lifecycle anchor.
4. Governance obligation — the known SPEC-022 REQ-316 ⊥ REQ-317 contradiction
   (design D4), routed to a Revision at reconcile.

**Evidence.** Both e2e suites green in-worktree (candidate 27, lifecycle 3);
`cargo clippy` zero warnings; build clean once the embed is present.

## Synthesis

**Closure story.** SL-165 conforms `check_provenance` to SPEC-022 REQ-317: a
`close_target` may now be sourced from a recorded `candidate/<N>/<label>` whose
chain traces to a Verified journaled-evidence root. The landed gate
(`src/dispatch.rs`) is a faithful, minimal-blast-radius realisation of the locked
design — every §5.5 invariant is present and correct on read:

- **INV-1 root preserved** — the recursion terminates only at a Verified
  `review/<N>` | `phase/<N>-NN` row, routed through the *full* existing journaled
  gate (Verified + phase-hole, F3), not a weakened subset.
- **INV-2/D3 scope** — destination gated to `close_target`; source gated to
  `{review_surface, close_target}` ∧ `kind == audit`. `scratch`/`experiment`
  refused.
- **INV-3** Created-only; **INV-4** named budget `= 16`; **INV-5** count-exact
  `target_ref → row` (fail-closed on duplicates, mirrors `ledger.rs:464`).
- **INV-6/D5 lineage binding** — `is_ancestor(row.merge_oid, source_oid)` in
  `candidate_create` post-resolve binds *content* (not the ref name) to the
  verified-traced merge; the source-side analog of admit's I3, closing the RV-175
  F-1 blocker. The helper returns the matched row so the binding needs no second
  lookup — clean.

Conformance algebra is clean (3/3, 0 undeclared/undelivered). The PHASE-03
lifecycle anchor exercises the whole repair→close→integrate→`status done` path
and asserts the fix-now lands on trunk with an honest journal trunk row and no
manual fold — the IMP-188 reproduction, now first-class.

**The one real gap (F-1), now repaired in audit.** The landed test matrix covered
only 3 of the plan's mandated refuse cases (accept · no-row · scratch-role),
leaving the security teeth — INV-6 moved-ref (the codex blocker), INV-4
over-budget/cyclic, INV-3 Conflicted, INV-5 ambiguous, INV-2 experiment-kind, and
the non-evidence chain hop — present in code but unverified against regression.
Audit fix-now closed this: 12 pure unit tests over `trace_candidate_provenance`
(the structural refuse branches, impractical to hand-craft through the CLI) plus
one e2e moved-ref refusal exercising the git lineage binding end-to-end. All
green; clippy clean; the bin gate is test-only-additive (no src logic touched).

**Standing risks / tradeoffs consciously accepted.**
- **OQ-4 / INV-3 (F2):** a hand-resolved `Conflicted` candidate carries a valid
  tip but is refused (weaker hand-merge provenance). Documented v1 limitation, not
  a silent gap. Tolerated for v1.
- **A-1 (F3):** provenance gates *lineage*, not *content* — the fix-now commits
  layered above the merge are content-trusted by admit's I3 + the governing RV.
  With F-2's narrowing the source is always an audited `review_surface`, so this is
  reviewed audit content, never arbitrary scratch/experiment work.
- **Build embed:** the fresh-worktree build needs the gitignored `web/map/dist/`
  (RustEmbed `Assets::get`) copied in — environmental, identical on edge, not a
  SL-165 defect. Noted so the next auditor doesn't misread the compile error.

The spec self-contradiction (F-2) is the one item audit cannot resolve in-scope —
it routes to reconcile as a Revision (below).

## Reconciliation Brief

### Per-slice (direct edit)
- None. `design.md` matches the landed implementation (every invariant traced to
  source). `plan.toml` PHASE-02 VT-1 is now satisfied by the audit fix-now (the
  refuse matrix + moved-ref it mandated). No prose drift to reconcile.

### Governance/spec (REV)
- **SPEC-022 REQ-316 (F-2) → REV modify.** REQ-316 ("Source provenance") forbids
  any non-journaled `--source`; REQ-317 mandates sourcing a `close_target` from
  `refs/heads/candidate/<N>/<label>`. The landed gate conforms to REQ-317 and
  widens REQ-316. Author a Revision narrowing REQ-316 to admit the traced
  candidate-source exception (destination `close_target`; source an `audit`
  `review_surface` or chained `close_target`; chain roots at Verified evidence;
  INV-6 lineage binding). Settle OQ-2 (exact wording) at REV authoring. External
  review per ADR-013 / design D4/Q3-A.
- **SPEC-022 REQ-317 (F-2) → REV status/conformance.** Confirm REQ-317 is now
  satisfied by the substrate; conformance-link SL-165 → REQ-316/REQ-317.
- **OQ-3 (companion check)** — REQ-317's SPEC-021 process-owner note: assess at
  reconcile, likely no change (the audit-time guard is IMP-130's mandate, a
  non-goal here).
- **RFC-005 placement (slice OQ-4, deferred)** — note the close-projection hazard
  (H2-adjacent) at reconcile; do **not** rewrite the RFC here.

## Reconciliation Outcome

### Direct edits applied
- None. Per the brief, `design.md` matched the landed gate (every invariant traced
  to source) and `plan.toml` PHASE-02 VT-1 was satisfied by the audit fix-now. No
  per-slice prose drift to reconcile.

### REVs completed
- **REV-014** (`reconcile-sl-165`): `done` — covers RV-177 **F-2** (the SPEC-022
  REQ-316 ⊥ REQ-317 contradiction). One `modify REQ-316` row, applied and
  manually landed: the **Source provenance** clause is narrowed to admit the traced
  candidate-source `close_target` exception the landed gate implements (settles
  design OQ-2 wording). Faithful to `trace_candidate_provenance` /
  `check_provenance` on candidate tip `e7d2ebaa` — `kind=audit` ∧ `role ∈
  {review_surface, close_target}` per hop, `status=Created` (INV-3), count-exact row
  match (INV-5), bounded recursion depth ≤ 16 (INV-4), Verified journaled-root via
  the full gate incl. phase-hole (INV-1), `merge_oid` lineage binding (INV-6).
  External adversarial review per ADR-013 / design D4/Q3-A: codex (GPT-5.5) —
  REQUEST-CHANGES on pass 1 (4 under-described teeth), APPROVE on pass 2 after the
  wording was corrected against source. Rationale + before/after in revision-014.md.
- **REQ-317**: already `active`; now satisfied by the substrate (no status row
  needed). Conformance recorded: `coverage record` SL-165 → REQ-316 + REQ-317 (VA),
  per design R3. Slice conformance algebra clean (3/3).

### Assessed — no change
- **OQ-3** (REQ-317 SPEC-021 process-owner note): no companion tweak. REQ-317
  already scopes the operator obligation to SPEC-021 and fixes only the substrate
  fact; SL-165 implemented exactly that path. The audit-time guard is IMP-130's
  mandate, a non-goal here.
- **RFC-005** (slice OQ-4, deferred): close-projection of a candidate-sourced repair
  noted as an H2-adjacent hazard for the RFC author. RFC **not** edited in this pass.

### Withdrawn / tolerated
- **F-1**: `verified` — remediated in audit (fix-now: 12 unit + 1 e2e moved-ref;
  PHASE-02 VT-1 satisfied). No reconcile write.
- **F-2 standing tolerations** (carried from §Synthesis): OQ-4/INV-3 hand-resolved
  `Conflicted` refusal and A-1 content-vs-lineage trust — both documented v1
  limitations, named in the narrowed REQ-316 prose, not silent gaps.

Reconcile pass complete — handoff to `/close`.
