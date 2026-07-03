# IMP-250: Fork/subprocess dispatch-arm base-clean + import-gate parity (SL-191 PHASE-05 EX-4 deferral)

## Context

SL-191 PHASE-05 delivered the non-mutating `prove` base-clean cadence (fmt-check +
lint, never auto-fix) and the reject-and-halt import gate (`src/worktree/import.rs`)
for the **pi / shared-funnel** dispatch arm (EX-1..3, green). PHASE-05 EX-4
("subprocess-arm base-clean coverage decided and recorded — its own beat vs shared
funnel") was consciously **deferred**: the shipped arm is correct and gated, and
fork/subprocess-arm parity is a separate delivery path with no regression on the
shipped arm.

## Follow-up

Decide and implement whether the subprocess/fork dispatch arm gets its own
base-clean beat or rides the shared funnel gate, and add the import-gate coverage
for that arm. Disposed `follow-up` in RV-242 (SL-191 audit); this item is the
durable "recorded" half of EX-4.
