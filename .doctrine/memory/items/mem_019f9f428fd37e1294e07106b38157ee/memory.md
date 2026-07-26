# Auditing a concurrency design: run the gate more than once — an intermittent failure is the invariant reporting itself

When a slice's central claim is a concurrency invariant, one green gate proves nothing — re-run it; a low-rate flake in the phase's own VT is often the invariant reporting its own violation, not test noise.

## The pattern

When the slice under audit makes a **concurrency claim**, a single green
`doctrine check gate` is close to worthless as evidence. Run it at least twice, and
loop the specific concurrency test tens of times. A test that fails at a low rate is
not automatically noise to be re-run until green — when it lives in the phase that
shipped the invariant, it is often the invariant reporting its own violation.

## The evidence (SL-228 / RV-312 F-1)

SL-228's flagship invariant is claim→bind→act: *a live fork implies its binding*.

- `doctrine check gate` → **exit 0**.
- The immediate re-run, same tree, same binary → **exit 101**, 4010 passed / 1 failed.
- The failing test was `worktree::create::tests::two_concurrent_same_name_spawns_leave_exactly_one_winner`
  — PHASE-04's *own* VT-5/VT-1, asserting the winner's `ForkBinding`.
- Measured rate: **2 failures / 20 runs (10%)**.
- `slice verify-vt 228` had already reported PHASE-04 VT-5 as **PASS**. That PASS was
  luck: verify-vt runs the criterion once.

The root cause was real, not test flakiness: `consume_arming_slot` and
`consume_arming_binding` ran *before* `claim_lock::acquire` in `act_on_create`
(`src/worktree/create.rs`), six lines above a comment asserting the lock covered the
whole claim→bind→act window. Two same-name spawns both consumed the one-shot arming;
one got `Some(binding)`, the other `None`; whichever won the claim was arbitrary. The
`None` winner forked **unbound** — which surfaces only as `worker_commit`'s
`unprovable-fork`, after a whole worker run is spent, with no re-bind verb.

Fixed by moving both consumes under the lock (they stay before `fork_core`, so
SL-199 EX-3's property is preserved). 40/40 after; suite 4011 passed / 0 failed.

## Why this generalises

- **`verify-vt` runs each criterion once.** A `VT` PASS on a concurrency criterion is
  a single sample. It is not a probabilistic claim and must not be read as one.
- **The bug and its detector often ship together.** The phase that builds a
  concurrency invariant usually writes the test that can falsify it. So the signal is
  already in the suite — the audit's job is to *sample it enough times to see it*.
- **The cheap read is the wrong one.** "Flaky test, re-run it" and "real invariant
  hole" produce identical first observations. Only the repeat distinguishes them, and
  the repeat is minutes.

## How to apply

At any audit whose slice claims a race-safety, locking, atomicity, or
claim/bind/act-style property:

1. Run `doctrine check gate` **twice** before believing green.
2. Identify the phase's concurrency VTs and loop each ~20–40×:
   `for i in $(seq 1 40); do cargo test -p doctrine --bin doctrine <test> -- --exact ...; done`
3. Any non-zero failure rate is a **finding**, not a re-run. Raise it before deciding
   whether it is the test or the invariant — then read the ordering of the lock
   acquisition against every input the locked region consumes.
4. Suspect specifically: state consumed *before* the lock that is *used* inside it.
   That shape is invisible to single-run testing and is what bit SL-228.

Related: [[mem.pattern.dispatch.half-arm-unprovable-fork]] — the operator-error route
to the same `unprovable-fork` dead end; this memory is the silent second route.
