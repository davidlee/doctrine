# Claude Agent isolation:worktree forks the spawning orchestrator session's local HEAD under baseRef=head — base controllable by placement, not ref-redirect

**Empirical, controlled test (2026-06-14, marker-commit probe — first-party, not a
subagent guess).** With `worktree.baseRef = "head"` in `.claude/settings.local.json`,
a claude `Agent` spawned with `isolation: worktree` forks its worktree from the
**spawning orchestrator session's current local HEAD**, NOT from `origin/main`, NOT
from an opaque "session-root."

Test: orchestrator entered a throwaway worktree, committed a unique empty marker
`d58ce62` (descendant of `main@babd656`, on no shared ref), then spawned a
`dispatch-worker` `isolation:worktree` Agent. The worker's worktree HEAD came back
**== `d58ce62`** — the orchestrator's moved HEAD. baseRef=head is honoured and tracks
the spawning session's HEAD wherever it points.

**The handle is orchestrator session PLACEMENT, not a ref.** To make the worker base
== B: put the orchestrator session *on* B (run inside the `dispatch/<slice>`
coordination worktree with its tip checked out — SL-064 §7c stationary-head + §8.0
leg-1, as originally designed). Do NOT try to redirect via `git update-ref
origin/main` — that is not the handle (and never was).

**Corrects (retracts) two earlier findings** that misdiagnosed the same observation:
- [[mem.pattern.dispatch.claude-isolation-worktree-base-session-root-opaque]] —
  claimed base is "opaque, not orchestrator-controllable" and "baseRef cannot point
  at an arbitrary tip." Both false: it points at the orchestrator's local HEAD, which
  is an arbitrary tip you set by placement.
- [[mem.pattern.dispatch.claude-agent-worktree-forks-origin-main-tracking-ref]] —
  its own spawn-3 evidence ("forked `main` HEAD anyway after setting origin/main to
  the dispatch tip") is the tell: the orchestrator session was *on main*, so the
  worker correctly forked main HEAD. It read controllability-by-ref failing as
  controllability failing. The orchestrator was never placed on the dispatch tip.

**Architecture consequence (reverses the prior conclusion).** Both P1 (origin
staleness) and P2 (dependent-phase base) are solved on the claude arm by baseRef=head
+ placement — no `WorktreeCreate` hook, no subprocess-only routing required:
- **Parallel file-disjoint phases:** orchestrator HEAD = common base B → every worker
  forks B.
- **Serial dependent phases:** orchestrator advances its own HEAD/tree to phase N's
  integrated tip before spawning N+1 → next worker forks the dependency. They are
  **claude-dispatchable**, contra the retracted "inline only" conclusion.

**Residual (unchanged, accepted — ADR-011 D6 class).** Not pre-worker fail-closable:
a stripped setting or a HEAD moved mid-spawn yields a wrong base caught only *after*
the worker runs, by the §8.4 `verify-worker` belt (`git merge-base --is-ancestor B
HEAD`), pre-import — loud and late, never silent. True pre-worker fail-closed still
wants `WorktreeCreate` (IMP-072), now a nicety for fail-closability, NOT the base-
control mechanism.

Related: [[mem.pattern.dispatch.fork-rung3-base-not-session-head]],
[[verify-harness-behavior-empirically]].
