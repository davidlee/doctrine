# IMP-276: Rewrite dispatch-agent skill for self-commit funnel

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Shipped `dispatch-agent` SKILL.md still documents the retired
live-worktree-import funnel: "ro-.git blocks self-commit", footer
`worktreePath` → verify-worker → `worktree import --from-worktree`.

Actual claude-arm funnel since SL-198/SL-199: the worker self-commits via the
gated `worker_commit` MCP tool; the orchestrator lands via `dispatch_import` →
`dispatch_conclude_phase` → `dispatch_reap`.

Cost of the stale skill: each orchestrator session re-derives the real cadence
from tool schemas + memories (~2–3k tokens) and risks mis-instructing the
worker prompt with the retired mechanics.

Do a rewrite pass after SL-206 lands (its dispatch changes settle the funnel
shape). Source of the observation: RFC-011 case note
`[dispatch-agent; SL206-P11-resume]`.
