# ISS-219: worker_commit red-gate refusal embeds full check transcript (~295k chars)

## Defect

When `worker_commit` refuses at the commit gate (`reason: "commit-gate-red"`), the
refusal `detail` payload embeds the **entire** `doctrine check commit` transcript
verbatim. Observed in SL-206 P0: ~295k characters — thousands of `[Prose Citation]`
unresolved-citation warnings, the `[Lifecycle]` summary, and the full RED test-suite
output — returned through the MCP tool result to the (nested) worker, which then
funnels it up the spawn chain (worker → orchestrator → main thread).

## Impact

Token sink, multiplied by the spawn chain. A single red-gate refusal can dominate a
dispatch turn's token budget while carrying almost no actionable signal — the worker
needs the verdict + the failing lines, not the whole transcript. Directly relevant to
RFC-011 token-efficiency (case-note filed under `[P0 nested-spawn probe; sl206-unjail-p0]`).

## Fix direction

Truncate/summarize the gate output in the refusal `detail`: gate verdict +
first-N failing lines (or the `new`/`changed` offender keys) + a pointer to the full
transcript on disk, not the inlined transcript. Cap the detail length.

## Provenance

Surfaced by the SL-206 P0 confined-orchestrator nested-worker probe (2026-07-06).
Source: the `worker_commit` refusal path — `src/mcp_server/worker_commit.rs` (the
`check commit` gate result → refusal `detail` construction).
