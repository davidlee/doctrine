`/reconcile` is the **sole writer of reconciled truth** for per-slice artefacts,
and its write surface for `design.md` is a **direct edit**. When the governing
design run is locked with a materialised watermark, the edit lands *out of band*
and the run's section fingerprints diverge from the file. **That divergence is
the surface working as designed, not corruption** — do not treat it as damage
and do not repair it.

## Do not run the correcting-a-locked-run loop here

[[mem.pattern.design-run.correcting-a-locked-run]] gives a regress → hand-edit →
`adopt_authored` → re-lock cycle, at high trust, tagged `locked` and `design`.
It is correct **for a run still in flight** — one that will be re-locked and go
on to govern execution. Its final beat is the re-locking cost: a fresh
attestation per changed section in the lane `review_policy` names, plus **two
user acts** (the review-pass disposition and the design acceptance).

At reconcile none of that buys anything. The phases are implemented, audited and
green; the design run has already done its governing. Spending two user acts and
an adversarial attestation to re-lock a run whose slice is about to go `done` is
pure ceremony — and the acts are ones an agent must not author on its own
initiative anyway.

**The discriminator is the stage, not the lock.** Locked + still governing →
adopt. Locked + at reconcile → direct edit, divergence expected.

## Why the divergence is the point

A locked run cannot be corrected in flight, so every claim execution falsifies
**accumulates for the whole of execution**. That is the standing cost the lock
buys, and reconcile is where the interest is paid: the phases record each
divergence with its evidence as they find it, the audit collects them into the
reconciliation brief with a write surface named per item, and reconcile writes
them. `SL-238` paid eight prose corrections at once this way (`RV-363` `F-6`),
five of them the same shape — a design rule and the criterion resting on it
agreeing with each other and disagreeing with the tree.

## Two write surfaces reconcile does not own

- **`plan.toml` criteria.** `EN-`/`EX-`/`VT-` ids are immutable-append. An
  acceptance or disposition for a criterion goes in `design.md` §Verification
  and the slice notes — never by editing the criterion. (`SL-238` `F-7`.)
- **The conformance registry.** A "spurious undeclared/undelivered" finding is
  fixed by `doctrine slice selector add|rm`, which is what `slice conformance`
  reads out of `slice-NNN.toml`. The `design.md` change-table row is the human
  **mirror**; edit only the prose and conformance stays red. (`SL-238` `F-1`.)

Related: [[mem.pattern.design-run.correcting-a-locked-run]] (the in-flight
protocol this one bounds), [[mem.pattern.doctrine.core-loop]].
