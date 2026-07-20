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

## Evidence

### SL-206 PHASE-04 (2026-07-06)

PHASE-04 shipped a `doctrine-role: probe` def + 3 orchestrator read tokens; the
conformance surface validating them shipped in PHASE-03 (on `dispatch/206` only).
Stale PATH `doctrine doctor` → exit 1 (`invalid doctrine-role: probe`, 3×
`forbidden MCP token`). Fresh worktree binary → exit 0, zero conformance findings.
Both tag `0.16.0`; divergence is source, not version. Worked around by
land-not-rewrite (orchestrator lands the worker's bytes, fresh-binary verified) —
see `mem.pattern.dispatch.worker-commit-stale-path-false-red`.

### SL-199 F-1 / RV-251 (2026-07)

PHASE-04 upgraded `agent_conformance` from role-blind to role-keyed (worker →
`worker_commit`; orchestrator → three funnel tokens). `just validate` ran the
pre-SL-199 installed v0.15.7 and flagged all three funnel tokens as forbidden.
The SL-199-built binary's `doctor` was **exit 0**. Same class: stale
PATH-binary artifact masking a current-build-green state. Also recurred at
SL-199 T7/F8.

## Fix direction

Point the `worker_commit` belt's `doctrine doctor` (and any other belt corpus
verb) at the **workspace build** — e.g. resolve `./target/debug/doctrine` in the
worktree, or `cargo run -q --bin doctrine --`, or have the gate invoke the same
binary the running MCP server was built from — rather than PATH. This dissolves
the self-tightening-check bootstrap: a doctor-check upgrade is validated against
itself, not against the last-installed generation. Until fixed, multi-phase
dispatch where a phase teaches the binary a check its predecessor shipped will
keep false-redding until promotion.

## Provenance

- `originates_from` SL-199 (audit RV-251, finding F-1)
- `originates_from` SL-206 PHASE-04 (case-notes)
- Merged with IMP-270 (duplicate — same root cause, same fix)
- Adjacent: IMP-193 (retire `validate` once `doctor` is fast enough)
- Related: ISS-216 (reseat gap), dispatch funnel (SL-198/199)
- Related: ISS-053 (same stale-binary class, conformance registry angle)
