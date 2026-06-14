# IMP-072: WorktreeCreate hook for pre-worker fail-closability on the claude dispatch arm (deferred; NOT needed for base control)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Base control is already solved — this hook is NOT needed for it.** Confirmed
2026-06-14 (controlled marker-commit test): under `worktree.baseRef="head"` the claude
`Agent isolation:worktree` worker forks the **spawning orchestrator session's local
HEAD** ([[mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head]]).
So base==B is achieved by orchestrator **placement** (run the session inside the
`dispatch/<slice>` coordination worktree at its tip) — parallel AND serial-dependent
claude workers are base-controlled with **no hook** (SL-064 design §8, option Y).

**What Y/this hook would still add.** A `WorktreeCreate` creation-replacing hook
(Claude Code) gives **true pre-worker fail-closability**: it runs *before* the worker's
first command, so a wrong base (stripped `baseRef`, HEAD moved mid-spawn) could abort
the worker pre-run instead of being caught post-run by the §8.4 `verify-worker` belt
(loud + pre-import, but after a wasted worker run — the §8.1 residual). It is a
belt-tightening nicety, not a base-control mechanism.

**Cost (the gate).** WorktreeCreate has **no `agent_type` and no matcher** (confirmed
2026-06-14 docs) → fires for *every* `isolation:worktree` subagent in the repo,
reopening the ADR-011 D7 σ blast-radius: the hook must carry a benign pass-through +
a dispatch-vs-benign discriminator with a misclassification race. Build only if the
post-run `verify-worker` belt proves insufficient and true pre-worker fail-closed is
demonstrably needed — that is now the **sole** justification (arbitrary-B base control
is no longer one). See SL-064 design §8 + §8.5 (ADR-011 amendment).
