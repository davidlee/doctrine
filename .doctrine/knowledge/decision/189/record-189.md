# DEC-189: Representativeness binds the row list, not only the readable set

## The generalisation

`DEC-185` set the bar for the conformance fixture's **readable set**: it must be
no wider than the property under test requires — representativeness, not
production parity. The bar generalises one level up, to **which propositions are
rows at all**.

A row whose control cannot fire is worth much less than it reads, in exactly the
sense `ISS-341` named for the readable set: *the verdicts are not false — they
are worth much less than they read, and nothing in the transcript says so.*

## The suite already states the rule; this record only applies it

From the `Unrowed` doc comment in `conformance.rs`:

> A row needs a delta that falsifies its reading … A condition no delta can
> change is not a test. Rowing one anyway would produce a row whose control
> cannot fire — the `B4` defect the round-6 split exists to remove — so it would
> *lower* the suite's honesty while raising its row count.

That rule is the one thing in the fourteen-property suite that is genuinely
mechanism-independent, and it is the reason `Unrowed`/`Reading` has no outcome
field at all: attaching a verdict to an observation is a type change a reviewer
sees in the diff.

## What it decides

**The row list does not port across confinement mechanisms.** Porting the
taxonomy and the discipline is required; porting the *membership* is refused,
because membership is a function of what deltas the mechanism admits.

Under a microVM boundary, four of the fourteen lose their host-facing delta:

| property | why it has a delta under bubblewrap | why it loses one under a microVM |
|---|---|---|
| Row 10 `ClosedDescriptorSet` | one descriptor table, `exec` can leak | no shared table exists |
| Row 12 `OwnedStandardStreams` | trusted side's own endpoints | serial console / vsock is a different proposition |
| Row 13 `MappedCapsuleIdentity` | `uid_map`/`gid_map` are host-facing | guest-internal; says nothing about the host |
| Row 14 `ConfinedCapabilities` | `CapBnd`/`CapInh` in the host user ns | guest-internal, and guest root is assumed by the threat model |

Transplanting them would produce four green rows whose controls cannot fire —
the `ISS-341` defect family rebuilt on a new mechanism, which is the outcome the
whole rescue exists to avoid.

Conversely, a microVM earns rows bubblewrap never could: hypervisor device-set
conformance, guest-image closure identity, host-perimeter liveness, and the
tap-is-an-endpoint forward drop.

## Reading `REQ-459` criterion 3

Criterion 3 admits a backend that *"passes the same property suite
independently."* This decision reads **"the same property suite"** as the same
taxonomy, the same one-property-removed control discipline (`DEC-156`), and the
same four evidential tiers — **not** the same row membership.

That reading is not obviously the literal one, and it is not settled here.
`QUE-211` owns whether the backend axis is many mechanisms under one contract or
distinct assurance tiers, and its answer determines whether this reading needs a
`REV` against `SPEC-030` or merely a wording clarification alongside `IMP-405`.

## Provenance

Owner's direction, 2026-08-11, in the `RFC-025` capsule-rescue round. See
`RSK-231` for why the programme is being revisited architecturally rather than
patched, and `DEC-188` for the precedent this follows — declared and honest
beats derived and flattering.
