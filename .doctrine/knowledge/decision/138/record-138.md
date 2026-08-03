# DEC-138: Review disposition binds the responder's turn, not the raiser's assent

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What the user is bound to

Responding — not agreement. `review-disposition-attested` holds the design run at
`reviewing → locked` while the minted `RV` carries a finding that is both
`blocker`-severity and in `open` or `contested` state. Once the responder has
disposed of it, the gate is open, whether or not the raiser has verified.

That line is the whole decision, and the property it buys is narrower than it
first sounds. `contest` moves a finding `answered → contested`
(`review.rs:2354`), which is blocking again under this predicate — so a raiser
*can* re-block, repeatedly. What they cannot do is leave the responder with
nothing to do about it: re-disposing clears it again.

Terminal resolution is where that fails. It blocks on
`{open, answered, contested}`, so the responder disposes, the finding becomes
`answered`, and the edge stays shut — **no act available to the responder opens
it**. Only the raiser's `verify` or `withdraw` does. The difference between an
obligation to respond and a wait on someone else's assent is exactly one state
wide, and that state is `answered`.

**The predicate is per-finding, not the review-level `await`.** `derived_status`
(`review.rs:1009-1022`) reads `status` and never `severity`, so `await ==
Responder` fires on an open `nit` — which would hold this edge for a typo. Its own
doc is explicit that it is *"a priority summary … never an exclusive gate — the
turn gate is per-finding `can`"*. The filter this row wants exists nowhere yet:
D-C9b's `doc_unresolved_blockers` (`review.rs:1490`) is `severity == Blocker &&
status ∉ {verified, withdrawn}`, which is the same test minus the state
restriction — it counts `answered`, and that is exactly the difference this record
turns on.

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

- **Block without exit** — a determined raiser can make the responder respond
  round after round, and nothing bounds it.
- **Exit without block** — waiving and disposing become the same act, and the
  ledger `DEC-125` bought is spent.

Be precise about what the block gives, because it is easy to overstate. It
guarantees the responder always holds a **clearing act**, which terminal
resolution does not. It does **not** bound the interaction: finding state is not
monotone, so the cycle can repeat. **Termination comes from the waiver**, not
from disposal — the second reason that arm cannot carry a precondition, and
independent of the honesty argument above.

## The fence is authority, not prohibition

This is the same posture the design takes on the review policy, and for the same
reason: the agent drives the CLI, so any rule expressible in the payload is one
an agent can satisfy. A stricter gate would be theatre. What holds instead is
that **both arms are user acts** — each carries an `AcceptanceDeclaration`, so
`AcceptanceAttestation::bind` is the only route — and that every choice lands in
the change log.

**Do not overstate that.** It is presentation and trace, not prevention. `--as` is
cooperative role assertion and explicitly *not* a security boundary (`ADR-007`,
`review.rs:2813`), and an `AcceptanceDeclaration` cannot distinguish a user's
payload from an agent's — `DEC-088`'s standing limitation, which this record does
not close and is not this slice's to close. So the property an implementation must
preserve is the achievable one: **either arm must be taken in the user's name and
must leave a change row.** An agent that wants past this gate can take it; what it
cannot do is take it quietly, or as its own.

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

## `Conducted` over an empty ledger

`DEC-125` mints the `RV` on **entry** to `reviewing`, and `derived_status` reads
an empty finding list as `(Done, None)`. So `Conducted { review }` is satisfiable
the moment the stage is entered, naming a review that has not happened. This is
accepted, and it is worth saying why it is not the empty-draft defect `sec-3`
refuses for `PerSection`.

There, an `EverySection` spelling would report the row satisfied with **no act
performed at all** — vacuous truth straight out of the engine. Here the row is
always discharged by a user disposition carrying an acceptance. A user who names
`Conducted` over an unreviewed `RV` has made a false statement in a durable record;
they have not slipped past a gate. The two cases differ in whether anybody acted.

The alternative — demanding evidence that a review occurred — is the toll this
record exists to refuse, and it would misfire immediately: the ledger's own
guidance is that *a clean review harvests nothing — a valid outcome, not a skipped
step*, so a review that found nothing is indistinguishable from one never run, by
construction.

## Standing limitation

The `Conducted` predicate needs `IMP-392`'s `RV`-backed finding set; the `Waived`
arm needs nothing from it. Until `IMP-392` lands, the interim is the same on both
arms rather than a strict path and a lax one.

Related: `DEC-125` (which this refines and does not supersede), `DEC-126`,
`DEC-088`, `ADR-007`.
