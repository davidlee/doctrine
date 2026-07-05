# CHR-039 — Workflow-harness worktree-fork + arming probe (findings)

De-risks IDE-031 (Workflow-templated slice-driver). Empirical, on this harness
build (Claude Code, Opus 4.8, `docs/claude/workflows.md` runtime). Method:
throwaway one-agent Workflow spawning `agent(isolation:'worktree')`, observed via
a `DOCTRINE_BIN` shim that logs the `WorktreeCreate` hook payload then delegates
to the real `doctrine worktree create-fork`.

## Rig

- Instrumentation: `env.DOCTRINE_BIN` in `.claude/settings.local.json` → a shim
  (`scratchpad/doctrine-probe-shim.sh`) that appends `{ts, PWD, argv, env, raw
  stdin payload}` for `worktree create-fork` invocations, execs the real binary
  for everything else. Env route (not a tracked-file edit) — propagation into the
  workflow runtime confirmed live.
- Two runs of the identical probe: (A) shell cwd = primary `/workspace/doctrine`
  [edge]; (B) shell cwd = `/workspace/doctrine/.dispatch/SL-198` [dispatch/198].

## Probe matrix

| SQ | Verdict | Evidence |
|---|---|---|
| **SQ1** — hook fires for a workflow worktree-agent? | **YES** | Both runs logged a `hook_event_name:WorktreeCreate` payload; a real linked worktree was minted (`git-common-dir` = primary `.git`, shared). The runtime does NOT fork off HEAD without invoking the hook. |
| **SQ2** — can the confined trigger's cwd conditions hold? | **YES, controllable** | Payload `cwd` = the **shell/session working directory** (A: `/workspace/doctrine`; B: `/workspace/doctrine/.dispatch/SL-198`). In B, `cwd_is_coord_root ∧ coord_in_dispatch` both held; classifier returned Passthrough only because `base=None` (create.rs:222). Fork nested under the cwd's tree: `.dispatch/SL-198/.worktrees/wf_…-1`, detached at the coord HEAD `05604889` (dispatch/198), not edge. |
| **SQ3** — can `base` be armed given the script has no Bash? | **Yes — outside the script** | Confined-without-base is a benign Passthrough (create.rs:222), so arming is the sole missing ingredient. Arm in the launching shell (pre-arm) OR inside a spawned agent's Bash turn. Server-side `dispatch_arm` MCP is convenient (re-arm between phases in a pure-script driver) but **not mandatory**. Actual confined-fork-at-base not yet demonstrated. |
| SQ4 — funnel/review MCP clean in a background workflow? | **UNTESTED** | Highest-value remaining probe for the widened goal (autonomous design→inquisition→plan tail). |
| SQ5 — `taskBudget:{total}` live in this build? | **UNTESTED** | — |

## Corollary findings

- **Env propagation** — `settings.local.json env.DOCTRINE_BIN` reaches both the
  `WorktreeCreate` hook exec env AND the workflow agent's own env. (The stale
  ambient `DOCTRINE_BIN` in the launching shell — a deleted SL-199 dispatch
  binary — did NOT leak into hook exec; hooks get the settings env.)
- **MCP path ignores config-env `DOCTRINE_BIN`** (operator-reported, unverified
  here) — a dev/custom doctrine for the driver's MCP writes would need the value
  passed via the claude invocation env, not `settings.local.json`. Matters if the
  autonomous driver ever runs against an unreleased binary.
- **Passthrough posture** — the harness mints `.worktrees/wf_<runid>-N` nested
  under the cwd's tree, **detached at that tree's HEAD, no branch** (`worktree.
  baseRef = "head"`). Fork base tracks the session tree HEAD, not hard-pinned to
  edge. Probe worktrees are NOT auto-removed while the session lives — clean with
  `git worktree remove --force`.

## Verdict for IDE-031

Coarse design **viable, not dead**. The confined-arm path is reachable from a
Workflow: launch the driver session cwd'd in the coord tree (on `dispatch/<n>`),
arm `base`, and a workflow `agent(isolation:'worktree')` fires the confined fork
at `base` with the dispatch branch. Remaining gates before committing the design:
SQ3 empirical fork demo, and SQ4 (does the funnel/review/plan MCP surface run
clean inside a background workflow, or stall on the allowlist gate) — the latter
is load-bearing for the widened goal of driving the whole design→plan tail from a
workflow.
