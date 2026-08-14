# RSK-232: Worktree subagent write-confinement is unenforced

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The risk

An in-session subagent given a worktree (`isolation: worktree`, `/fork`,
`/worktree`) can write anywhere in the repo. Nothing stops it reaching back into
the primary tree — not a hook, not the OS.

This is not a defect anyone introduced. It is the residue of removing a
mechanism that did two jobs, and only one of them was the one being retired.

## How it arose

`SL-182` built the `PreToolUse` confinement wall with a single remit, stated in
`DEC-152`: *"The wall exists to keep **worktree** subagents inside their
worktree. That is the whole of its remit."* Its `resolve_target` had four arms —
`Jail(wt)` for a confirmed worktree, `Reject` for everything else with an
`agent_id`.

The `Reject` arm was the defect. It caught every ordinary subagent as collateral
(`IMP-269`, `IMP-342`, `IMP-401` leg 2). `DEC-152` (accepted 2026-08-05) fixed
that by granting pass-through — **and explicitly preserved the other arm**:

> `Jail(wt)` behaviour is unchanged and stays proven by the existing suites (the
> behaviour-preservation gate).

`SL-247` was to build that. It was abandoned. `SL-254` then achieved
`DEC-152`'s *effect* by a different route — deleting `pretooluse.rs` and
`subagent.rs` whole — which removes the `Reject` arm and the `Jail(wt)` arm
together. The consequence `DEC-152` ruled out is what shipped.

`DEC-154`'s three-valued topology discriminator (confirmed-worktree / confirmed-
not / **unknown ⇒ keep the deny**) went the same way, unbuilt, on the same
surface.

## What is and is not exposed

**Not exposed: dispatch workers.** They are strictly better off. A worker is now
a bwrap-confined subprocess under `--ro-bind / /` plus an rw bind of its own
worktree — an OS boundary in place of a fail-open hook, which is the trade
`SL-254` was cut to make and made correctly. This risk is not an argument
against that slice.

**Exposed: every other worktree subagent.** Solo `/worktree` isolation is
explicitly untouched by `SL-254` (`slice-254.md` § *Solo worktrees*), and the
harness's own `isolation: worktree` keeps working. Those subagents get a
worktree and no boundary. The failure is quiet: the write succeeds.

## Why it may still be acceptable

`DEC-152`'s own rationale narrows what is actually at stake, and it should be
read before this is treated as urgent:

- the wall is *"a guard-rail against mostly-accidental holes in the primary
  tree, not adversarial containment"* (owner, settling `SL-247` `OQ-4`);
- `RSK-225` already made the strong claim untrue on this arm — the wall mediated
  only `Bash|Edit|Write`, so any writable MCP tool went around it;
- development runs inside an outer bubblewrap jail.

So the honest statement is **a guard-rail was removed, not a containment
boundary breached**. What makes it worth tracking is that it was removed
*silently* — no decision record says "we are dropping this", and `DEC-152` says
the opposite.

## Fix directions

Not settled:

- **Accept and restate.** If the guard-rail is not worth rebuilding, say so in a
  `DEC` and correct `DEC-152`'s preservation clause, so the corpus stops
  claiming a guarantee that does not exist. This is the floor — it costs one
  record and removes the false claim either way.
- **Rebuild at spawn, not at deny.** `DEC-154`'s own residual note argues this:
  *"the answer … is NOT to widen the deny but to move the check to spawn time"* —
  a verb that asserts the spawner is correctly seated before launch. Positive
  assertion where a human can read the error, rather than inference at the point
  of denial.
- **Confine solo worktrees the way dispatch is confined.** Spawn worktree
  subagents as confined subprocesses under the same bwrap prefix. Uniform with
  the surviving arm; the cost is that it is no longer an in-session subagent.

Prefer the first as an immediate hygiene step regardless of which of the others
is taken.

## Provenance

- `IMP-401` leg 1 — *"worktree-isolated subagents stay in their worktree"*. Leg 2
  (the acute defect) shipped; this is leg 1, which lost its enforcement rather
  than being satisfied. Resolved there, tracked here.
- `DEC-152` — the preservation clause this contradicts.
- `DEC-154` — the unbuilt discriminator, and the spawn-time alternative.
- `SL-254` — the slice that deleted the surface.
- `RSK-225` — the pre-existing hole that already limited what the wall claimed.
- `RFC-025` § *What subagents look like once dispatch is gone* — the post-capsule
  finding this is the executable half of.
