# Notes SL-206: Workflow-templated slice-driver

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — De-risk gate (operator-run empirical, 2026-07-06)

Run operator-side (not a worker): PHASE-01 produces no source delta, evidence
lands in authored notes, and verification is VH/VA — it does not fit the
worker-delta funnel. Rig: throwaway scratch slice **SL-207** + its dispatch coord
tree (`.dispatch/SL-207` on `dispatch/207`); a one-agent dynamic Workflow
(`scratchpad/p1-forkprobe.js`) spawning `agent(isolation:'worktree')`, driver Bash
cwd parked at the arming dir, `dispatch arm-spawn --base <sha>` armed.

### EX-1 / SQ3 — confined fork mints at the ARMED base on the dispatch branch — **PASS**

Decisive disambiguation run — armed base set to coord **HEAD's parent** so the
armed base ≠ cwd HEAD:

| fact | value |
|---|---|
| coord HEAD (cwd) | `1fadff60` |
| armed base (`.doctrine/state/dispatch/spawn/base`) | `7fe7e231` (HEAD^) |
| fork `git rev-parse HEAD` | **`7fe7e231`** — == armed base, **≠ cwd HEAD** |
| fork `--git-common-dir` | `/workspace/doctrine/.git` (shared → confined linked worktree) |
| fork toplevel | `.dispatch/SL-207/.worktrees/wf_7030fd9a-8ae-1` (nested under coord) |
| fork branch | `dispatch/wf_7030fd9a-8ae-1` (reflog: "Created from 7fe7e231") |

The fork landed at the **armed base**, not cwd HEAD ⇒ the `worktree create-fork`
classifier (`src/worktree/create.rs::classify_create`, POSITIONAL trigger
`cwd_is_arming_dir` + base present ⇒ `Fork{base}`) reads the arming `base` file
and forks there. A confirming run with armed base == coord HEAD (`1fadff60`) also
landed correctly. **Gotcha caught mid-probe:** the first run OMITTED
`isolation:'worktree'` on the workflow `agent()` call → ran in-place in the coord
tree, no fork (`show_toplevel` == coord root, HEAD attached to `dispatch/207`).
The footgun applies to the workflow `agent()` path, not just the Agent tool.

### EX-2 / ISS-216 — a CHANGED confined agent-def made live under .claude/agents/ — **PASS (manual placement)**

- `.claude/agents/*.md` (and the worker's `.doctrine/agents/` symlink target) are
  **gitignored** — the derived/runtime live-read tier.
- `doctrine install` has **no `--force`/`--reseat`** flag; it is skip-if-exists
  (`src/install.rs`) ⇒ it CANNOT reseat a changed, already-installed def
  (ISS-216 fault #1). Fault #2: the `dispatch-worker` live symlink still points at
  the pre-subdir-split `.doctrine/agents/dispatch-worker.md`.
- **Procedure (demonstrated):** write the new/changed def bytes directly to
  `.claude/agents/<name>.md` (gitignored derived path). A scratch probe def written
  v1 → rewritten v2 with an added `mcp__doctrine__dispatch_phase_receipt` token was
  immediately readable at the harness-read path with the v2 token present. For a
  NEW def (PHASE-04 `dispatch-probe.md`) place a plain file; for a CHANGED existing
  def (orchestrator gaining tokens) overwrite the existing plain file directly.

### EX-3 — not triggered (both premises PASS; no design revisit).

**VH-1 (human):** confirm from the EX-1 table that fork base == armed base
(`7fe7e231`), distinct from cwd HEAD. **VA-1 (agent):** ISS-216 workaround =
direct write to the gitignored `.claude/agents/<name>.md`; recorded above.

Scratch teardown (post-sign-off): remove `.dispatch/SL-207` coord tree + branch
`dispatch/207`, abandon SL-207.
