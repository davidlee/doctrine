# Claude dispatch arm wrong-base risk under shared-clone contention

The claude `/dispatch` arm (`/dispatch-agent`) places workers at base==B by
`cd`-ing into the coordination worktree before the `Agent` `isolation: worktree`
spawn (forks the Bash-cwd HEAD under `baseRef: "head"`). That placement is
**correct when `main` is static but unreliable under a busy shared clone**: many
concurrent agents contend on git's repo-global locks (`index.lock`, `HEAD.lock`,
`.git/worktrees`), the worker's worktree creation intermittently loses the race,
and the subagent **silently falls back to the main worktree** — where
`baseRef: "head"` then tracks a **moving `main`**. Worker lands on a wrong, dirty,
moving base instead of B. Observed failing 3× consecutively on SL-121 PHASE-02
while other agents advanced `main`; a worker even started on the correct fork and
was **clobbered to `main` mid-run** when `main` moved. Full root cause + evidence:
**ISS-034** (provenance). Refines [[mem_019ec65ecbc77282bad7e10a5240ad27]] and
[[mem_019ec6142d3b71008f2149a6d84ba981]] — placement controls base *until*
contention overrides it; see also [[mem_019edf7cbf877c329e0cb3ef5ee6469e]].

**Why:** the parallel-work model assumes `isolation: worktree` gives a *stable*
fork pinned to B for the worker's whole lifetime. Under real multi-agent load that
invariant does not hold; it held in earlier dispatches only because `main` happened
to sit still.

**How to apply:**
- Always carry a **base-guard** in the worker prompt: first action greps the
  prerequisite seams + asserts `git status` clean + checks `merge-base
  --is-ancestor B HEAD`, STOP-and-report on mismatch. This made every SL-121
  wrong-base spawn non-destructive (the arm fails *closed*). Make it standard, not
  ad hoc.
- The orchestrator must still run `doctrine worktree verify-worker --base B`
  before import — and treat a missing `worktreePath:` footer in the Agent return as
  a red flag (no isolated tree was created).
- When `main` is actively churning, **prefer the subprocess arm**
  (`/dispatch-subprocess`, deterministic `doctrine worktree fork --worker` + bound
  cwd) **or inline `/worktree` + `/execute`** off the coordination branch — not the
  claude `isolation: worktree` arm.
- Unrelated friction seen alongside this: 2 of 3 `SubagentStart` stamp hooks point
  at a `doctrine (deleted)` binary path → workers come up unstamped, `verify-worker`
  refuses until hand-stamped. (ISS-034 Defect B.)
