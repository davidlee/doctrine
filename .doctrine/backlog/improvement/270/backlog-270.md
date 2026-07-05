# IMP-270: Gate should run the workspace-built doctrine, not the stale installed binary

## Problem

`just validate` (and the `doctrine check gate` path) invokes **bare** `doctrine
doctor` / `doctrine prompt check`, which resolves to the **installed** binary on
`PATH` (`~/.cargo/bin/doctrine`). That binary lags the working tree: it reflects
whatever was last `cargo install`ed, not the code under review. So any slice that
**upgrades a doctor check** produces a false gate-red — the old installed check
runs against the new artifact and rejects it — until the operator integrates and
reinstalls.

## Evidence (recurring class)

- **SL-199 F-1 (RV-251).** PHASE-04 upgraded `agent_conformance` from role-blind
  ("a confined agent may hold only `worker_commit`") to role-keyed (worker →
  `worker_commit`; orchestrator → the three funnel tokens), shipping the
  `dispatch-orchestrator` agent-def the upgrade blesses. `just validate` ran the
  pre-SL-199 installed v0.15.7 and flagged all three funnel tokens as forbidden.
  The SL-199-built binary's `doctor` was **exit 0**. Pure measurement artifact.
- **SL-199 T7 / F8.** Same class — a stale PATH-binary artifact masking a
  current-build-green state.

## Idea

Point the gate recipes at the **workspace build** — `./target/debug/doctrine`
(after a `cargo build`) or `cargo run -q --bin doctrine --` — so the check that
runs is the check under review. Alternatively, have `check gate` shell to the
in-tree binary it just built rather than whatever is on `PATH`. This dissolves the
self-tightening-check bootstrap: a doctor-check upgrade is validated against
itself, not against the last-installed generation.

## Relations

- `originates_from` SL-199 (audit RV-251, finding F-1).
- Adjacent to IMP-193 (retire `validate` once `doctor` is fast enough) — same
  recipe surface, different concern.
