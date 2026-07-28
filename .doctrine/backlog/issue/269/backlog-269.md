# ISS-269: slice conformance reads boundaries from the primary tree but phase status from cwd

Surfaced at the SL-231 audit (RV-318 F-6).

## Observed

Same binary, same slice, same instant — two roots, two answers:

```
$ cd /workspace/doctrine && ./target/debug/doctrine slice conformance 231
SL-231: conformance
undeclared (46): ...

$ cd .doctrine/state/dispatch/candidate/cand-231-review-001 \
    && /workspace/doctrine/target/debug/doctrine slice conformance 231
SL-231: conformance incomplete — partial coverage
  - recorded row for PHASE-01, which is not a completed phase
  - ... (all five phases)
```

All five `phase-0N.toml` in the primary read `status = "completed"`, and
`slice status 231` reports 5/5. The worktree run reports the exact inverse.

## Cause

`boundaries_path()` (`src/state.rs:716-722`) deliberately resolves through
`crate::git::primary_worktree(cwd)`, so the registry is found from anywhere.
Phase status does not — it is read from cwd's own
`.doctrine/state/slice/<NNN>/phases/`, which is gitignored runtime state and
therefore **absent from every linked worktree**. The two halves of the
completeness gate resolve to different roots: the gate sees five recorded rows
and zero completed phases.

## Impact

Not cosmetic. `dispatch sync --prepare-review` gates on this completeness check.
Worse, the `/audit` skill instructs the auditor to run `slice conformance` as the
mechanical drift signal *and* to review a dispatched slice on the candidate
interaction branch — which is a linked worktree. Following both instructions
produces `incomplete — partial coverage`, which the same skill says to treat as a
finding in its own right ("the registry was not recorded as phases landed"). An
auditor who trusts it will `record-delta` rows that already exist.

## Candidate fix

Resolve phase status through `primary_worktree(cwd)` as well — the registry
already establishes that runtime state has one home. Or refuse the completeness
gate outright when cwd is a linked worktree, rather than reporting a derived
answer from half-resolved inputs.

Related: ISS-268 (same report, different defect — row attribution), ISS-271
(`verify-vt`'s wrong-tree severity).
