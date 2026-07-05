# Workflow agent(isolation:worktree) mints the confined fork AT the armed base (positional arming), not cwd HEAD — SQ3 demonstrated

A dynamic-Workflow `agent(isolation:worktree)`, Bash cwd parked at the coord
tree's arming dir (`.doctrine/state/dispatch/spawn`) with `dispatch arm-spawn
--base <sha>` armed, mints the confined fork **at the armed `base` file**, not at
cwd HEAD. This is the SQ3 demonstration CHR-039 explicitly deferred ("actual
confined-fork-at-base not yet demonstrated").

**Evidence (SL-206 PHASE-01, 2026-07-06, this harness — Claude Code / Opus 4.8):**
decisive disambiguation run set the armed base to coord **HEAD's parent** so armed
base ≠ cwd HEAD:

- coord HEAD (cwd) = `1fadff60`; armed base = `7fe7e231` (HEAD^)
- fork `git rev-parse HEAD` = **`7fe7e231`** (== armed base, **≠ cwd HEAD**)
- fork `--git-common-dir` = shared primary `.git` (confined linked worktree)
- fork under `.dispatch/SL-207/.worktrees/wf_<runid>-1`, branch
  `dispatch/wf_<runid>-1`, reflog "Created from 7fe7e231"

Mechanism: `src/worktree/create.rs::classify_create` POSITIONAL trigger
(`cwd_is_arming_dir` ∧ base present ⇒ `Fork{base}`) reads the arming `base` file.
So the safety shape holds from the workflow path: script → confined orchestrator
(cwd = arming dir, armed) → fork at base.

**Footgun (caught mid-probe):** omitting `isolation:'worktree'` on the workflow
`agent()` call runs it **in-place in the coord tree** (no fork; `show_toplevel`
== coord root, HEAD attached to `dispatch/<n>`) — the same in-place hazard as the
Agent-tool path, and it silently burns a probe cycle. The tell is a
schema-returned `show_toplevel` equal to the coord root.

Builds on [[mem.fact.workflow.agent-worktree-fires-create-fork-hook]] (hook fires,
cwd = session dir) and [[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]].
See also [[mem.pattern.dispatch.unarmed-agent-worker-runs-in-coord-tree]].
