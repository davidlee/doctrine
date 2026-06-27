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

Provide a **first-class path** that lands the RV-admitted reconciled truth (the
fix-now-bearing candidate tip) onto trunk + records an honest journal trunk row,
without a manual dance and without silently dropping the repair.

The candidate mechanisms (design picks — likely 1 or 2, with 3 as a complementary
guard):

1. **Source `close_target` from the admitted `review_surface` OID** — let
   `candidate create --role close_target` accept the RV-pinned admitted candidate
   tip directly, bypassing the prepare-review-row gate on the close axis. Most
   direct (the admitted OID *is* the reconciled truth); keeps `review/<slice>`
   immutable.
2. **Auto-fold at close** — `sync --integrate` (or a close pre-step) detects
   candidate-only commits ahead of journaled `review/<slice>` and folds them into
   the journal (mechanises `mem_019ee369`).
3. **Audit-time guard/route** (complementary, root-prevention) — warn/refuse when
   a fix-now commit lands on the candidate branch rather than `dispatch/<slice>`,
   steering the repair onto the coordination tip so it flows through
   prepare-review natively.

In scope:
- The chosen mechanism on the dispatch candidate / integrate surface.
- Honest journal trunk row at the correct (repaired) tip so `slice status done`
  passes natively.
- Tests proving close-after-fix-now lands the repair (regression for the silent
  drop).

## Non-Goals

- **The operator drift *detector*** — owned by **IMP-130** (warn on
  `review_surface` candidate drift before `/close --source`, land in SPEC-021 /
  skills). SL-165 supplies the *landing path*; IMP-130 supplies the *guard*. If
  mechanism #3 is taken, it coordinates with IMP-130 (one detector, not two) —
  but SL-165 does not re-implement the detector.
- **Non-FF auto-merge** of a moved trunk (RFC-006 territory; reverses ADR-012
  D2/D4 FF-only). SL-165 preserves FF-only.
- The OQ-5 checkout-independent integrate rewrite (SL-157) — disjoint mechanism.
- Re-surveying RFC-005's hazard taxonomy. SL-165 may note its placement
  (close-projection hazard, H2-adjacent) but does not rewrite the RFC.

## Affected Surface (coarse — `/design` refines)

- `dispatch candidate create` / `admit` provenance gate (SL-068 candidate layer).
- `dispatch sync --integrate` projection + journal trunk row write.
- Possibly `/close` + `/audit` SKILL.md (mechanism #3 routing).

## Risks / Assumptions / Open Questions

- **OQ-1 (governance altitude).** Mechanism #1 changes candidate provenance
  semantics (close_target may source a non-journaled admitted OID) → touches
  **SPEC-022 REQ-317** + SL-068; likely a **Revision**, not mechanism-only.
  Mechanism #2 brushes the journal-projection contract. `/design` must decide the
  altitude and whether a REV/RFC is required.
- **OQ-2 (IMP-130 boundary).** Confirm at design: SL-165 = path, IMP-130 =
  detector, disjoint. Mechanism #3 must not fork the detector.
- **OQ-3 (`review/<slice>` immutability).** Mechanism #1 keeps the reviewed bundle
  immutable; the fold (#2) rewrites it. Design states the chosen semantics
  explicitly.
- **OQ-4 (RFC-005 placement).** Standalone vs folded into RFC-005's survey as a
  named close-projection hazard.

## Verification / Closure Intent

Done when: a slice carrying an audit fix-now on its candidate branch can be closed
through a single first-class command sequence that (a) lands the repair on trunk,
(b) records a journal trunk row at the repaired tip, (c) passes `slice status done`
natively, and (d) is covered by a test that fails under the old silent-drop
behaviour. No manual fold / pre-FF dance required.

## Follow-Ups
