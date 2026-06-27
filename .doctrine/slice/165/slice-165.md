# Close-projection path for audit fix-now repairs

## Context

`/audit` routinely commits a fix-now repair on the candidate branch
(`candidate/<slice>/review-001`), on top of the candidate merge — outside the
dispatch journal. The admitted `review_surface` OID correctly points at that
repaired tip, so review looks correct. But dispatch **close** projects from the
*journal* (`review/<slice>` / `dispatch/<slice>`), which never saw the fix-now:

- `dispatch candidate create --role close_target --source refs/heads/candidate/...`
  refuses — the provenance gate requires a journaled source ("no prepare-review
  journal row for source").
- `--source refs/heads/review/<slice>` is the *pre-repair* bundle → naive close
  lands trunk **without** the repair, silently re-opening the gap audit just caught.
- `slice status done` refuses without a journal trunk row, which only
  `sync --integrate` writes, which only sees journaled refs.

Catch-22: close machinery wants the repair in the journal; the repair is
candidate-only. The reconciled truth (the RV-admitted candidate tip) is *knowable*,
but no first-class path lands it. Two manual workarounds exist
(`mem_019ee369` fold-into-journal; `mem_019f06a1` pre-FF-trunk-absorb) — both
error-prone dances that can ship unreconciled code. Hit at SL-159 close (RV-172
F-1/F-2), 2026-06-27. From IMP-188.

## Scope & Objectives

Make the substrate **conform to SPEC-022 REQ-317** (D1), which already mandates the
path: a `close_target` sourced from the repaired candidate
(`--source refs/heads/candidate/<N>/<label>`). The implementation refuses it today —
`check_provenance` demands a journaled source. Design `/design` locked the fix (see
`design.md`):

- **Extend `check_provenance`** to accept a recorded `candidate/<N>/<label>` source
  for a `close_target` create, tracing the candidate chain (bounded recursion) to a
  Verified journaled-evidence root (**provenance model A**). The existing
  admit-by-OID + FF-only `integrate --trunk` machinery is unchanged and already
  lands the admitted (repaired) tip + writes an honest journal trunk row.

This collapses the original three-mechanism menu to **one surgical conformance fix**.
Auto-fold and the audit-time guard are now non-goals (below).

**Headline discovery:** SPEC-022 is internally contradictory — **REQ-316**
(journaled-only provenance) forbids the `--source` that **REQ-317** mandates. SL-165
reconciles both REQs via a **Revision (REV) at reconcile** (the slice's known
governance obligation).

In scope:
- `check_provenance` extension on the dispatch candidate surface (`src/dispatch.rs`).
- Honest journal trunk row at the repaired tip so `slice status done` passes
  natively (already provided by integrate once the close_target is admitted).
- Tests: gate accept/refuse matrix + full repair→close→integrate→`status done`
  lifecycle (regression anchor for the silent drop / refusal).
- Spec reconciliation REQ-316/317 — authored via REV at reconcile, not in code commits.

## Non-Goals

- **The operator drift *detector*** — owned by **IMP-130** (warn on
  `review_surface` candidate drift before `/close --source`, land in SPEC-021 /
  skills). SL-165 supplies the *landing path*; IMP-130 supplies the *guard*. Not
  re-implemented here.
- **Auto-fold at integrate** (original mechanism #2) — rejected as anti-doctrinal:
  it contradicts admit-by-OID's explicit-operator-choice philosophy (REQ-316).
- **Audit-time guard/route** (original mechanism #3) — IMP-130's mandate, not this
  slice.
- **Non-FF auto-merge** of a moved trunk (RFC-006 territory; reverses ADR-012
  D2/D4 FF-only). SL-165 preserves FF-only.
- The OQ-5 checkout-independent integrate rewrite (SL-157) — disjoint mechanism.
- Re-surveying RFC-005's hazard taxonomy. SL-165 may note its placement
  (close-projection hazard, H2-adjacent) but does not rewrite the RFC.

## Affected Surface (coarse — `/design` refines)

- `src/dispatch.rs` — `check_provenance` (+ new `trace_candidate_provenance`,
  `is_journaled_evidence_ref` predicate); `candidate_create` read-ordering.
- Tests: `tests/e2e_dispatch_candidate.rs`, `tests/e2e_dispatch_lifecycle.rs`.
- `.doctrine/spec/tech/022/**` — REQ-316/317 reconciliation, via REV at reconcile.

## Risks / Assumptions / Open Questions

Design decisions locked (`design.md` §7): model A · bounded recursion · close_target-
scoped · `status==Created` gate · spec via REV. The original scoping OQs resolved:

- **OQ-1 (governance altitude) → resolved.** Largely dissolved: the code is
  *conformance* to the controlling REQ-317. The residual REQ-316 narrow (a normative
  gate widening) routes through a **Revision at reconcile** (design D4 / Q3-A).
- **OQ-2 (IMP-130 boundary) → resolved.** SL-165 = path; IMP-130 = detector; disjoint.
  The audit-time guard is a non-goal here.
- **OQ-3 (`review/<slice>` immutability) → resolved.** Model A keeps the reviewed
  bundle immutable (no fold). Auto-fold rejected as anti-doctrinal.
- **OQ-4 (RFC-005 placement) → deferred.** Note as a close-projection hazard
  (H2-adjacent) at reconcile; do not rewrite the RFC here.

Carried design-level OQs (see `design.md` §6): depth-budget constant (cosmetic);
exact REQ-316 wording (REV authoring); hand-resolved-`Conflicted` source (v1: refuse,
OQ-4 there).

## Verification / Closure Intent

Done when: a slice carrying an audit fix-now on its candidate branch can be closed
through a single first-class command sequence that (a) lands the repair on trunk,
(b) records a journal trunk row at the repaired tip, (c) passes `slice status done`
natively, and (d) is covered by a test that fails under the old silent-drop
behaviour. No manual fold / pre-FF dance required.

## Follow-Ups
