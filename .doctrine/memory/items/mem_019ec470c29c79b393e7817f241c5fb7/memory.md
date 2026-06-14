# glob git add on .doctrine entity dirs sweeps foreign untracked files on shared main; stage exact paths

During an inline orchestrator write on a SHARED `main` worktree with live
concurrent agents, a glob `git add` over an authored-entity directory pattern
silently stages **other agents' untracked files** that match the glob.

Concretely (SL-060 PHASE-05 backfill): `git add .doctrine/slice/*/slice-*.toml`
to stage ~60 backfilled slice TOMLs also picked up a concurrent agent's
**untracked** `slice-063/slice-063.toml` (their in-flight SL-063 scaffold), which
landed in the backfill commit as a `create mode` entry. Commit-without-`-a` does
NOT protect against this — the glob explicitly stages the untracked match.

Caught it via the `create mode 100644 …063…` line in the commit summary; fixed
with `git rm --cached <file>` + `git commit --amend --no-edit`, which restored the
file to untracked so the owning agent could commit it themselves.

Rules on shared main (inline doctrine-mediated writes):
- Stage the **exact known file set**, not a glob — or after a glob-add, run
  `git diff --cached --diff-filter=A --name-only` (output content, not exit code —
  rtk masks exit) and unstage any path you did not author this turn.
- A `create mode` / new-file (`A`) entry in your commit you didn't intend = a
  swept foreign untracked file. Investigate before it lands.
- Exclude files carrying foreign uncommitted WIP entirely (re-stage only your
  delta), and re-anchor to the moved HEAD each step.

Related: [[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]]
(commit-without-`-a` protects against working-tree dirt, but not a glob-add of
untracked), [[mem.system.coordination.concurrent-design-shared-main-worktree]].
