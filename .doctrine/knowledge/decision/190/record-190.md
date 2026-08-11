# DEC-190: Conformance splits into a neutral verdict kernel and per-mechanism payloads

## The decision

`crates/doctrine-control/src/conformance.rs` is **14,252 lines** in one file. It
is split in two:

- a **verdict kernel** — small, backend-neutral, the thing a reviewer can check
  without loading a confinement mechanism;
- a **payload layer** — per-mechanism, and not portable by design (`DEC-189`).

The bubblewrap payload layer is not migrated to a second backend. Owner's
direction, 2026-08-11: *a few hundred lines of policy and a bonfire for the
remaining 13,500*.

## What is kernel

Types and discipline, all currently `pub(crate)` inside `doctrine-control`:

- `Property` / `Axis` — the taxonomy, not its membership;
- `RowVerdict` — `Proven` / `Violated` / `Unproven` / `Indeterminate`, and the
  distinction between them, which is where the suite's honesty actually lives;
- `AdmissionVerdict`, `Admission`, `NotAdmitted` — including
  `Unavailable { missing, remedy }`, which is the `POL-002` facet-3 compliance
  mechanism for a feature-scoped host capability, not a convenience;
- the four evidential tiers: rows (admitted on), axes, `Claim`/`AuxOutcome`
  (table C — reported, never admitted on in either direction), and
  `Unrowed`/`Reading` (no outcome field at all, so a verdict cannot be attached
  by accident);
- `DEC-156`'s one-property-removed control discipline;
- `BackendId` — already open by construction: not a closed enum, no `Display`,
  `&'static str` so it cannot be built from runtime text, *"because the contract
  must bind mechanisms nobody has written yet."*

## What is payload

The shell text and its probes — `/proc/self/uid_map`, `/proc/self/gid_map`,
`/proc/self/fd`, `/proc/self/mountinfo`, `CapBnd`/`CapInh`, the sentinel-token
vocabulary — plus the fixture's own capsule construction. All of it is
namespace-shaped. None of it means anything across a hypervisor boundary.

## Why this is the answer to `RSK-231`'s third question

`RSK-231` asks: *"Can the conformance surface be reduced so that correctness is
checkable without loading the whole design?"* Today the answer is no, and 14k
lines is why — forming a defensible opinion about one row requires loading the
production backend, the fixture, the placement validator and several phases of
`SL-248`'s authored plan.

A rescue that ports the epistemology into a second 14k-line file has not bent the
cost curve; it has doubled it. The split *is* the bend.

## Sequencing, and it is load-bearing

**Do not extract before `QUE-211` settles.** The kernel is the part that must not
be redone, and whether the verdict carries an assurance tier changes its central
type. A tier-blind kernel is a kernel rewritten.

## The gate this owes

`AGENTS.md` behaviour-preservation: when changing shared machinery, the existing
suites are the proof — they must stay green **unchanged**. `RV-352` records the
reproduction baseline: two independent gate runs at `297 passed; 0 failed`, and
`backend verify` exit 0 with all fourteen properties and five axes `Proven`. The
extraction is admitted against that baseline or it is not admitted.

## Related

`DEC-189` (row membership is per-mechanism), `QUE-211` (what gates this),
`RSK-231`, `RFC-025`.
