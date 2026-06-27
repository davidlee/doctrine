# Design SL-165: Close-projection path for audit fix-now repairs

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`/audit` commits a fix-now repair on the `review_surface` candidate branch
(`candidate/<N>/review-001`), above the candidate merge. SPEC-022 **REQ-317** (D1,
the RV-116 resolution) mandates the path to land such a repair on trunk: create a
`close_target` *sourced from the repaired candidate*
(`--source refs/heads/candidate/<N>/<label>`). But the implementation refuses
exactly that. SL-165 makes the substrate conform to REQ-317 and reconciles a
spec self-contradiction it exposed.

## 2. Current State

`candidate_create` (`src/dispatch.rs:878`) calls `check_provenance`
(`src/dispatch.rs:805`, sole call site `:913`) for **every** role. The gate finds a
journal row with `target_ref == source_ref` and `status == Verified`:

```
candidate create: no prepare-review journal row for source {source_ref} —
run `dispatch sync --prepare-review` first
```

A `candidate/<N>/<label>` ref never owns a journal row, so a close_target sourced
from a repaired candidate is refused. Downstream is already correct:

- `candidate admit` (`:1180`) pins `admitted_oid` = re-read candidate tip, gated by
  **I3** `is_ancestor(merge_oid, tip)` — so a *repaired* tip (fix-now above the
  merge) is admissible.
- `plan_candidate_trunk_row` (`:2008`) projects `admission.admitted_oid` **directly**,
  FF-only — it does **not** read `review/<N>`.
- `trunk_integration` (`src/ledger.rs:438`) requires a journal trunk row, written by
  `integrate --trunk`.

So the *only* missing link is the create-time provenance gate. The two manual
workarounds (`mem_019ee369` fold-into-journal; `mem_019f06a1` pre-FF-trunk-absorb)
exist solely to route around it; both are error-prone and can ship unreconciled
code.

**The spec contradiction (the headline).** REQ-316 "Source provenance" states
`candidate create` **refuses** any `--source` that is not a Verified journal row —
no candidate exception. REQ-317 **mandates** sourcing the close_target from a
`candidate/<N>/<label>`. REQ-316 forbids what REQ-317 requires; the code implements
REQ-316, making REQ-317 unsatisfiable. Both REQs carry the same false claim — "the
substrate and the code already agreed."

## 3. Forces & Constraints

- **REQ-317 is the controlling intent** — the candidate-source path is already the
  blessed contract; SL-165 is conformance, not a new capability.
- **REQ-316's invariant must survive** — "no candidate from unverified evidence."
  The widened gate must keep every accepted chain rooted at Verified journaled
  evidence (transitively).
- **Checkout-independence (REQ-318)** — provenance must be decided from the
  `dispatch/<N>` object db (`candidates.toml` + `journal.toml` tree-reads), never the
  working filesystem.
- **Admit-by-OID (REQ-316) / FF-only integrate (ADR-012 D2/D4)** — unchanged. SL-165
  touches only create-time provenance; no integrate semantics move.
- **Governance routing (ADR-013)** — the REQ-316 edit is a normative requirement
  change → a **Revision** at reconcile (slice decision Q3-A).
- **No parallel implementation** — extend the existing gate; reuse `Candidates` /
  `CandidateRow`.

## 4. Guiding Principles

- Minimal blast radius: one gate function, one new helper, one moved read.
- Transitive honesty over loosening: the provenance *root* is preserved, reached
  through recorded candidate hops, never abandoned.
- Conform the code to the controlling requirement; reconcile the spec's internal
  conflict deliberately, through the governed (REV) path.

## 5. Proposed Design

### 5.1 System Model

`check_provenance` gains the candidate role and ledger, and a recursive base/step
that walks the recorded-candidate chain to a journaled-evidence root.

```
check_provenance(journal, candidates, slice3, role, source_ref):
  if is_journaled_evidence_ref(source_ref):            # review/<N> | phase/<N>-NN
      → journaled gate (existing): row.target_ref==source_ref ∧ Verified (+ phase-hole)
  elif role == close_target ∧ is_candidate_ref(source_ref):
      → trace_candidate_provenance(source_ref, budget)
  else:
      → existing refusal (unchanged)

trace_candidate_provenance(ref, budget):
  budget > 0                       else bail "provenance chain too deep / cyclic"
  row = exactly_one(candidates.rows where target_ref == ref)   # F1: count, not first-match
                                   else bail "no recorded row" / "ambiguous candidate row for {ref}"
  row.status == Created            else bail "source candidate {ref} is {status:?}, not clean"
  next = row.source_ref
  if   is_journaled_evidence_ref(next): → FULL journaled gate (Verified + phase-hole)  # F3
  elif is_candidate_ref(next):          → trace_candidate_provenance(next, budget-1)
  else:                                 → bail "source candidate built from non-evidence {next}"
```

The journaled base-case routes through the **existing** `check_provenance` journaled
body — Verified-row check **and** the `phase/<N>-NN` earlier-failed-hole check — so a
candidate tracing to a phase ref inherits the full gate, not a weakened subset (F3).
Row match is **count-exact, fail-closed** on duplicates (F1), mirroring
`trunk_integration`'s discipline (`ledger.rs:464` — "count, never first-match").

### 5.2 Interfaces & Contracts

- `check_provenance(journal: &Journal, candidates: &Candidates, slice3: &str,
  role: CandidateRole, source_ref: &str) -> anyhow::Result<()>` — signature widened
  (role + ledger). Sole caller updated.
- `fn is_journaled_evidence_ref(source_ref, slice3) -> bool` — factored predicate:
  `review/<slice>` or `phase/<slice>-NN`. Single source of truth so base-case and
  recursion-step agree (no classifier drift).
- `fn trace_candidate_provenance(candidates, journal, slice3, ref, budget) ->
  anyhow::Result<()>` — new private helper.
- No CLI surface change: `candidate create --role close_target --source
  refs/heads/candidate/<N>/<label>` is the unchanged invocation that now succeeds.

### 5.3 Data, State & Ownership

- Reads only: `Candidates` (`candidates.toml`) + `Journal` (`journal.toml`), both
  tree-read from `dispatch/<N>` (REQ-318). No new persisted state.
- `candidate_create` sequencing: move the `read_candidates` read (currently
  `:922`) ahead of the provenance call (`:913`); pass the ledger in. Both are pure
  object-db reads — no behavior change on the journaled path.
- Ownership unchanged: orchestrator-classed create; the gate is a pure validator.

### 5.4 Lifecycle, Operations & Dynamics

End-to-end (the IMP-188 reproduction, now first-class):

1. dispatch slice → `prepare-review` (journaled `review/<N>`, Verified).
2. `candidate create --role review_surface --source review/<N> --worktree`; audit
   commits fix-now on its branch; `admit --role review_surface`.
3. `candidate create --role close_target --base refs/heads/main --source
   refs/heads/candidate/<N>/review-001` → **gate traces** review-001 → `review/<N>`
   (Verified) → accept; no-ff 3-way merge folds the repaired tip into `main`.
4. `admit --role close_target` (I3 holds) → `integrate --trunk refs/heads/main`
   projects the admitted OID FF-only. Repair on trunk; journal trunk row written.
5. `slice status done` passes natively (`trunk_integration` finds the row).

No fold, no `branch -D review/<N>`, no hand-FF.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (root preserved).** Every accepted chain terminates at a Verified
  journaled evidence ref. REQ-316's invariant holds transitively.
- **INV-2 (scope).** Only `role == close_target` unlocks the candidate exception;
  review_surface / scratch keep the journaled-only refusal.
- **INV-3 (status).** Only `CandidateStatus::Created` candidates qualify; a
  `Conflicted` (parked-at-base) candidate is refused. Fix-now commits do not mutate
  `status`, so a repaired review_surface stays `Created`. **Known v1 limitation (F2):**
  a `--worktree` candidate that hit a conflict and was *hand-resolved + committed*
  carries a valid tip but `status == Conflicted`, so it is refused — that hand-merge
  has weaker provenance than a clean candidate; v1 requires re-creating it clean. See
  OQ-4.
- **INV-4 (termination).** Bounded `budget` (constant, 16) + recorded-chain walk
  cannot loop the gate; over-budget / cycle → refuse.
- **INV-5 (count-exact, F1).** The `target_ref → row` resolution is fail-closed on
  duplicates (exactly-one), never first-match.
- **A-1 (lineage ≠ content review, F3).** The trace proves the source candidate's
  *lineage root* is Verified journaled evidence. It does **not** prove every commit on
  the source tip is reviewed — fix-now commits, and commits inherited through a traced
  `scratch`/`experiment` candidate, ride above the recorded `merge_oid` and are
  *untracked by provenance*. That content-trust is `admit`'s I3 (descends-from-merge)
  **plus** the governing RV review — the same split the existing fix-now model already
  relies on. Provenance gates lineage; review gates content.
- **Edge:** chain hop to a non-evidence, non-candidate ref → refuse. Missing row →
  refuse. Superseded candidate row still resolvable by `target_ref`; status gate
  decides admissibility.

## 6. Open Questions & Unknowns

- **OQ-1.** Depth budget constant — proposed 16; cosmetic, settle at plan.
- **OQ-2.** Exact REQ-316 narrowed wording — settled at REV authoring (reconcile).
- **OQ-3.** Does REQ-317's process-owner note (SPEC-021) need a companion tweak?
  Flag for reconcile; likely no (process guard is IMP-130's mandate).
- **OQ-4 (F2).** Should a hand-resolved `Conflicted` candidate ever qualify as a
  close_target source? v1: no (re-create clean). Revisit only if it bites in practice.

## 7. Decisions, Rationale & Alternatives

- **D1 — Provenance model A (recorded-candidate).** Source must be a recorded
  candidate tracing to Verified evidence. Matches REQ-317's wording ("the repaired
  candidate") and keeps the journaled root. *Alt rejected:* (B) admitted-review_surface
  only — over-constrains operator sequencing; (C) any-descendant-of-evidence —
  abandons the recorded-provenance root.
- **D2 — Bounded recursion (ii).** "Traces to verified evidence" means the trace
  actually terminates there, however many candidate hops intervene. *Alt rejected:*
  single-hop (under-powered once candidates chain); trust-merge-lineage (a malformed
  orphan row passes).
- **D3 — close_target-scoped exception.** Matches REQ-317; minimal blast radius.
- **D4 — Spec edit via REV at reconcile (Q3-A).** REQ-316 is a normative FR; widening
  its gate is governance → Revision + external review, not a quiet direct edit.
  Implement the conforming code now (REQ-317 is the controlling intent); author the
  REV at reconcile. *Alt rejected:* (B) reconcile-direct — skips scrutiny of a
  normative-gate widening; (C) consult-now — the resolution content is clear, only
  routing was open and is now decided.

**Non-goals.** Auto-fold at integrate (anti-doctrinal — contradicts admit-by-OID's
explicit-operator-choice philosophy); audit-time guard/detector (IMP-130's mandate);
non-FF auto-merge (RFC-006); OQ-5 checkout-independent integrate (SL-157).

## 8. Risks & Mitigations

- **R1 — Widened gate admits unverified content.** *Mitigation:* INV-1 — chain must
  root at Verified evidence; tested refuse-matrix; REV external review confirms.
- **R2 — Status gate misjudges a usable candidate.** *Mitigation:* `Created` is the
  clean-merge status; fix-now does not change it; explicit Conflicted-refusal test.
- **R3 — Spec edit drifts from code.** *Mitigation:* single REV authored at reconcile
  against the landed gate; conformance link SL-165 → REQ-316/317.

## 9. Quality Engineering & Validation

- **Accept:** close_target from a recorded candidate tracing `review/<N>` (Verified) →
  create succeeds; merge tree carries the fix-now.
- **Refuse:** no row · `Conflicted` candidate · non-evidence chain hop · over-depth /
  cyclic.
- **Regression:** close_target from `review/<N>` (journaled) still works; review_surface
  default works; phase-hole refusal intact.
- **Lifecycle (anchor):** full repair→close→integrate→`status done`; the test fails
  under today's refusal, passes after the gate.
- Test homes: `tests/e2e_dispatch_candidate.rs` (gate matrix),
  `tests/e2e_dispatch_lifecycle.rs` (end-to-end).
- `just gate` green; clippy zero-warning.

## 10. Review Notes

**Internal adversarial pass (self), integrated:**

- **F1 — ambiguous row match.** First-match `find` on `target_ref` is unsafe under
  supersession/label reuse. → INV-5: count-exact, fail-closed (mirrors
  `trunk_integration`, `ledger.rs:464`). Integrated §5.1, §5.5.
- **F2 — `status==Created` over-refuses** a hand-resolved `Conflicted` candidate.
  → Kept Created-only for v1 (weaker hand-merge provenance), named as a limitation
  (INV-3) + OQ-4. Not a silent gap.
- **F3 — lineage ≠ content review.** Trace proves the lineage root only; tip commits
  (fix-now, traced scratch/experiment) are content-trusted downstream by admit-I3 + RV.
  → A-1 added; journaled base-case clarified to run the FULL existing gate
  (Verified + phase-hole), not a subset.

**Verified against source (not the map agent):** `check_provenance` gate
(`dispatch.rs:805/913`), `plan_candidate_trunk_row` direct admitted-OID projection
(`:2008`), `candidate_admit` I3 pin (`:1180`), `trunk_integration` done-gate
(`ledger.rs:438`). SPEC-022 REQ-316 ⊥ REQ-317 contradiction confirmed verbatim.

**Residual for external pass:** confirm the narrowed REQ-316 gate preserves "no
candidate from unverified evidence" (INV-1) and that A-1's lineage/content split is the
intended trust boundary.
