# IMP-072: WorktreeCreate hook to enable parallel claude-quality isolated dispatch workers on subscription (option Y escape hatch)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Why deferred.** SL-064/SL-066 dispatch established (empirically) that claude
`Agent isolation:worktree` forks session-root main HEAD, opaque — not the
orchestrator's coordination-tree tip — so it can't be base-pinned to an isolated
`dispatch/<slice>` tip ([[mem.pattern.dispatch.claude-isolation-worktree-base-session-root-opaque]]).
v1 (option X) abandons claude isolated workers: parallel dispatch lives on the
subprocess arm (codex/pi → DeepSeek, ~$1/hr) where `fork --base B` pins any base;
the claude arm does premium solo `/execute`.

**What Y is.** A `WorktreeCreate` creation-replacing hook (Claude Code) that forks
the worker worktree off an orchestrator-chosen B (the deferred SL-056 `create-fork`,
now feasible — the hook *creates*, so it supplies path+base + can fail-closed).
Enables parallel **claude-quality** isolated workers on subscription billing.

**Cost (the gate).** WorktreeCreate has **no `agent_type` and no matcher** (confirmed
2026-06-14 docs) → fires for *every* `isolation:worktree` subagent in the repo,
reopening the ADR-011 D7 σ blast-radius: the hook must carry a benign pass-through +
a dispatch-vs-benign discriminator with a misclassification race. Only worth building
if parallel claude-quality isolated dispatch proves worth that cost over the cheap
subprocess arm. See SL-064 design §8 + §8.5 (ADR-011 amendment).
