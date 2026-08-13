# IDE-052: Revive drive-slice.js for main-worktree sequential drive in a clone

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`SL-254` PHASE-10 deleted `install/workflows/drive-slice.js` (the shipped
`/drive-slice` workflow) whole: its landing cadence hard-requires a **bound**
fork and a **committed** fork tip (`dispatch_import`'s `require_binding` else
`unprovable-fork`), and the collapsed single-arm path structurally produces
neither — forks stay unbound by design (`OQ-1`) and the worker hands back an
uncommitted working tree. Deletion commit: **`dc7347152`** — recover the file
with `git show dc7347152^:install/workflows/drive-slice.js`.

Refinement from the execution itself (2026-08-13): the file is not the pure
Mode-B artifact the ruling assumed. It carries a real `(A)`/pi subprocess arm
(`fork_tip: null`, worktree-diff import) alongside the `(B)` claude arm. What
kills it for BOTH arms is narrower and more useful to know when reviving it:
its single landing verb is `dispatch_import`, which resolves the phase from the
durable fork binding via `require_binding`
(`src/mcp_server/dispatch.rs:289-307`). So a revived driver does not need a new
landing *cadence* — it needs a landing verb that names its phase explicitly
rather than reading it off a binding. Its `(B)` path additionally spawned via
in-session nested `agent(isolation:'worktree')`, and its `(A)` path did not
spawn at all (it assumed a prior orchestrator had), so the spawn half must come
from `scripts/spawn-confined.sh` either way.

## The idea

Owner observation, made while ruling on the deletion (2026-08-13): the pattern
`drive-slice.js` automated — a driver working through a slice's phases,
spawning workers, landing their output — is structurally close to what a
super-orchestrator does today running phases **sequentially in one main
worktree, inside a clone/capsule checkout** (exactly `SL-254`'s own execution
mode per its `notes.md` "Execution environment and standing directions": a
microVM holding a separate checkout, no `/dispatch`, no `/worktree`, work
happens directly in place).

Rather than the funnel-based bound-fork/committed-tip machinery
`dispatch_import` requires, a revived driver for this pattern would:

- Not require a bound fork or a funnel row — the driving session IS the
  coordination tree already.
- Import a worker's uncommitted working-tree diff directly (as `/dispatch`'s
  own retained path already does post-`SL-254`, per
  `install/dispatch-mechanics.md`'s "orchestrator is the main thread,
  unconfined" section) rather than composing against a committed fork tip.
- Possibly formalise the very batching/escalation pattern used to drive
  `SL-254` itself: spawn a phase-batch orchestrator, let it self-verify and
  commit, hand back a compact report, escalate real findings rather than
  improvising past them — see `SL-254`'s own session transcript for a worked
  example of the shape.

## Open questions

- Is this a rewrite of `/drive-slice`, or a genuinely new workflow with a
  different name — `drive-slice`'s existing name and prior art (`SL-206`
  PHASE-13/14) may carry assumptions (bound forks, funnel rows) worth breaking
  from rather than patching.
- Relationship to `SL-255` (clone provisioning) — if workers move to writable
  clones with self-commit, does that change what "committed tip" means here,
  or does this driver pattern stay clone-agnostic (main worktree driving,
  regardless of how *workers* are provisioned)?
- Whether this belongs as a `doctrine` shipped workflow at all, or as
  documented practice (a skill) for how to run a super-orchestrator session —
  the `SL-254` session that prompted this idea used ad hoc `Agent`-tool
  spawns with a hand-written local brief, not a shipped mechanism.
