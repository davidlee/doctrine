# Implementation Plan SL-049: CLI list-surface & input-validation hygiene

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-049 bundles two independent CLI papercuts — IMP-017 (list/render seam) and
ISS-004 (input validation) — into one hygiene slice. Each is a self-contained
fix against the shared conventions of `listing.rs` and the entity scaffolders.
The design (§4) settles the phasing: one phase per fix, file-disjoint.

PHASE-01 (IMP-017) lives in `memory.rs` + a one-line doc edit in `main.rs`.
PHASE-02 (ISS-004) lives in `spec.rs`, `input.rs` + a clap-command edit in
`main.rs`. The only shared file is `main.rs`, and the two edits sit in disjoint
regions (the `CommonListArgs.columns` doc vs the `spec req add` command). The
read-only `listing.rs` and `input.rs`/`entity.rs` leaves are touched by at most
one phase each.

## Sequencing & Rationale

**Order is free.** No phase produces an artifact the other consumes; neither
shares a writable file beyond the disjoint `main.rs` regions. The phases are
listed PHASE-01 then PHASE-02 by convention only, not dependency.

**Why two phases, not one.** Each fix carries its own behaviour tests and its
own invariant surface. Splitting keeps each commit reviewable against a single
governing design section (§1 / §2) and keeps the per-fix verification distinct.
Bundling them into one phase would entangle two unrelated test suites and two
unrelated invariant sets behind one exit gate.

**Why bundle the slice at all.** Both are small drifts from the same shared
seams; one design pass reconciles them and the per-slice ceremony stays
proportionate to the work. The bundling is at the *slice* level — the phases
stay clean and independent.

**Dispatch note.** Because the phases are file-disjoint (modulo the trivial
`main.rs` split), they parallelise safely. Serial execution avoids the `main.rs`
merge entirely; a concurrent batch costs one trivial 3-way merge on disjoint
regions. Either is sound.

## Notes

- OQ-1 (§5) resolved at design: `SLUG_MAX = 100` **bytes**, not chars — an
  explicit `--slug` is verbatim/possibly-multibyte. Re-confirm at execute.
- FU-1 (§7): path-hostile explicit `--slug` (`/`, `..`, control chars) accepted
  verbatim into the symlink name is **pre-existing** (on `spec new --slug`) and
  out of scope; capture as a new backlog issue at close, do not fix here.
