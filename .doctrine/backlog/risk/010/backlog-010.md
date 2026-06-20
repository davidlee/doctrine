# RSK-010: Dispatch base-staleness surfaces as a candidate-time merge conflict on long drives

Surfaced during SL-122 audit (RV-110).

**The risk.** A `/dispatch` coordination branch is forked off a base captured at
setup. On a long drive (SL-122 ran across 5 phases over many hours), `main`
advances underneath it — here a `chore: fmt` commit (`05743bb7`) reformatted a
multi-line tuple in `src/relation.rs`, and SL-125 landed. The drift is invisible
until **candidate-create time**, when `dispatch candidate create` 3-way-merges the
impl bundle onto current `main` and conflicts on a merge-*base* divergence (both
sides rewrote the same multi-line block). A fmt-only fix on the dispatch branch
**cannot** resolve it — the merge-base itself must advance.

**Resolution that worked (SL-122).** Merge current `main` into `dispatch/<slice>`
(advancing the base past the drift), re-run `prepare-review` to regenerate the
bundle, then `candidate create` merges cleanly and `admit` accepts it. Cost: one
merge commit on the code branch + a re-sync.

**Why it's a risk, not just a one-off.** Related to ISS-036 (setup forks off stale
`origin/HEAD`). The longer a dispatch runs, the more likely the base goes stale and
the conflict only bites at the very end (post-audit-prep), where it is most
disruptive. Candidates: (a) `prepare-review` / `candidate create` could detect a
stale base and prompt a rebase up-front; (b) periodic base-refresh during long
drives; (c) clearer diagnostics naming the merge-base divergence.
