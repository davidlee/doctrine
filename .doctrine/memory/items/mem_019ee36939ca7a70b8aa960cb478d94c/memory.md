# Dispatch close: fold audit fix-now into the journal before close_target projection

**Trap.** `/audit` often adds a fix-now commit *on the candidate branch*
(`candidate/<slice>/review-001`), on top of the candidate merge — outside the
dispatch journal. The admitted **review_surface** OID correctly points at that
fix-now tip, so review looks fine. But `close_target` creation and
`sync --integrate` both project from the **journal** (`review/<slice>` /
`dispatch/<slice>`), which does NOT contain the fix-now. Integrating as-is lands the
slice on trunk *without* the fix-now — silently reintroducing the very gap the audit
caught.

**Tell.** `candidate create --role close_target --source refs/heads/candidate/...`
fails: "no prepare-review journal row for source" — close_target source must be a
journaled ref (`review/<slice>`), not the candidate branch.

**Fix (fold into journal, re-prepare).**
1. Cherry-pick the fix-now commit onto `dispatch/<slice>` in its coordination
   worktree (`.dispatch/SL-NNN`). (Restore any stray `journal.toml` deletion first —
   see [[mem_019ec88a6b43746395017bd7869e38e9]].)
2. `dispatch sync --slice N --prepare-review`. It refuses to clobber the stale
   `review/<slice>` — `git branch -D review/<slice>` first (its commits stay
   reachable from the candidate branch; the admitted OID is pinned in
   `candidates.toml`), then re-run. Confirm the new `review/<slice>` carries the
   fix-now.
3. `candidate create --role close_target --payload code --base refs/heads/main
   --source refs/heads/review/<slice>` → `candidate admit --role close_target
   --review RV-NNN`.
4. `sync --integrate --trunk refs/heads/main`, then the ISS-030 tree-true checks
   ([[mem_019ec912f7fd746284bfaef00717443e]]).

**Root prevention:** during audit, commit fix-now onto `dispatch/<slice>` (the
coordination tip), not the candidate branch, so it flows through prepare-review
natively. Observed SL-123 close, 2026-06-20.

