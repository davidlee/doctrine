# IMP-188: Audit fix-now on candidate branch is invisible to dispatch close projection

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`/audit` routinely commits a fix-now repair **on the candidate branch**
(`candidate/<slice>/review-001`), on top of the candidate merge — outside the
dispatch journal. The admitted `review_surface` OID points at that repaired tip, so
review looks correct. But dispatch **close** projects from the *journal*
(`review/<slice>` / `dispatch/<slice>`), which never saw the fix-now:

- `candidate create --role close_target --source refs/heads/candidate/...` refuses:
  "no prepare-review journal row for source" — close_target source must be a
  journaled ref.
- `--source refs/heads/review/<slice>` is the *pre-repair* bundle → naive close
  lands trunk **without** the repair, silently re-opening the gap audit just caught.
- `slice status done` refuses without a journal trunk row, which only
  `sync --integrate` writes, which only sees journaled refs.

Catch-22: the close machinery wants the repair in the journal; the repair is
candidate-only. Hit at SL-159 close (RV-172 F-1/F-2). Known trap, documented in
memory `mem_019ee369` (fold-into-journal workaround) and
`mem.pattern.dispatch.close-preff-trunk-absorbs-repair` (pre-FF-trunk workaround).
Both are manual dances; this should not need a workaround.

## Why it's a system problem

The reconciled truth (RV-admitted candidate tip) is **knowable**, but no first-class
path lands it. Operators improvise — re-prepare (rewrites the "immutable" reviewed
ref) or hand-FF the trunk (leaves `review/<slice>` stale). Both are error-prone and
easy to get wrong in a way that ships unreconciled code.

## Candidate fixes (pick during design)

1. **Source close_target from the admitted `review_surface` OID.** Let
   `candidate create --role close_target` accept the admitted candidate tip (the
   RV-pinned reconciled truth) directly, bypassing the prepare-review-row gate for
   the close axis. Most direct — the admitted OID *is* the truth.
2. **Auto-fold at close.** `sync --integrate` (or a close pre-step) detects
   candidate-only commits ahead of the journaled `review/<slice>` and folds them
   into the journal automatically (the canonical manual fix, mechanised).
3. **Guard + route at audit time.** Refuse / warn when a fix-now commit lands on the
   candidate branch instead of `dispatch/<slice>`; steer the repair onto the
   coordination tip so it flows through prepare-review natively (root prevention).

Likely **1 or 2** for ergonomics, **3** as a complementary guard.

## Related

- `mem_019ee369` — canonical fold-into-journal trap + fix.
- `mem.pattern.dispatch.close-preff-trunk-absorbs-repair` — pre-FF-trunk alternative.
- IMP-130 — adjacent: review_surface candidate drift guard before `/close`.
- SPEC-022 (git interaction model), ADR-012 (dispatch integration topology).
- Observed: SL-159 close, RV-172, 2026-06-27.
