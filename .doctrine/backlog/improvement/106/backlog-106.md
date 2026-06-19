# IMP-106: dispatch: preferred worker harness for subagent-vs-subprocess arm selection

The `[dispatch] preferred-subprocess-harness` config (IMP-101 / SL-108 D3)
only selects BETWEEN subprocess harnesses (codex vs pi). There is no config
mechanism to tell a Claude orchestrator whether to use Claude subagents
(dispatch-agent arm) or pi subprocess workers (dispatch-subprocess arm).

Currently the arm selection is inferred: a Claude orchestrator defaults to
the subagent arm (it can use the `Agent` tool), a codex/pi orchestrator
must use the subprocess arm. But a project may want Claude to dispatch pi
subprocess workers instead — e.g. for reproducibility, isolation, or because
the pi RPC protocol provides structured `agent_end` outcomes.

A new config key (likely `[dispatch] preferred-worker-harness` or similar)
would let the dispatch router select the arm explicitly rather than relying
on env-marker inference alone.
