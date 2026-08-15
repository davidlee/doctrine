# ISS-366: Empty review ledger derives done, not active-awaiting-raiser

A review with no findings derives `Done · await=none`, so a freshly-minted
ledger reports as finished before anyone has looked at anything.

```
$ doctrine review status RV-358      # minted, never used
RV-358 — design review of SL-238
design · done · await=none · findings 0 · rounds 0
```

## Where it is

`src/review.rs:1103` — `derived_status` opens with the empty case and returns
`(ReviewStatus::Done, Await::None)`:

```rust
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Done, Await::None);
    }
    ...
```

## Why that reading is wrong

It contradicts the seed template the same tool writes. `review new` scaffolds
`review-NNN.toml` carrying:

```toml
# Findings are append-only `[[finding]]` tables, added by `review raise`.
# Empty here at creation — a fresh review is Active, awaiting the raiser (D-C8).
```

So the authored artefact says *active, awaiting raiser* and the engine says
*done* over the same bytes. `ADR-007` D-C8 is the authority and the template
quotes it.

The empty set is genuinely ambiguous — "nobody has raised yet" and "the pass
ran and found nothing" are different facts that a finding count of zero cannot
tell apart. `derived_status` resolves that ambiguity toward the flattering
reading, which is the shape `STD-003`'s second prohibition names: a surface
asserting a clean result it never observed. `RV-350` and `RV-353` show the
distinction has to be recoverable — a concluded pass that found nothing is a
result worth stating, and it should not encode identically to an untouched
ledger.

## Cost

The lie is quiet and lands on the reviewing stage. `/design`'s reviewing
obligation routes findings onto an RV ledger, and an agent that checks the
ledger's status before raising is told the review is already done. Found while
resuming `SL-238`'s design run — `RV-358` had been minted the night before and
read `done`, which briefly looked like the pass had happened.

## Shape of the fix

Not obvious, and that is why this is an issue rather than a chore. The engine
cannot distinguish the two empty cases from the finding vector alone, so the
fix needs a fact the ledger does not currently store — a conclusion marker, a
round count that survives, or a raiser-side "pass ran" attestation. Whatever
carries it has to respect the storage rule: a review's status is derived and
must not become a stored field.

Adjacent but distinct from `IMP-433` (lift RV derived status to a tier
engine-side readers can reach) — that one is about *reachability* of the
derived value, this one is about the value being wrong at the empty case. They
should probably be fixed together.
