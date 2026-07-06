# ISS-218: worker_commit gate belt runs doctrine doctor via stale PATH binary, not the build/coord binary

## Problem

The claude-arm `worker_commit` gate belt runs its conformance check
(`just validate` → `doctrine doctor`) via **PATH `~/.cargo/bin/doctrine`**. In a
jail that binary is readonly and built from **edge/main**, so it lags whatever is
in flight on `dispatch/<slice>`. When a later phase's authored content depends on
an earlier phase's not-yet-promoted binary-level change (new role, grown
allowlist, new check), the PATH doctor doesn't know it and **false-reds** a
correct delta as `commit-gate-red`.

This directly contradicts AGENTS.md's own rule: "run corpus-inspecting verbs from
the coord tree's `./target/debug/doctrine`, not `~/.cargo/bin/doctrine`."

## Evidence (SL-206 PHASE-04, 2026-07-06)

PHASE-04 shipped a `doctrine-role: probe` def + 3 orchestrator read tokens; the
conformance surface validating them shipped in PHASE-03 (on `dispatch/206` only).
Stale PATH `doctrine doctor` → exit 1 (`invalid doctrine-role: probe`, 3×
`forbidden MCP token`). Fresh worktree binary → exit 0, zero conformance findings.
Both tag `0.16.0`; divergence is source, not version. Worked around by
land-not-rewrite (orchestrator lands the worker's bytes, fresh-binary verified) —
see `mem.pattern.dispatch.worker-commit-stale-path-false-red`.

## Fix direction

Point the `worker_commit` belt's `doctrine doctor` (and any other belt corpus
verb) at the build/coord binary — e.g. resolve `./target/debug/doctrine` in the
worktree, or have the gate invoke the same binary the running MCP server was built
from — rather than PATH. Until then, multi-phase dispatch where a phase teaches
the binary a check its predecessor shipped will keep false-redding until
promotion.

Relates to ISS-216 (reseat gap) and the dispatch funnel (SL-198/199).
