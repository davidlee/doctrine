# IDE-021: Lease-based edit-exclusion coordination

The deferred coordination half of PRD-005 (Reservation & Leasing). PRD-005 §2
explicitly carves it out of the reservation contract as "a deferred extension of
the same primitive, specified elsewhere" — no such spec exists yet.

Distinct from reservation (SL-148): reservation is a **permanent** claim on an
*identity*; a lease is a **transient** claim on *write-exclusion* over an
existing entity, with TTL, heartbeat, release, force-acquire (grace period),
crash recovery, clock-skew handling, and per-kind write-gating. Reference shape:
lazyspec RFC-035 § Lease-Gated Writes (`/workspace/lazyspec/docs/rfcs/`).

Not a single slice — needs governance first: a PRD/tech spec for the lease
capability (likely descending PRD-005), then slice(s). Builds on the git-ref
`Claim` backend from SL-148 (same ref/CAS substrate).

Open questions the spec must settle: agent identity resolution; lease ref layout
vs. claim ref layout; which writes are gated and how the gate degrades offline;
interaction with the permanent reservation claim.
