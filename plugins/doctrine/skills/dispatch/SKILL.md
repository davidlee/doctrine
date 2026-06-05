---
name: dispatch
description: Placeholder — parallel sub-agent execution of a slice's phases is not yet implemented in Doctrine. Do not route here; use /execute for serial phase execution.
---

# Dispatch

> **Placeholder.** Not yet implemented.

Intended future home for parallel, sub-agent-driven execution of a slice's
phases — batching tasks by dependency and budget, dispatching workers in
isolated worktrees, and merging at phase boundaries. Until it exists, execute
phases serially with `/execute`.

When built, it would draw on `superpowers:dispatching-parallel-agents` and
`superpowers:using-git-worktrees` as authoring references.

## TODO

- [ ] Define when this skill triggers and its STOP condition.
- [ ] Spell out the batching / isolation / merge model against the doctrine
      slice + state surface.
- [ ] Wire a routing slot into `/route` once execution fan-out is real.
