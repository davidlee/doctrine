# candidate admit accepts a fix-on-top tip — the DRIFT flag advises supersede, but do not

Stacking a fix-now commit directly on top of a recorded candidate merge is a
**first-class flow**, not a corruption. `candidate admit`'s I3 ancestor-check
(`src/dispatch.rs:2099`) accepts a tip that is a *descendant* of the recorded
merge, so the admission pins the fix-inclusive OID.

The trap: `candidate status` renders that row as **DRIFT** (live tip ≠ recorded
merge OID) and the `next:` hints suggest publishing a superseding candidate.
**Do not supersede.** Superseding re-runs the 3-way merge from `base` + `source`
and produces a tree *without* the on-top fix commit — silently dropping the very
work you fixed.

**How to apply.**

1. Commit the fix on the candidate branch; the branch tip moves past the merge.
2. `candidate admit --candidate refs/heads/candidate/<N>/<label> --review RV-NNN`
   — admits at the live tip. DRIFT is expected and benign once admitted.
3. Before relying on it, assert the fix is genuinely carried:
   `git merge-base --is-ancestor <fix-oid> <candidate-ref>`.
4. At close, chain the `close_target` from **that candidate**
   (`--source refs/heads/candidate/<N>/fix-001`), never from `review/<N>` —
   `review/<N>` predates the fix, and SPEC-022 forbids a review surface as a
   trunk payload anyway.

Observed at SL-227 (`cand-227-fix-001` → `cand-227-close-001`).

Related: [[mem.pattern.review.verified-is-terminal-amend-in-prose]],
[[mem.pattern.dispatch.integrate-needs-close-target-first]].
