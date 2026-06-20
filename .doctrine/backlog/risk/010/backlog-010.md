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

**Second manifestation — setup HARD-ABORTS, not just candidate-time conflict
(SL-125 drive, 2026-06-20).** The ISS-036 stale-`origin/HEAD` base bites at
`dispatch setup` itself when origin lags far enough to predate the slice's *own*
authored files. Here local `main` was 36 commits ahead of `origin/HEAD`, which did
not contain `.doctrine/slice/125/plan.toml`. The trunk ladder
(`git.rs` `trunk_ladder`: `DOCTRINE_TRUNK_REF` → `origin/HEAD` → `main` → `master`,
`worktree.rs` `coordinate` ~1716) resolved to `origin/HEAD`, forked the coordination
worktree there, then `slice::run_phases` failed regenerating phase sheets:
`Plan for slice 125 not found at .dispatch/SL-125/.doctrine/slice/125/plan.toml`
→ `coordinate failed after add; rolled back cleanly`. A plain `git worktree add HEAD`
*does* carry the file, confirming it is base-selection, not checkout.

**Workaround that worked.** `DOCTRINE_TRUNK_REF=main doctrine dispatch setup …`
(and the same env on every later `sync` against trunk). Local `main` is the de-facto
trunk in this repo (commit-on-main, origin unpushed), so the ladder's first rung
should point there. Env does not persist across this harness's shell calls — prefix
each orchestrator dispatch command.

**Extra sharp edge observed.** `dispatch setup`'s rollback reverted *uncommitted*
changes in the **session main working tree** (in-flight WIP for the very phase being
dispatched). Recoverable only because the WIP had been saved to a patch first.
Setup should not touch the session working tree on rollback. Candidate (d): default
the ladder to `main` when `origin/HEAD` lacks `HEAD`'s tree (or warn loudly); (e)
setup rollback must never mutate the session working tree.
