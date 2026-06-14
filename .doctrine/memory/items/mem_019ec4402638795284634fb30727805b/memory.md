# git cat-file -e exit code can false-positive under the rtk hook; use ls-tree for existence

In this jail, the Claude Code rtk hook transparently rewrites many `git`
invocations (`git X` → `rtk git X`). In some invocation contexts the **inner
git exit code is masked** — the wrapper returns 0 even when the underlying git
command failed. Observed concretely: `git cat-file -e <rev>:<path> && echo A ||
echo B` printed the success branch for a **nonexistent** path (and likewise via
`rtk proxy git cat-file -e`), while a bare `git cat-file -e HEAD:bogus; echo $?`
on its own line correctly returned 128. The masking is context-dependent, not
universal — do not assume any `git … && … || …` one-liner reports the real
status.

Impact: dispatch-funnel guards that branch on a git exit code (existence,
`merge-base --is-ancestor`, `grep` in a piped chain) can silently take the wrong
branch — e.g. concluding a new file "already exists at base B" and suspecting a
parallel implementation (false alarm during SL-060 PHASE-02 import).

Rule of thumb for funnel guards here:
- Existence: `git ls-tree --name-only <rev> <dir>/` and match the **printed
  name**, not `cat-file -e`'s exit.
- Disjointness / collision: `git diff --name-only B..S | grep` on **output
  text** (reliable — the decision reads printed paths, not the exit code).
- When you genuinely need an exit code, capture `rc=$?` on the **next line**
  and test `$rc`, rather than chaining with `&&`/`||` on the same line.

Related: [[mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import]] (rtk
stat-proxies `git diff` bodies — same hook, different symptom).
