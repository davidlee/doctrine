## Source
RFC-011 case notes: SL-210-drive, SL-213-drive-4, SL-222-drive-3c5cf7, SL-208 sess-a

## Problem
`dispatch_import` refuses with `undeclared-scope` but `Refused.detail` is empty — no offending paths named. The orchestrator must source-dive `conformance.rs` + run `slice selector list` + `git diff --name-only` to diagnose. The CLI arm prints them (`report_undeclared_scope`); the MCP tool returns them empty.

## Cost
~4 tool calls per occurrence. Recurred across at least 4 slices.

## Suggested Fix
Populate `Refused.detail` with the offending paths from the scope check, matching the CLI arm's behaviour.

## Resolution (fixed)
Already shipped by SL-224 PHASE-01, before this item was triaged. `Refusal::scope_detail`
(`src/worktree/import.rs`) recomputes the undeclared set from the SAME pure predicate the
belt used (`conformance::undeclared_paths`) and renders it through the shared id-bearing
formatter (`conformance::undeclared_detail`) — one runnable
`doctrine slice selector add SL-NNN <path> --intent design-target` per offending path. The
MCP arm calls it on the refusal value in `src/mcp_server/dispatch.rs`. The CLI arm's
`report_undeclared_scope` renders through the same formatter, so both surfaces agree.
