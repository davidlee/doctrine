# IMP-301: dispatch_import: populate Refused.detail with undeclared paths

## Problem

`dispatch_import` refuses with `undeclared-scope` but `Refused.detail` is
empty — no offending paths are named. The orchestrator must source-dive
`conformance.rs` + run `slice selector list` to diagnose which files are
missing from the design-target selectors.

## Impact

Recurred **4 times** in the RFC-011 case notes (SL-210, SL-213, SL-222 ×2).
Each refusal costs ~15 min + several tool calls of manual diagnosis.

## Evidence

- SL-210-drive: "Diagnosis required source-diving conformance.rs + discovering
  `slice selector list`"
- SL-213-drive-4: "the Refused detail field was empty — the refusal names no
  offending paths, so diagnosing required reading src/worktree/import.rs"
- SL-222-drive-3c5cf7: "Refused detail field was empty" + stale glob typo
  (`src/command/**` matching nothing) went undetected

## Proposed fix

Populate `Refused.detail` with the list of undeclared paths. The
`classify_import` scope belt already computes the undeclared set — it just
doesn't surface it in the refusal. A one-field change in the MCP server
response path.

## Related

- IMP-256: plan-time selector completeness check (prevention)
- IMP-282: exclude slice-own process artifacts from undeclared leads (separate
  class of false-positive)
