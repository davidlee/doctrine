## Source
RFC-011 case notes: SL-199-armswitch-a7, SL-206-P01-forkprobe-3f9b

## Problem
When `isolation: worktree` is omitted from a dispatch-worker Agent call, the agent runs IN-PLACE in the coord tree. The WorktreeCreate hook never fires, no fork is minted. The base-guard catches it but only after a full worker turn (~50-100k tokens). The symptom invites a long misdiagnosis chain (hook broken, jail broken, agent-def broken) before the simple cause is found.

## Cost
~3-5 wasted worker/probe spawns per occurrence.

## Suggested Fix
`dispatch arm-spawn` (or a pre-spawn lint) should emit a reminder that the next Agent spawn MUST carry `isolation: worktree`. A cheap invariant: on any "worker in coord tree" symptom, FIRST confirm the flag was present.

## Resolution — obsolete (2026-08-14, `SL-254` reconcile)

Both halves of this item's subject are gone. `SL-254` collapsed dispatch onto a
single confined-subprocess arm (`DEC-202`), so there is no in-session `Agent`
dispatch-worker spawn to omit `isolation: worktree` from; and `arm-spawn` —
the verb this item proposed hanging the pre-spawn lint on — has zero references
in `src/`.

A worker is now launched by `./scripts/spawn-confined.sh`, which creates the
worktree itself. The failure mode this item costs out (~3–5 wasted spawns per
occurrence, ~50–100k tokens each) is structurally unreachable: there is no flag
to forget.

Provenance: `SL-254` Follow-Ups `OQ-2` · `DEC-202`.
