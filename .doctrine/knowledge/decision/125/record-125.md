# DEC-125: Review findings unify on RV with a section reference

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Two implementations of one concept

The design run holds its own `Finding { id, subject, summary, blocking, resolution }`
(`snapshot.rs:293-307`), raised and disposed through the run's `declare` wire
(`run.rs:1105-1156`). The `RV` kind holds findings with
`severity: blocker | major | minor | nit` and a five-verb turn-based ledger —
raise, dispose, verify, contest, withdraw — with a baton and cooperative roles.

The runtime one is strictly poorer, and it is the one that gates the design lock.
`blocking` defaults to `false`, so a finding is non-blocking **by omission**; there
is no severity, no verify step and no contest. A design can therefore lock over an
arbitrary pile of undisposed major findings without anyone deciding anything.

`SL-233`'s design did not intend two models. It states
(`slice/233/design.md:298-299`): *"Findings remain runtime data unless promoted to
knowledge or the final authored review ledger."* Two tiers of one concept, with a
promotion leg. **The promotion leg was never built** — nothing in
`src/design_run/` references an `RV`, and `IntegratedReview.id` is a `DesignId`,
not a canonical ref. This is the fourth instance in this subsystem of a design
bullet that specified something never built, with a thinner mechanism left
standing in its place.

## The decision

Scrap the runtime `Finding`. Enrich `RV` findings with a **document-section
reference**, and let the design run derive over the ledger it already should have
been using.

## Why the tier objection does not hold

Run state is guarded by a monotonic revision under compare-and-swap, and the
design says that revision *"structurally cannot see the authored tier"*. Writing
findings into an authored `RV` looks like breaking that. It is not — the run
already crosses into authored state three ways:

1. **`design materialise` writes `design.md`**, and guards the crossing with the
   *authored watermark*: the fingerprint of the file as Doctrine last left it. The
   run does not own authored state; it references it and detects drift.
2. **Checkpoints mint authored entities from inside `apply`**, through `DEC-086`'s
   journalled intent with reservation and recovery-from-first-incomplete-effect.
   `DEC-120` through `DEC-126` were all minted this way, from inside one run.
3. **The project has already made this exact tier call once**, deliberately:
   observations moved to authored-and-committed *"so they survive the worktree they
   were captured in"*, with the review-noise tradeoff written down rather than
   dodged.

So *findings are runtime data* was a choice, not an architectural constraint — and
it is the choice that produced the boolean, the missing severity, the absent
verify and contest, and a promotion leg nobody built.

## What unification buys

- **Severity**, which is what a derived outstanding-findings summary needs and what
  a boolean cannot express.
- **Verify and contest**, which the runtime model has no equivalent of.
- **The promotion leg dissolves** rather than getting built: findings are born
  authored, so there is nothing to promote.
- `DEC-124`'s review disposition can cite a real `RV`, which it cannot today.
- **Section-by-section adversarial review becomes a first-class `RV` capability**
  rather than a design-run-only trick. It is not a design-specific workflow, and
  the section granularity `RV` currently lacks is the only reason it was not
  already one.

## The genuine complication

A run finding binds to a section whose **fingerprint moves**, and `DEC-066`
invalidation is snapshot-internal. An `RV` finding carrying a section reference
needs the fingerprint too, or staleness stops working — and `RV` has no
fingerprint concept. The watermark precedent suggests the shape (the `RV` records
the fingerprint it was raised against; the run derives staleness) but this is real
work and is where the design should expect to be hardest.

Minor and manageable: an `RV` must exist before findings enter it, so entering
`reviewing` mints one. That is arguably an improvement — the review becomes a
first-class artefact from the start rather than something that appears at the end.

## Scope

`SL-244` specifies its conditions against the **`RV`-backed** model, not against
`Finding`. Writing contracts against a record that is going away is the mistake
`DEC-121` caught on the other edge — specifying mechanism around something that
should not exist. The migration is its own item.

Related: `DEC-124` (the review disposition that cites an `RV`), `DEC-126` (the
classification this changes), `DEC-120`, `DEC-121` (the same pathology, other
edge), `DEC-086` (checkpoint minting), `DEC-066` (the fingerprint rule the
complication is about), `ADR-007` (the review ledger), `SPEC-029`.
