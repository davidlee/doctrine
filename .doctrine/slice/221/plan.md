# Implementation Plan SL-221: Unify dispatch boundary writes on the object-db ref

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Collapse the dispatch boundary-write seam onto the object-db ref (design.md D-B1,
locked through RV-278/279/281). One writer, one truth: the `dispatch/<slice>` ref
is the sole source of committed boundary rows; nothing reads or writes a
working-tree boundaries ledger. The plan sequences the design's work into five
phases whose order **is** the behaviour-preservation gate — the earlier a phase,
the closer it holds the existing `mcp_server` + `e2e_dispatch_sync` invariant
assertions unchanged.

## Sequencing & Rationale

The dependency spine is: relocate the compose primitives down (so the CLI can
reuse them without an ADR-001 cycle) → extract the shared helper → point both
writers at it → make the consumer order-safe → delete the retired path.

- **PHASE-01 — relocate + generalise.** Pure move of the commit engine from
  `mcp_server/dispatch.rs` into `dispatch.rs` (engine tier) plus the one signature
  generalisation (`commit_on_behalf` takes an explicit `target_ref`). This is the
  behaviour-preservation crux; it lands first so every later phase composes commits
  from the engine tier. Invariant assertions stay green; the only edits are the
  bounded `target_ref` call-site churn (design §8 R1, RV-279 F-4).
- **PHASE-02 — extract `land_boundary_row`; conclude delegates.** The shared helper
  is introduced and the primary writer (conclude) is refactored onto it with no
  behaviour change. Kept separate from 01 so the relocation diff and the extraction
  diff are each independently verifiable.
- **PHASE-03 — rewire the escape hatch.** `record-boundary` moves from the
  working-tree write to `land_boundary_row` on the ref. This is the **one
  deliberate behaviour change** in the slice, so it is isolated in its own phase and
  its e2e is rewritten (not "kept unchanged") — the honest split from RV-279 F-4.
- **PHASE-04 — consumer-normalise ordering (D-B4).** `plan_phases` sorts by phase
  ordinal before chaining. Placed after 03 because the escape hatch landing an
  out-of-order phase on the ref is what makes the ordering hazard real; the fix is
  small, local, and independently testable (VT-1 out-of-order).
- **PHASE-05 — retire + prove dead.** With both writers on the ref (02, 03) and the
  consumer order-safe (04), the working-tree path is unreferenced and deletable.
  The red-first ISS-225 regression proves the clobber is impossible by construction
  once `commit_boundaries` is gone. This is the payoff: SL-064 §4.1 holds with no
  exception.

Phases share `src/dispatch.rs` heavily, so execution is **serial** (no
file-disjoint parallelism to exploit).

## Notes

### Plan-time verification (design premises re-grepped against the current tree)

The design was locked at author-time; every concrete premise was re-resolved
against the tree before scaffolding these phases. All held; the deferred open
questions resolve as follows.

- **OQ-3 (import stays behaviour-identical) — PROVEN.** `commit_on_behalf` today
  derives its CAS target from `HEAD` (`--symbolic-full-name HEAD`,
  `mcp_server/dispatch.rs:304`). The coord worktree checks out
  `refs/heads/dispatch/NNN` (`resolve_coord`), and `DISPATCH_REF_PREFIX =
  "refs/heads/dispatch/"` (`kinds.rs:38`), so `dispatch_ref(slice)` **equals** the
  ref HEAD resolves to. Both production call sites — `dispatch_import` (491) and
  `conclude_boundary_commit` (578) — run in the coord worktree, so passing the
  explicit `dispatch_ref(slice)` is behaviour-identical. The escape hatch (CLI, runs
  anywhere) is exactly why the explicit-ref generalisation is needed: `HEAD` would
  not resolve to the coord branch off-worktree.
- **OQ-2 (deletion audit) — CLEAN.** `ledger::read_boundaries_file`'s only
  production caller is `commit_boundaries` (`dispatch.rs:2802`, deleted PHASE-05);
  `ledger::record_boundary`'s only production caller is `run_record_boundary`
  (`dispatch.rs:871`, rewired PHASE-03). Both become unreferenced → deletable. The
  ref-side sibling `read_boundaries` has no production caller outside its own tests
  and is **not** in the deletion set (untouched).
- **OQ-1 (escape-hatch commit identity) — design default adopted.** Reuse
  `dispatch_identity()` with a `Conclude`-shaped commit provenance; the row-level
  `boundary::Provenance` field carries attribution. No new `Manual` identity variant
  unless operator attribution is later required (revisit is cheap — a follow-up, not
  a blocker).

### Implementation hazard surfaced at plan (not caught by the three design passes)

`Provenance` is an **overloaded name** across three unrelated types:
`boundary::Provenance` (Funnel/Manual/Solo, the row field), `knowledge::Provenance`,
and the commit-identity `Provenance` (Import/Conclude) that PHASE-01 relocates.
`dispatch.rs:29` already imports `use crate::boundary::{BoundaryRow, Provenance}`
and uses the unqualified `Provenance` in ~6 places (incl. `plan_phases`'s
provenance filter at 1952 and test helpers). Moving the commit-identity
`Provenance` into the same file collides. Resolution (PHASE-01, no design
deviation — the design signatures name the moved type `Provenance`): alias the
boundary import (`use crate::boundary::Provenance as BoundaryProvenance`, or
fully-qualify its use sites) so the moved commit-identity type owns the unqualified
name that `land_boundary_row(prov: &Provenance)` expects. The blast radius is
otherwise self-contained: the commit engine symbols are referenced only within
`mcp_server/dispatch.rs` today (confirmed by grep + the RV-281 reviewer's LSP pass),
so no external file needs `crate::dispatch::` requalification.

### Behaviour-preservation gate (the proof, per design §8 R1 / RV-279 F-4)

Phases 01–02 hold the existing `mcp_server` + `e2e_dispatch_sync` **invariant
assertions** green. Two bounded, behaviour-neutral edits ride along and are called
out, not smuggled: the `commit_on_behalf` `target_ref` call-site arg (PHASE-01), and
the `record-boundary` e2e rewritten to assert the ref (PHASE-03). PHASE-05 deletes
the `commit_boundaries`/prepare-review splice-test cluster — those tests exercise
*removed* behaviour, so deleting them is required, not a gate violation. The gate is
about invariants, not literal test bytes.
