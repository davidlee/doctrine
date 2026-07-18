# Stranded audit fix-now surfaces as slice conformance undelivered; cherry-pick onto trunk at close

**The trap** (sibling of [[mem.pattern.dispatch.close-preff-trunk-absorbs-repair]]).
An `/audit` fix-now commit made on the **review candidate branch**
(`candidate/NNN/review-001`, on top of `review/NNN`) — not folded back into
`review/NNN` or the dispatch journal — is invisible to the standard close. The
`close_target` candidate sources `refs/heads/review/NNN`, so `dispatch sync
--integrate --trunk main` lands the bundle **without** the repair. Unlike the
"dispatched but no trunk row" refusal, this one **does** produce a valid trunk row
and `slice status done` succeeds — so the repair silently never reaches trunk.

**The detection signal.** If the fix-now touched a file that is a declared
selector, `slice conformance SL-NNN` (run post-integration on the landed tree)
reports it under **`undelivered`** — declared but absent from the recorded phase
deltas (the repair is outside every `boundaries.toml` range). That's the tell: a
declared code file reading undelivered at close = a change that never landed.
Confirm with `git merge-base --is-ancestor <fixnow-oid> main` (NO) and grep the
landed file for the repair's symbol.

**The fix (lighter than pre-FF / journal-fold).** The repair is usually a small,
disjoint commit (e.g. a test added to `main.rs`). After the reconcile artefacts are
committed on the `main` worktree, `git cherry-pick <fixnow-oid>` onto trunk. The
journal trunk row (the admitted `close_target`) becomes an **ancestor** of the new
`main` tip, which the `slice status done` closure seam accepts (ancestry, not
equality — verified SL-222). Record it in the RV `## Reconciliation Outcome`.

**Root prevention (unchanged):** commit audit fix-now onto `dispatch/<slice>` (or
fold into `review/<slice>`), never onto the candidate branch, so it flows through
`prepare-review` natively. See
[[mem.pattern.dispatch.close-preff-trunk-absorbs-repair]] and
[[mem.pattern.dispatch.close-split-lineage-reconcile-on-edge]]. First observed:
SL-222 close, 2026-07-18 (the audit had noted the repair was committed on the
candidate; the surface split hid it until conformance flagged main.rs).
