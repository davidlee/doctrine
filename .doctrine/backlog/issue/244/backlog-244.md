# ISS-244: Flaky: two_concurrent_same_name_spawns_leave_exactly_one_winner — winner's DispatchRecord binding intermittently None

## Symptom

`src/worktree/create.rs` test
`two_concurrent_same_name_spawns_leave_exactly_one_winner` failed once in roughly
ten full-suite runs, asserting on `record.binding() == None` for the *winner* of the
claim race. 4/4 subsequent full-suite runs were green.

## Suspected mechanism (unconfirmed)

The arming slot appears to be **consumable by the loser** of the branch-as-claim
race before the loser refuses — leaving the winner's `DispatchRecord` written
without its `(slice, phase)` binding. The binding is a one-shot consume mirroring
`base` (SL-228 PHASE-04, VT-6: "a stale arming slice/phase cannot mis-bind the next
spawn"). If that consume happens before the claim is adjudicated rather than after,
the loser burns the slot the winner still needs.

That is a hypothesis from the failure mode, not a diagnosis — nobody has reproduced
it under instrumentation.

## Why it matters

An unbound record is not cosmetic: `require_binding` turns it into the
`unprovable-fork` refusal, a deliberate dead-end triage beat. A worker whose fork
lost its binding to a concurrent same-name spawn cannot commit (`worker_commit`
refuses) and cannot be healed by import (which refuses the same unbound fork) —
recovery is operator-only. The window is narrow and needs two same-name spawns,
which the harness makes unlikely in practice, but the consequence for that fork is
terminal rather than degraded.

## Provenance

Observed by the SL-228 PHASE-05 worker while running the full suite repeatedly.
**Nothing in the PHASE-05 delta touches `create.rs`, arming, or
`bind_dispatch_record`** — the code under test is PHASE-04's, landed at `fefa4d8f`.
So this is a pre-existing defect surfaced by repetition, not a regression.

## Next step

Reproduce under a loop with the arming slot instrumented (`cargo test <name>
--exact` in a shell loop, several hundred iterations) before attempting a fix — a
speculative reorder of consume-vs-claim risks trading one race for another, and
PHASE-04's rejected-alternatives record (notes.md, D-P4-1) shows this area has
already eaten several plausible-but-wrong fixes.

## Related

- SL-228 PHASE-04 — branch-as-claim fork sequence, `ClaimLock`, VT-1 / VT-6
- ADR-008 — dispatch worker confinement and parallel spawn topology
