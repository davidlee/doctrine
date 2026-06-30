# A `dispatch/` branch prefix is NOT unique to the coordination worktree — worker forks ride `dispatch/<name>` too

When identifying the dispatch **coordination worktree** by its branch, a bare
`branch.starts_with("dispatch/")` test is **wrong** — it matches worker forks too.

- **Coord tree** rides `dispatch/<NNN>` (slice number) — `worktree/coordinate.rs`
  (`format!("dispatch/{slice:03}")`, the `-b` on `git worktree add`).
- **Every claude dispatch worker fork** rides `dispatch/<name>` **unconditionally**
  — `worktree/create.rs` `act_on_create::Fork`: `let branch = format!("dispatch/{name}")`
  (e.g. `dispatch/agent-<hex>`), via `fork_core`.

Both satisfy `starts_with("dispatch/")`, so the prefix cannot tell a worker fork
from the coord tree. A predicate that needs to single out the coord worktree must
anchor on the **registered coordination-worktree dispatch state**, not the branch
prefix (and not even `dispatch/<NNN>`-numeric — the worker `name` is harness-supplied
and could be numeric).

Sharp edge note: the **benign** `act_on_create::Passthrough` path (a non-arming-dir
`isolation: worktree` Agent spawn) makes a **detached** tree at HEAD — so a probe
that spawns a worker WITHOUT `dispatch arm-spawn` observes detached HEAD and misses
the `dispatch/<name>` Fork shape entirely. The real dispatch worker is the **armed
Fork** path. Don't generalise the Passthrough observation onto it.

Surfaced by the SL-181 inquisition (RV-199 F-1, blocker): the slice's
"positive coordination signal" guard was void because worker forks matched it.
Related: [[mem.pattern.dispatch.claude-arm-coord-placement]],
[[mem.pattern.dispatch.fork-rung3-base-not-session-head]].
