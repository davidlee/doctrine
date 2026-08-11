# CPT-002: The headline capsule threat is the work, not the wall

Owner's framing, 2026-08-11, in the `RFC-025` capsule-rescue round:

> There are many threat vectors the capsule should account for, but by far the
> biggest risk vector is likely to remain what's written *inside* a capsule. It's
> easy to get fixated on network and other forms of "hard security" — and a
> certain amount of that is very healthy — but misalignment (or, incompetence
> indistinguishable from malice) is the headline threat, and Doctrine's most
> important defence is still its defining purpose: governing the imposition of
> the user's intent, and rooting out plausible-seeming but incorrect heresies.

## The concept

A capsule faces two threats that are **not** the same shape and are not defended
by the same machinery.

| | **escape** | **heresy** |
|---|---|---|
| what happens | the worker breaks the boundary and reaches the host | the worker stays perfectly inside the boundary and produces plausible, wrong work |
| defended by | the confinement mechanism | trusted-side inspection, verification, conformance, review, and the design gates |
| measured by | the property suite | almost nothing, presently |
| likelihood | low, and adversarial | **high, and continuous** |

Escape is the dramatic threat. Heresy is the **expected** one: it happens on
ordinary days, from ordinary agents, with no adversary present. An LLM worker
that confidently ships a subtly wrong derivation is not an edge case in this
system — it is the modal failure, and the corpus records instance after instance
of it.

Confinement bounds the **blast radius**. Governance bounds the **intent**. Only
the second addresses the threat that actually arrives.

## Why this needs saying, given that the architecture already does it

`ADR-020` already encodes the defence, in one sentence:

> Its local claims, process exit, branch names, and prose are evidence to
> inspect, never admission authority.

That is the whole anti-heresy stance — the worker's own account of its work
carries no authority, and the trusted side re-derives the verdict from artefacts.
`SPEC-030` builds it out: `FR-006` pins one result identity and checks ancestry,
merge policy, actual changed paths, declared scope, forbidden paths, modes and
interpretation obligations **from Git objects**; `FR-007` verifies the exact
normalized candidate in a separate fresh capsule and lets the *trusted* evidence
capture, not capsule prose, determine the verdict. `red-team.md` `RT-1` is the
same instinct — hostile verification code must not run in the control plane.

So the mechanism exists. What does not exist is the **stated priority**, and the
asymmetry of attention is the problem this record names.

## The attention asymmetry, and it is measurable

Confinement work is visible and satisfying: it produces fourteen properties, five
axes, a nineteen-row transcript and a green verdict. Anti-heresy work produces no
comparable artefact — a review that catches a plausible-but-wrong derivation
leaves a finding, not a suite.

So confinement wins the attention contest by default, and `RSK-231` is the
receipt: six days of intense agent toil spent on the wall, culminating in an
abandoned slice, while the wall was never the headline threat. That is not an
argument for neglecting confinement. It is an argument that **the ratio was never
chosen** — it was set by which work was easier to see.

## What follows

1. **`DEC-191`'s front list is scoped to confinement and must say so.** Kernel
   attack surface, filesystem reach, escape consequence, network perimeter,
   resource ceilings, credential reach, process containment — every one of them
   is an *escape* front. The per-front posture is an honest account of the wall;
   it is silent about the work, and a reader must not take a strong confinement
   profile as a strong safety claim. That inference is `ISS-341`'s family in its
   most consequential form.
2. **A strong confinement profile does not license weaker inspection.** If
   anything the implication runs the other way: a hypervisor boundary makes
   escape less likely without making heresy any less likely, so the *share* of
   residual risk carried by inspection goes **up** as the wall gets better.
3. **The capsule programme's success measure is not the property suite.** It is
   whether wrong work is caught before admission. Nothing currently measures
   that, and `SPEC-030` states no success measure for it.
4. **This belongs in the governance text, not only here.** `SPEC-030`'s
   § Concerns or § Overview should carry the priority explicitly, and it can ride
   the `REV` that `DEC-191` already requires against `REQ-459` rather than
   needing its own.

## What this record does not claim

Not that confinement is unimportant, and not that the hard-security work is
misdirected — the owner's words are *"a certain amount of that is very
healthy."* A capsule holding a worker with broad local authority and no boundary
would be indefensible regardless of the threat ranking, and the authority floor
(`DEC-191`) is unconditional. The claim is about **priority and attention
share**, not about whether the wall should exist.

Nor does it claim the ratio is currently wrong by some computed amount. It claims
the ratio was never deliberately set.

## Related

`ADR-020` (carries the defence as a mechanism), `SPEC-030` `FR-006`/`FR-007`,
`RSK-231` (the receipt), `DEC-191` (whose fronts this scopes), `DEC-189`,
`RFC-025`, `red-team.md` `RT-1`.
