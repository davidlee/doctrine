# CON-004: Landed state is append-only

The zero-archaeology invariant for the RFC-025 capsule model. Three clauses that
say different things; conflating them produces either a useless constraint or an
expensive one.

## 1. Inside a capsule, before landing — total freedom

The worker commits, amends, rebases, resets, iterates however it likes. The
capsule is disposable and its history is its own. Mess is *supposed* to happen
here, and making it cheap is the point.

## 2. At the boundary — all-or-nothing, and failure does not cost the work

Either the accepted ref advances to the new OID, or it does not move. What is
forbidden is a **partial landing that needs reconstructing**.

This does **not** mean redoing the phase. On a refusal the capsule is still
alive with its work intact: the control plane hands the refusal back ("you
touched an undeclared path"), the worker fixes it in place, rings again, and
harvest re-runs at a new pinned OID. A retry loop, not archaeology.

Teardown is therefore conditional on a **successful landing**, not on the worker
exiting.

## 3. After landing — append only

The landed commit is immutable. A subsequent fix is a new commit from a new
capsule contracted at the new base. Never an amend, rebase, or force-push of
landed history. Incremental fixes across phases are expected and cheap; they
append.

## Why this is achievable here and not today

Today's coordination branch is a mutable staging area — advance coord, project
refs, write journal rows, mint a candidate, pin, replay integrate. That
apparatus is the source of the model's recovery affordances:
`sync --record-integration`, `candidate ingest`, supersede-on-moved-trunk,
`--allow-corpus-clobber`, `marker --clear`, `gc --force`, the heal-forward
import ladder, `dispatch next`'s triage prescriptions. Each is individually
reasonable; collectively they are a census of the model's wedge states.

With one immutable commit object in and one CAS ref advance out, there is no
intermediate mutable state to wedge.

## Structural consequence: idempotency is content-addressed

RT-6 requires idempotent harvest and keys it on the journal. With no journal
there is nothing to key on except **the pinned OID itself**. Harvesting the same
OID twice is a no-op by content-addressing; a new OID is a fresh attempt.
Simpler than bookkept idempotency, and it cannot drift out of sync with reality.

## Verification

Not aspirational — asserted:

- **Every probe row** gains a post-refusal assertion: after a refusal at any
  stage, the trusted side is byte-identical to its pre-run state, modulo the
  disposable quarantine dir. (Today only H5 asserts byte-identity.)
- **H15 sharpens**: kill at *each* stage, not one interruption point. After any
  interruption the accepted ref is at its old value or its new one, and nothing
  else requires cleanup.
- **New measurement** — recovery affordances reachable; target zero. Counted
  against the current model too, for the before/after column.

## Related

- [[capsule-admission-conflict-semantics]] — the reuse split that buys this.
- RFC-025 `red-team.md` RT-6 (doorbell/idempotency), RT-5 (OID pinning).
- `probe-specs.md` § Measurements, rows H5/H15.
