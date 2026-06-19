# Enforce ADR-001 layering: engine crate extraction + dependency fitness gate

## Context

ADR-001 (`leaf ← engine ← command, no cycles`) is review-only. It has no
automated enforcement, and the 2026-06-19 architecture audit confirmed the drift
the ADR itself predicted: cycles between the relation engine and the command tier
(addressed by SL-111). ADR-001 explicitly names two escalations once cycles
recur — promoting the engine to its own crate (a compiler-enforced boundary) and
adding a fitness function. We are well past the trigger.

The leverage here is durability: a boundary the compiler or CI checks stops the
drift from re-growing the moment a future change reintroduces an upward edge.
Without it, every structural fix (SL-111, etc.) decays back under review-only
pressure.

## Scope & Objectives

- Pick the enforcement mechanism (decide in `/design`): the heavier option is
  extracting the engine tier (`entity`, `registry`, relation core, `fsutil`, the
  hoisted `kinds`) into its own workspace crate so `command ← engine` is enforced
  by Cargo — a command module simply cannot be named from the engine crate; the
  lighter option is a dependency fitness test (e.g. a `cargo test` that parses
  `use crate::` edges and asserts the layer DAG). Design weighs them — not
  mutually exclusive.
- Classify each module to a layer authoritatively (the audit's draft table is a
  starting point) and encode that classification where the gate can read it.
- Make a layering violation **fail the build / `just gate`**, with a clear error
  naming the offending edge.
- Update ADR-001 (or a follow-on ADR / revision) to record the enforcement
  decision and the layer assignment as canon.

Closure intent: introducing a deliberate upward edge fails the gate locally;
`just gate` runs the check; the layer map is recorded as canon, not folklore.

## Non-Goals

- Breaking the existing cycles — that is SL-111 (a hard dependency; this slice
  assumes the engine tier is already acyclic).
- Resolving the `install`-as-utility wart ADR-001 flagged, beyond classifying it.
- A full crate-per-tier split of the whole repo; scope is the engine boundary
  plus the gate, not a workspace reorganisation.

## Summary

Make ADR-001 a machine-checked boundary — engine crate extraction and/or a
dependency fitness test — so layering violations fail the build instead of
relying on review. Depends on SL-111 having broken the current cycles.

## Follow-Ups

- Feeds back into ADR-001 governance (record the enforcement + layer map).
- If a fitness test is chosen over (or before) the crate split, the crate split
  may become its own later slice.
