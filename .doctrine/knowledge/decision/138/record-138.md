# DEC-138: Review disposition binds the responder's turn, not the raiser's assent

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What the user is bound to

Responding — not agreement. `review-disposition-attested` holds the design run
at `reviewing → locked` while the minted `RV` awaits the **responder** on a
`blocker`. Once the responder has disposed, the baton is with the raiser and the
gate is open, whether or not the raiser has verified.

That line is the whole decision. Requiring terminal resolution instead would make
progression depend on someone else's assent: `contest` hands a finding back for
re-disposition, so a raiser who keeps contesting keeps the responder on the ride
indefinitely. Disposal is monotone and the responder can always perform it;
assent is not theirs to give.

## Why the waiver stays unconditional

`Waived { reason }` is the user's exit, and it must work at the moment it is
worth having — over live findings, not only over an absent review. The draft this
record replaces made it inadmissible there, on the reasoning that an
unconditioned waiver is a synonym for dismissing the findings.

It is not, and the difference is what the ledger already records. A waiver leaves
every finding standing on the `RV`, undisposed and legible, beside a stated reason
in the change log. *I have read these and I am proceeding* is a different claim
from *these findings do not exist*, and the record keeps them different.

Without this arm the only exit is `withdraw`, which means **raised in error**.
Forcing a user who disagrees with a live finding to spell their decision as a
retraction of the review is worse than letting them say what they mean.

## Why neither half works alone

They are a pair, and each closes the other's failure mode.

- **Block without exit** — a contesting raiser owns the user's progression, and
  the obligation has no termination proof.
- **Exit without block** — waiving and disposing become the same act, and the
  ledger `DEC-125` bought is spent.

## The fence is authority, not prohibition

This is the same posture the design takes on the review policy, and for the same
reason: the agent drives the CLI, so any rule expressible in the payload is one
an agent can satisfy. A stricter gate would be theatre. What holds instead is
that **both arms are user acts** — each carries an `AcceptanceDeclaration`, so
`AcceptanceAttestation::bind` is the only route and `DEC-088`'s guarantee is
carried — and that every choice lands in the change log.

Whatever an implementation calls the mechanism, this is the property it must
preserve: **an agent must not be able to author either arm.** That is the
invariant; the naming is not.

## The asymmetry that must be stated

Two gates read the same `RV` under different rules, and this is deliberate:

| gate | subject | rule |
|---|---|---|
| `review-disposition-attested` | the design run's `reviewing → locked` | not awaiting the responder on a `blocker` |
| the close-gate (D-C9b) | the slice's `audit → reconcile`, `reconcile → done` | every `blocker` terminal — verified or withdrawn |

Different subjects at different altitudes: advancing a design run is not closing
the work it designs, and it is reasonable for closure to want the stricter thing.
Left unstated it reads as one rule implemented twice and inconsistently, so the
design says which is which.

## Standing limitation

Both arms need `IMP-392`'s `RV`-backed finding set. Until it lands, neither
condition is enforceable — so the interim is the same on both arms rather than a
strict path and a lax one.

Related: `DEC-125` (which this refines and does not supersede), `DEC-126`,
`DEC-088`, `ADR-007`.
