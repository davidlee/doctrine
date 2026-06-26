# Fork-landed phase leaves an unbound source-delta (conformance undelivered)

Solo /execute on a fork never stamps capture_phase_boundary on the primary tree → conformance reports the phase's paths UNDELIVERED (empty registry). Bootstrap with record-delta(baseline..forktip).

## Why

The SL-147 solo phase-binding (`src/state.rs` `capture_phase_boundary`) stamps a
phase's `code_start_oid`/`code_end_oid` at the in_progress/completed flips **in
the tree where the flip happens**. When `/execute` runs in `mode=solo` on an
isolated worktree fork (not `/dispatch`), the code lands on the fork branch and
the boundary is never written to the *primary* tree's registry. The phase shows
`completed`, but `slice conformance <id>` reads the primary registry and finds no
row for it → the design-target selectors come back **undelivered** (the empty/
incomplete-registry signal), even though the work is real and on the fork.

This is the opposite cell from boundary-pollution
([[mem_019f031a315c7803900fcf398092e674]]): there a *polluted* range over-reports
(undeclared); here a *missing* range under-reports (undelivered). Same machinery,
two failure modes.

## How to apply

When `slice conformance <id>` reports a phase's paths as **undelivered** during
an audit of a fork-based solo `/execute`, do **not** read it as dropped work:

- Confirm the delta exists on the fork: `git log --oneline <fork-branch>`.
- A solo phase lands as exactly one non-merge feat commit; the true range is
  `baseline..forktip` (e.g. the edge baseline before the fork → the fork tip).
- Bootstrap the registry (audit step-2 backstop / record-delta escape hatch):
  `doctrine slice record-delta <id> PHASE-NN --start <baseline> --end <forktip>`,
  then re-run conformance — it should read conformant.

Confirmed empirically in the SL-157 PHASE-01 audit (RV-166 F-1): fork
`sl-157-phase-01`, delta `da243b3d`, baseline `42c55624`.
