---
name: dispatch-agent
description: The claude arm of `/dispatch` — spawn a worker via the `Agent` tool using the dispatch-worker subagent type with worktree isolation; a matcher-scoped SubagentStart hook stamps the worker marker. No env channel, no bwrap. Reached only from the `/dispatch` router on a claude↔env-marker agreement; do not invoke directly.
---

# Dispatch — claude arm (`Agent` tool + SubagentStart stamp)

The harness-shaped **spawn half** for Claude Code. The harness-identical funnel
(capture `B` → import → verify → branch-point → one commit → record) and the whole
drive loop live in the [`/dispatch` router](../dispatch/SKILL.md) — **do not
restate them here.** This skill is only *how a claude worker is spawned and given
its identity*.

**Reached from the router, never directly.** `/dispatch` routes here only when the
agent's harness self-belief (claude) **agrees** with env-marker detection
(`CLAUDECODE` etc.). A mismatch/unknown refuses there, naming the cause — never a
blind spawn.

## Spawn — `Agent` tool, default-created worktree, hook-stamped identity

There is **no `fork` verb and no env channel** on this arm. Launch the worker with
the `Agent` tool:

```
subagent_type: dispatch-worker        # PINNED to DISPATCH_WORKER_AGENT_TYPE (τ — drift test reds on divergence)
isolation: worktree                   # Claude default-creates the worktree; doctrine does not intermediate creation
prompt: <pre-distilled worker prompt> # policy digest, design excerpts, memories, task + file set, verify cmd
```

**Claude owns worktree creation.** doctrine does not run `git worktree add` here —
the `WorktreeCreate` `create-fork` path is **deferred** (its payload carries no
type/path/base — O3-RED). Identity is conferred *after* creation by a hook.

### Identity — the SubagentStart stamp (best-effort, fail-open)

A **matcher-scoped, sync-blocking `SubagentStart` hook** runs the claude analog of
`fork --worker`'s mark step:

```
SubagentStart hook:  doctrine worktree marker --stamp-subagent   (matcher: dispatch-worker; reads payload JSON on stdin)
    parse stdin       → refuse missing-cwd / bad-dir / missing-agent-type   (TRUST BOUNDARY)
    provision <cwd>   → ADR-006 D9 allowlist (withheld tier excluded)
    write_marker(cwd) → the marker IS worker identity
    # NO `git worktree add` (Claude owns it); NO compensating rollback (owns no worktree).
```

- **Not fail-closable (ADR-011 D6, O3-red).** SubagentStart is a **read-only** hook
  event: a non-zero exit only surfaces stderr — the subagent runs **regardless**.
  So the stamp **cannot abort the worker**.
- **Blocking ⇒ no write race.** The hook gates worker execution until it exits, so
  on hook **success** the marker + provisioning are present before the worker's
  first action — not a timing race.
- **Fail-OPEN on hook failure.** On a stamp failure the worker proceeds
  **unstamped and un-gateable by any hook**. The fence is therefore the layered
  funnel, not a hook exit code: the `import` `.doctrine/`/`.claude/` belt + the
  worker-mode guard + the pre-distilled prompt + the **orchestrator post-spawn
  marker check that aborts an unstamped fork (IMP-052)** — enforced where the
  harness *can* abort.

### One source of truth for the literal (τ)

`subagent_type: dispatch-worker` above is **pinned to the binary's
`DISPATCH_WORKER_AGENT_TYPE`** — replicated across the `Agent` `subagent_type`, the
SubagentStart matcher, and `install/agents/claude/dispatch-worker.md`'s `name`. A
one-character drift **fails OPEN** (the matcher never fires ⇒ no stamp ⇒
`marker_present == false` ⇒ `worker_mode == false` ⇒ the worker writes unrefused,
contained by the belt + IMP-052). A cross-surface **drift test REDS on mismatch**
(`src/worktree.rs`, `src/boot.rs`) so the replicas cannot silently diverge.

## Concurrency — parallel EXECUTION, one landing per base (υ)

Each SubagentStart fires independently for its own worktree, so **concurrent
file-disjoint claude dispatch is first-class *in execution*.** But v1 buys parallel
**execution, not parallel landing**: the funnel-back is serialized by `import`'s
stationary-head precond (router §7c). The orchestrator's sequential imports bump
coordination HEAD `B→B+1` after each landing, so the next sibling — also forked at
`B` — then hits `head-moved` and must **re-dispatch onto the bumped base**. **One
landing per base, not an orderly N-drain** (in-verb re-anchor deferred, IMP-043).
The headline (parallel execution) must not outrun this footnote.

## Confessed residuals (claude arm only)

- **Base-pinning (charge-2/M1).** The SubagentStart payload carries **no
  base/parent field**, so the worker forks from **session HEAD**, and the
  default-created worktree's base is **opaque and not orchestrator-controlled**.
  Under the §7c stationary-head constraint session HEAD **is** `B` in the common
  case; a move between base capture and spawn is caught **late, not silently** at
  `import` `head-moved` (cost = a wasted worker run, not a base-integrity
  violation). No hook-time pre-refusal in v1 (the hook sees no `B`).
- **Self-clear (§4c).** No env-lock, no bwrap on this arm: a non-compliant worker
  can `rm` its marker or `env -u DOCTRINE_WORKER`. claude worker-sole-writer is
  **accident-fenced + prompt-enforced, not malice-proof** until a free env channel
  or OS confinement lands (IDE-004 / userns-bwrap). The funnel's `.doctrine/`-reject
  belt is the malice containment, not the marker.
- **worker-on-main / no `DOCTRINE_WORKER` env** — the deferred D2b residual; the
  worker shares the jail-wide build target (§8).

## Red Flags

**Never:**
- Spawn with any `subagent_type` other than the pinned `dispatch-worker`, or hand
  the worker a base — Claude owns creation; the hook stamps identity.
- Treat a non-zero SubagentStart exit as having aborted the worker (read-only
  event — it did not). Rely on IMP-052, not the hook code.
- Oversell concurrency: claim parallel **landing**. v1 lands one worker per base.
- Run the `fork` verb or a bwrap profile here (that is the codex/pi arm,
  `/dispatch-subprocess`).
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy);
  the source is here under `plugins/`.

**Always:**
- Pin `subagent_type` to `DISPATCH_WORKER_AGENT_TYPE`; let the drift test guard it.
- Run the IMP-052 post-spawn marker check; abort an unstamped fork.
- Return to the router for the funnel cadence — import, verify, branch-point, one
  commit, record.
