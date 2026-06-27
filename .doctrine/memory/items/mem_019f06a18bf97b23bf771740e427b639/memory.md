# Dispatch close alt: pre-FF trunk so close_target absorbs the candidate-only repair

**Same trap as [[mem_019ee36939ca7a70b8aa960cb478d94c]]** (audit fix-now committed on
the candidate branch, outside the dispatch journal; `close_target` + `sync
--integrate` project from `review/<slice>`, which lacks it → naive close lands trunk
*without* the repair, and `slice status done` refuses: "dispatched but no trunk
row"). That memory's canonical fix folds the fix-now back into the journal
(cherry-pick onto `dispatch/<slice>`, `git branch -D review/<slice>`, re-prepare).

**Alternative fix (lighter — when trunk can fast-forward to the admitted candidate
tip, the common case for a clean linear repair).** Pre-load trunk with the
reconciled truth, then let the *standard* close_target merge absorb it:

1. `git branch -f main <admitted-candidate-tip>` — fast-forward trunk to the
   RV-admitted `review_surface` tip (e.g. `830cd857`, which already contains the
   fix-now). Verify `git merge-base --is-ancestor <old-main> main` (linear, no
   rewrite). Ref-only — `main` must be checked out nowhere (primary tree stays on
   `edge`); fully reversible.
2. `candidate create --role close_target --payload code --base refs/heads/main
   --source refs/heads/review/<slice>`. Because `base` (main) now already contains
   the repair and `source` (`review/<slice>`) is its **ancestor**, the no-ff 3-way
   merge is a **content no-op**: close_target tree == candidate tip tree. Verify
   identical trees + that it's a ff-descendant of main.
3. `candidate admit --role close_target --candidate refs/heads/candidate/<slice>/close-001
   --review RV-NNN`, then `sync --integrate --trunk refs/heads/main` → a ff no-op
   that just **records the journal trunk row** at the correct tip, satisfying the
   status seam. Then the ISS-030 tree-true checks
   ([[mem_019ec912f7fd746284bfaef00717443e]]): `--show-journal-trunk-oid` == `main`.

**Why this is consistent, not a hack.** The standard close_target is *always*
`merge(review/<slice>, current-trunk)` — its tip routinely differs from the
`review/<slice>` tip (cf. SL-098: close_target `ac93dc58` ≠ review `ff037f`). This
route just advances trunk to the repair *first* (via the sanctioned admitted
candidate = RV-declared reconciled truth) instead of advancing `review/<slice>`.
Repair lands on trunk; journal row is honest about the trunk tip; `review/<slice>`
stays the immutable bundle that was actually reviewed.

**Trade-off vs the canonical fold.** Leaves `review/<slice>` pointing at the
pre-repair bundle (the fold makes it carry the repair). Pick the fold when the
journal/evidence should literally show the repaired bundle; pick this when trunk-FF
is clean and you'd rather not delete+rebuild the reviewed ref.

**Root prevention is unchanged:** commit audit fix-now onto `dispatch/<slice>`, not
the candidate branch, so it flows through prepare-review natively. Observed SL-159
close, 2026-06-27.
