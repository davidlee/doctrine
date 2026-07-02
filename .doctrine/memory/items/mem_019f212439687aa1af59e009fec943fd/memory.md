# Un-armed dispatch-worker Agent runs in the coord tree (native isolation), not an isolated fork

The doctrine `WorktreeCreate` hook (`create-fork`) discriminates a dispatch worker
**positionally** — a spawn is a dispatch worker iff the Agent payload cwd **is** the
arming dir `<coord>/.doctrine/state/dispatch/spawn/` (set up by `dispatch arm-spawn`
+ `cd`). Spawn a `dispatch-worker` Agent with `isolation: worktree` from anywhere
else (e.g. straight from the coord root, skipping the arm-spawn ritual) and the
hook does **not** classify it: you get a **native** `isolation:worktree`, which
forks off the Bash-tool cwd HEAD and — per the native path — runs **in the
coordination tree itself**, not an isolated fork. No `create-fork` stamp, no footer
`worktreePath`.

## What happened (SL-185 PHASE-02 & PHASE-03, 2026-07-02)

Both phases were driven by spawning `dispatch-worker` directly from the coord tree
(no `arm-spawn`, no `cd` into the spawn dir). Result each time: no `agent-<id>`
worktree in `git worktree list`, no footer, and the delta sitting as **clean
untracked/modified files in the coord tree** at HEAD==B. Because the worker prompt
forbade `git commit` (and native `isolation:worktree` only collapses a commit the
worker *made*), the delta stayed in the working tree rather than landing on the
branch — the importable-equivalent of a completed `import`.

**Contrast** [[mem_019f1cafd8737421a1d7f2c68e56fe44]] (armed Fork path: hook fires
→ isolated tree persists + footer `worktreePath`) and
[[mem_019ec4a71f0f7592bc07d9f5dad8efdb]] (native path: a worker that *commits*
lands it on the parent branch). This is the same native mechanism, with a
non-committing prompt, so the delta is a working-tree change instead.

## How to apply

- If you deliberately skip arm-spawn (pragmatic on a `claude-force-subprocess`
  project where you want opus authoring without the full hook ritual), expect the
  delta **in the coord tree**, not a separate worktree. The `/dispatch-agent`
  "ABORT if no `worktreePath` footer" rule assumes you *armed*; when you didn't,
  it's not an abort — it's the native path. **Discharge `verify-worker`'s belts by
  hand** (`git status` scope-clean, HEAD==B, no `.doctrine/`/`.claude/` touch) then
  commit in place; there is no tree to `verify-worker` or `--force`-reap.
- If you want the isolated-fork guarantees (auto scope belt, footer, reap), do the
  full ritual: `dispatch arm-spawn --base B` → `cd` into the spawn dir → spawn →
  `cd` back. See [[mem_019ec6142d3b71008f2149a6d84ba981]] (placement controls base)
  and [[mem_019f1a5ce1f472219da91d0724bb766b]] (teardown is conditional on the hook).

**Relates to RFC-005** (dispatch funnel integrity — hazard survey): the un-armed
path silently drops the isolation the funnel's Claude arm advertises — the
`.doctrine`/`.claude` belt and combined-tree verify run POST-hoc on the coord tree,
not pre-commit on an isolated fork. Fine when the orchestrator hand-verifies scope,
but it is a real narrowing of the funnel's guarantees worth surfacing in the survey.
