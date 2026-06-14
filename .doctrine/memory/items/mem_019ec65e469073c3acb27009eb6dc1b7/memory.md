# doctrine worktree import corrupts the patch under rtk; use the checkout-import substitute

**SL-067 dispatch (claude arm), both batches.** `doctrine worktree import --base B
--fork S` failed internally with `git apply --3way --index ... error: corrupt patch
at <stdin>:N` on BOTH phases — different N each time (627, then 556), and PHASE-02's
delta had no long lines, so the corruption is **content-independent**: it is the
verb's internal git diff→`git apply` pipe being mangled in the rtk-hooked
environment, NOT a bad patch. (Distinct symptom from
[[mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import]], which is the
stat-proxy `No valid patches in input` failure — same root cause family, the
`git apply` arm of the import.)

**Substitute (tested, both batches landed clean):** for the funnel's import step,
replace the verb with the checkout-import idiom after running the verb's belts BY
HAND on the trusted side:
- precond: coord `HEAD == B`, tree clean.
- `S^ == B` (immediate parent IS B); single non-merge commit (`git rev-list
  --parents -n1 S` → exactly 2 tokens).
- R-5: `git diff --name-only B..S` touches NO `.doctrine/`/`.claude/` path.
- then `git checkout S -- $(changed paths)` to stage S's exact blobs into the coord
  index (valid because the batch is disjoint and coord==B, so S's blob == the net
  diff). Verify fidelity: `git diff S -- <paths>` empty.

Then proceed normally: combined-tree verify → `branch-point-check` → one batch
commit → `record-boundary` → knowledge commit. The belts the `import` verb would
have enforced are exactly the four checked above, so nothing is lost.

Companion: the claude-arm `SubagentStart` stamp also **fails open** in this env (no
marker written for `Agent` `isolation:worktree` worktrees), so `verify-worker`
refuses `unstamped` every batch — proceed on a directly-proven `S^==B` (stronger
than its `--is-ancestor` base check) plus the manual R-5; the marker is a fail-open
proxy, the R-5 belt is the real protection. See [[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]].
