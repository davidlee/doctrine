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
| **SQ4** — doctrine MCP clean in a background workflow? | **YES** | Allowlisted `mcp__doctrine__memory_retrieve` ran clean inside a background workflow agent, returned 5/335 memories. Load-bearing positive for the autonomous design→plan tail. |
| **SQ4-escalate** — can a background worker escalate for user approval (`/consult` / `AskUserQuestion`)? | **NO live channel** | Workflow agent reported `AskUserQuestion` absent from its toolset; skills are main-thread only (agents can't invoke `/consult`). Escalation must be **return-value**: worker halts → returns structured "needs decision X" → the script/orchestrator boundary surfaces it to the user → answer feeds the next/resumed agent. Fits doctrine's report-and-halt model; IDE-031 must design for it. |
| **SQ4-permgate** — does an UN-allowlisted MCP call surface a live prompt / auto-deny / hang? | **RUNS CLEAN — no prompt** | With the main session in **acceptEdits** (not the usual YOLO/bypassPermissions), an un-allowlisted `mcp__doctrine__memory_search` from a background workflow agent **executed without any prompt, denial, or stall** (returned promptly; only a post-hoc output-size truncation). Likely the whole `doctrine` server is trusted via `enabledMcpjsonServers`, not gated per-tool. **NB: the mode matters** — in YOLO no prompt could ever surface anyway; this was tested under acceptEdits and STILL didn't prompt. |
| **SQ4-permgate (writes)** — do un-allowlisted MCP *writes* also run un-prompted? | **UNTESTED — safety flag** | Only a read was probed. If `worker_commit`/`dispatch_import`/`review_dispose` also run un-prompted from a background worker, the autonomous driver has unsupervised governance-write authority. IDE-031 must gate deliberately (cf. the existing server-side `worker_commit` gate), not inherit it. |

### Escalation verdict (combined)
A background workflow worker has **NO live channel to the user in either direction**:
- semantic ask — `AskUserQuestion` absent, skills unavailable (`/consult` impossible);
- per-action approval — un-allowlisted MCP executes without prompting.
All human-in-loop must be **return-value at the script boundary**: worker halts → returns a
structured "needs decision X" → the script/orchestrator (or the main thread) relays it to the
user → the answer feeds the next or resumed agent. This fits doctrine's report-and-halt dispatch
model, but IDE-031 must architect it explicitly (there is no automatic escalation).
| **SQ4-writes (FS)** — does a fork worker's filesystem write execute? | **YES, unconstrained** | An `isolation:'worktree'` agent wrote `haiku.txt` into its fork with no hook/permission block. FS writes are confined to the fork (import belt captures them). |
| **SQ4-mcp-in-fork** — can an `isolation:'worktree'` fork reach doctrine MCP? | **YES (clean binary)** | An isolated fork agent's `mcp__doctrine__memory_retrieve` returned 5/335 cleanly. ⇒ **no isolation limit** — the confined orchestrator (workflow → `agent(isolation:'worktree')` into the coord tree) CAN use the `dispatch_import`/`conclude_phase`/`reap` funnel MCP tools, subject to grant (dispatch-orchestrator subagent type already grants them). **MCP-funnel is viable; CLI-funnel not forced.** |
| ~~SQ4-writes (MCP) "not connected"~~ — **RETRACTED, shim artifact** | **retracted** | The earlier "MCP not connected from a fork" (writegate `memory_record`, first grant-test `memory_retrieve`) was a probe-rig artifact: the `DOCTRINE_BIN` shim wrapped `${DOCTRINE_BIN} serve --mcp`, breaking the stdio JSON-RPC handshake for the fork's OWN server. Main's pre-shim connection masked it (non-isolated agents inherit main's live pipe). Re-probed with a clean binary → forks connect. See the stdio-shim footgun below. |
| **SQ5** — is the workflow token `budget` live? | **plumbing live, unset** | The `budget` global exists and its methods ran (`budget.total`/`.remaining()` — no throw), but `total` was `null` (no `+Nk` directive this turn). A workflow CAN read `budget.remaining()` to pace a sub-orchestrator once a target is set. |

### Safety posture (operator conclusion)
**The workflow spawns a confined orchestrator, never a bare worker.** A workflow-spawned worker
has unsupervised FS-write authority in its fork, no live human channel (both escalation probes
negative), AND — corrected — **can reach doctrine MCP** from its fork. So containment is **by
policy, not by MCP-unreachability**: the worker is held in check only by the orchestrator-sole-writer
model (ADR-006), the gated `worker_commit`, the import belt, report-and-halt, and by granting workers
a **narrow** MCP tool surface (the dispatch-worker subagent type grants only `worker_commit`). The
**orchestrator is the trust boundary.** IDE-031 shape: **workflow (outer scripted driver) → confined
dispatch-orchestrator (gated writer, drives the funnel via its granted MCP tools) → workers in forks.**
The script never needs to Bash-arm `base` — it spawns ONE orchestrator, which arms + nested-spawns
workers from its own Bash turns (dissolves SQ3's no-Bash-in-script constraint). Prior art (SL-199,
verified live): `mem.fact.dispatch.confined-orchestrator-driveloop-realizable`,
`mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming`.

## Corollary findings

- **Env propagation** — `settings.local.json env.DOCTRINE_BIN` reaches both the
  `WorktreeCreate` hook exec env AND the workflow agent's own env. (The stale
  ambient `DOCTRINE_BIN` in the launching shell — a deleted SL-199 dispatch
  binary — did NOT leak into hook exec; hooks get the settings env.)
- **The doctrine MCP server IS launched via `DOCTRINE_BIN`.** `.mcp.json` command =
  `${DOCTRINE_BIN:-doctrine} serve --mcp`. So a dev/custom binary CAN front the MCP
  server through `settings.local.json env.DOCTRINE_BIN` (corrects the earlier
  operator belief that the MCP path ignores config-env). Caveat: the value is read
  at server-launch; a fork spawns its OWN server via this command in the fork's env.
- **STDIO-SHIM FOOTGUN (probe-rig lesson).** Wrapping `${DOCTRINE_BIN} serve --mcp`
  with a shell shim (even a transparent `exec "$REAL" "$@"`) **breaks fork-spawned
  MCP servers** — the stdio JSON-RPC handshake is intolerant of any wrapper
  buffering/latency. The main session's server, launched clean before the shim was
  set, keeps working — so non-isolated agents pass and ONLY forks fail, masquerading
  as an "isolation limit." Set `DOCTRINE_BIN` to a real binary, never a wrapper, when
  MCP is in play. (This invalidated an entire round of write-gate probing; caught by
  the operator, re-probed clean.)
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
