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
the `WorktreeCreate` `create-fork` path is **deferred** on the **σ blast-radius**
(its payload carries no `agent_type`/matcher ⇒ it would fire for *every* worktree
creation, ADR-011 D6/D7 as amended by SL-064 §8), **not** on base control.
**Base == B by placement.** The boot installer sets `worktree.baseRef='head'`
(per-operator, `settings.local.json`, SL-064 §8.3), so Claude's default-created
worktree forks the **orchestrator session's local HEAD** — which the orchestrator
parks at `B` by running from the `dispatch/<slice>` coordination tree (cwd ==
coord tree, HEAD == B). Identity is conferred *after* creation by a hook; base
correctness is **verified** after the worker returns (`verify-worker`, below).

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
  `verify-worker` belt that aborts an unstamped fork** — enforced where the
  harness *can* abort.

### Post-spawn — `verify-worker` (base==B belt, replaces prose IMP-052)

After the worker returns and **before `import`**, the orchestrator verifies the
fork on the trusted side:

```
doctrine worktree verify-worker --base <B> --dir <worker-worktree>
    rev-parse --verify HEAD   → resolves?           else  no-worker-head
    marker present at <dir>?                          else  unstamped
    merge-base --is-ancestor <B> HEAD                 else  wrong-base
    → Ok (exit 0): proceed to import
```

Fail-LOUD: on any refusal it prints the token to stderr, exits non-zero, and
**leaves the fork in place** (diagnostic only — the orchestrator decides; the
funnel halts, never auto-merges). This **one verb** closes two prior residuals:
the prose **IMP-052** post-spawn marker check is now the `unstamped` verdict, and
**IMP-043**'s deferred content-base assertion is now the `wrong-base` verdict
(`merge-base --is-ancestor B HEAD` — the base==B proof, SL-064 §8.4). Read-classed
(no writes; harmless under worker-mode).

### One source of truth for the literal (τ)

`subagent_type: dispatch-worker` above is **pinned to the binary's
`DISPATCH_WORKER_AGENT_TYPE`** — replicated across the `Agent` `subagent_type`, the
SubagentStart matcher, and `install/agents/claude/dispatch-worker.md`'s `name`. A
one-character drift **fails OPEN** (the matcher never fires ⇒ no stamp ⇒
`marker_present == false` ⇒ `worker_mode == false` ⇒ the worker writes unrefused,
contained by the belt + IMP-052). A cross-surface **drift test REDS on mismatch**
(`src/worktree.rs`, `src/boot.rs`) so the replicas cannot silently diverge.

## Against `dispatch/<slice>` — no fork branch, so the cut is synthesized (EX-4)

The orchestrator drives from the `dispatch/<slice>` coordination worktree
(SL-064 / ADR-012). This arm is **fork-less**: Claude default-creates the worktree
and the worker delta is imported and committed **directly onto `dispatch/<slice>`**
— there is no per-worker fork branch to preserve as the code unit. So
`phase/<slice>-NN` must be **cut from `dispatch/<slice>` at sync** (design §4.3).

That cut needs an input the funnel records: after the batch's code commit and
**before** the knowledge commit, the orchestrator runs `doctrine dispatch
record-boundary --slice <N> --phase PHASE-NN --code-start <B> --code-end <B+1>` (the
router's funnel step 7a). Stage-1 `dispatch sync --prepare-review` tree-reads the
committed `boundaries.toml` and synthesizes one `phase/<slice>-NN` per row (empty-
code phases skipped). The recording is the orchestrator's act on the trusted side —
**not** the worker's; the worker still writes source only.

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

- **Base-pinning (charge-2/M1) — CLOSED (SL-064 §8).** No longer a residual: with
  `worktree.baseRef='head'` the worker forks the **orchestrator session's local
  HEAD**, which the orchestrator parks at `B` by placement (cwd == coord tree) — so
  the base **is** orchestrator-controlled, not opaque (ADR-011 D5 falsified). The
  post-spawn `verify-worker` belt **proves** base==B (`merge-base --is-ancestor B
  HEAD`) before import and halts loud on `wrong-base`, closing the former
  clean-applying-semantically-wrong import worst case (was IMP-043). A pre-worker
  fail-closable arm (`WorktreeCreate`, IMP-072) stays deferred — but on the σ
  blast-radius now, not on base control.
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
  event — it did not). Rely on `verify-worker`, not the hook code.
- Oversell concurrency: claim parallel **landing**. v1 lands one worker per base.
- Run the `fork` verb or a bwrap profile here (that is the codex/pi arm,
  `/dispatch-subprocess`).
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy);
  the source is here under `plugins/`.

**Always:**
- Pin `subagent_type` to `DISPATCH_WORKER_AGENT_TYPE`; let the drift test guard it.
- Run `verify-worker --base <B> --dir <worktree>` after the worker returns and
  before `import`; abort the funnel on any refusal (the fork is left in place).
- Return to the router for the funnel cadence — import, verify, branch-point, one
  commit, record.
