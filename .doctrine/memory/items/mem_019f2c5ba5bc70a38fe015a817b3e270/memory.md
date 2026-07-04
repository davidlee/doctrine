# Confined subagent Bash cwd resets between calls — breaks positional-arming fork discrimination for a confined dispatch orchestrator

A **jailed subagent's Bash tool cwd resets to its assigned worktree root on
every tool call** — a `cd` takes effect *within* a call but is reverted by the
next. (The main-thread orchestrator's Bash cwd, by contrast, *persists* across
calls — [[mem_019efe28d60b7d51998f1f7912b8e7b8]] P3.)

**Witnessed live 2026-07-04 (Claude Code 2.1.198), SL-199 feasibility probe.**
A `general-purpose` subagent spawned with cwd inside a linked coord tree
(→ `Jail(coord-cwd)`): `cd .doctrine/state/dispatch/spawn && pwd` → spawn dir;
next separate call `pwd` → coord root. Reproduced twice.

## Why it breaks the confined dispatch orchestrator (Mode B / SL-199)

The claude-arm fork mechanism discriminates **Fork vs Passthrough positionally**:
`create-fork` (the `WorktreeCreate` hook) forks the worker at the armed base
**iff the payload `cwd` equals the arming spawn dir** (`create.rs:364`,
`cwd_is_arming = canonicalize(root/ARMING_SUBPATH) == cwd`). The payload `cwd`
follows the *spawner's* Bash cwd at spawn time. The main thread satisfies this by
parking its (persistent) cwd in `<coord>/.doctrine/state/dispatch/spawn` before
the `Agent` spawn.

A **confined orchestrator cannot** — its cwd resets to the coord root between the
`cd` call and the (separate) `Agent` spawn call. So a nested worker's payload
`cwd` is always the **coord root ≠ arming dir** → `cwd_is_arming = false` →
**Passthrough**: the worker lands *detached at coord HEAD with no
`dispatch/<name>` branch and no `jail/<name>.toml` record*
(`provision_jail_policy` runs only on the Fork arm, `create.rs:285`). Since
SL-198's `worker_commit` resolves its target by probing for that
`jail/<name>.toml` record, the confined arm's self-commit is dead on arrival.

Witnessed: nested spawn from the confined orchestrator forked at B but **DETACHED,
no branch, `jail/` dir absent**. Confinement itself held (orchestrator write
inside coord OK, escape to `/workspace/doctrine` denied read-only) — the wall is
fine; the *arming discriminator* is the break.

## How to apply (SL-199 design consequence)

The main-thread **positional-cwd arming does NOT port to a confined subagent
orchestrator**. SL-199 must decouple Fork discrimination from the spawner's
transient cwd. Leading direction: have `create-fork` discriminate on
**arming-file presence under the resolved root** (the orchestrator *can* write
`<coord>/.doctrine/state/dispatch/spawn/base` directly — a file write inside its
own jail, cwd-independent) rather than `cwd == arming-dir`. Must preserve the
main-thread arm and the benign-spawn guard (a disarm discipline, or an explicit
coord-in-dispatch marker, so ordinary `isolation:worktree` spawns from an armed
coord tree don't get force-forked). Touches `src/worktree/create.rs`.

Corollary: `arm-spawn` still works for the confined arm via
`doctrine dispatch arm-spawn --path .` (Bash from coord root, writes the coord's
spawn dir) — arming is fine; only spawn-time *positional* discrimination is
broken.

Related: [[mem.fact.dispatch.single-slot-arming-rendezvous]] (Agent blocks; no
turn between spawn and completion), [[mem.pattern.dispatch.claude-arm-coord-placement]]
(cwd-placement rules for the main arm), [[mem.fact.dispatch.arm-spawn-path-targets-cwd-root]].
