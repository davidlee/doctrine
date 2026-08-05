# DEC-141: VT-5's locked clause splits to VT-6

## The question

SL-244 PHASE-06's `VT-5` mandated four clauses in `tests/e2e_design_state.rs`:
a declared receipt elides bodies and keeps identity; an unknown or mismatched
edge token elides nothing; **a locked run emits no block**; and a refusal reads
no asset. Three landed in that suite. The third cannot: every fixture there is a
cold run at `exploring`, and a run at `Locked` is reachable only through the
four-component ladder in `tests/e2e_design_review.rs` — section attestations, a
minted pass, a disposed blocking finding, and an acceptance over every section.

## The decision

Strike the locked clause from `VT-5`, both in its `expects` and from its
`keywords`, and append `VT-6` carrying it with
`test_file = "tests/e2e_design_review.rs"`. The clause is rehoused, not weakened
or waived.

## Why this shape and not another

**Not a two-file mandate.** The obvious repair — name both suites on `VT-5` —
is not expressible: `Vt::test_file` is `Option<String>` (`src/plan.rs:67`), one
path, no list and no glob. The vtgate reads exactly that one file
(`src/vtgate.rs:111`).

**Append, not renumber.** Criteria ids are immutable and edits append. A new
`VT-6` is the sanctioned edit; amending `VT-5`'s text with a dated note is the
form this phase's own `VT-1` already carries.

**Not a hoisted ladder.** The DRY-est alternative was to lift
`record_lock_acts` / `lock_payload` / `Component` into `tests/design_fixture/`,
which the review suite already shares with its siblings, so the state suite could
climb to `Locked` itself. Rejected: those helpers are method-bound to the review
suite's own `Fixture`, so the lift refactors a landed SL-233 suite mid-phase to
serve one assertion.

**Not a duplicated ladder.** Re-creating the four edges beside a cold fixture is
the parallel implementation this slice's own `VA-2` rows hunt for, and it would
have to track the review suite's copy as the gate model moves. Hand-editing the
snapshot to `Locked` is refused by design.

## What it costs

One criterion's evidence sits in a different suite from its three siblings. That
is the honest price and it is stated rather than argued away: `VT-5` and `VT-6`
are one behaviour split across the two stages of the machine that can host it.

## Consequences

- No selector change was owed — SL-244's design target is the glob
  `tests/e2e_design_*.rs` (`slice-244.toml:32`), which already covers the review
  suite, so `check plan`'s undeclared-`test_file` leg stays green.
- The review `Fixture` gained a four-line `resume` over the `common::doctrine_cmd`
  seam both suites already share.

Related: [[DEC-142]] — the other PHASE-06 ruling taken in the same consult.
