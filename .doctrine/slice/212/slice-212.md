# Ingest hand-resolved trunk merge

Delivers **IMP-127**: a sanctioned path — the "it's complicated" path, *not* a
`--force` — that adopts an operator-performed (base, source) 3-way merge as the
candidate's Doctrine merge, so `admit` has a real `merge_oid` to validate when the
internal all-or-nothing auto-merge conflicts.

> **GOVERNANCE-GATED — not ready for `/design` until RFC-006 resolves.**
> IMP-127 is RFC-006's `Conflict → hand-resolve` branch. RFC-006 (open) deliberates
> whether integrate may land a **non-FF merge onto trunk** — which **reverses
> ADR-012 D2/D4 FF-only** ("trunk projection is fast-forward-only … never auto
> non-ff") and the D6 legitimacy claim ("unreviewed code never touches trunk").
> That reversal requires an **ADR-012 Revision** after external review. This slice
> **cannot proceed to design or plan** until RFC-006 lands a direction and (if it
> keeps the reversal) an ADR-012 Revision exists. Split out of SL-211 (2026-07-10)
> so the ungoverned row/refusal work there ships without waiting on this gate.

## Context

`doctrine dispatch candidate create` is all-or-nothing: it runs its own internal
3-way merge and either records a clean candidate or, on *any* conflict, parks the
worktree at base with `status=conflicted, merge_oid=""` and stops. There is **no
verb to feed a manual resolution back in**: `admit` refuses ("no Doctrine merge to
validate"), re-running `create` recomputes the same conflict, resolving +
committing in the parked worktree and `git checkout -B` does not help (admit
validates the recorded `merge_oid`, still empty). So the close dead-ends even when
the git conflict is trivial to resolve by hand (SL-104: an add/add on a test file
both lineages created independently — 30s in plain git).

The deliberate decision was **no `--force`** — correct. The gap is the *opposite*
of force: an "it's complicated" path where the operator does the real 3-way merge
by hand and the tool **adopts the resolution** as the candidate's Doctrine merge,
still validated (it is a true 3-way of the recorded base+source), just
operator-performed.

RFC-006 §"The capability" frames it precisely: `plan_trunk_row(non-FF case)` →
`merge-tree` → Clean{tree} auto-commits as `planned_new_oid`; **Conflict** parks
an ephemeral private worktree for hand-resolve (this slice) and ingests the
resolved commit on re-run. The clean auto-merge and this hand-resolve branch are
the same ADR-012 reversal; RFC-006 reviews them as a unit.

Trigger is **base drift**: trunk moves between bundle creation and close so the
close_target auto-merge conflicts — split lineage, a sibling slice closing first,
a dirty-tree rescue commit. Not exotic. Today the only escape forfeits the
admitted-OID CAS provenance (direct-land, SL-104) — the integrity the candidate
seam exists to give. RFC-016 §D names the target: hand-resolved 3-way → **ingest
row**, recorded at the moment of variation.

## Scope & Objectives

- An ingest verb that adopts an operator's hand-resolved (base, source) merge
  commit as the candidate's `merge_oid`, subject to validation that it **is** a
  true 3-way merge of the recorded base and source (not an arbitrary tree) — so
  `admit` → `integrate` proceed on genuine provenance.
- Record the ingest as a first-class lineage row (RFC-016 §D) so downstream gates
  consume it mechanically.

Objective: a conflicted candidate is resolvable by hand and re-entered into the
sanctioned candidate → admit → integrate flow, **without** `--force` and
**without** forfeiting CAS provenance — once RFC-006 sanctions the trunk posture.

## Non-Goals

- The **clean** non-FF auto-merge (RFC-006's `Clean{tree}` branch) — decide in
  RFC-006 / a sibling slice whether it lands here or separately.
- The row-recording + close-gate-recognition for *already-landed* tips — that is
  **SL-211** (IMP-236/169), ungoverned, shipping first.
- Any bypass of the 3-way validity check or the admit OID CAS.

## Affected surface (coarse — /design refines, post-RFC-006)

- `src/dispatch.rs` — `plan_trunk_row` non-FF branch; candidate create/admit
  seam; ephemeral private-worktree hand-resolve plumbing.
- ADR-012 (via Revision) — the FF-only posture, if RFC-006 keeps the reversal.

## Risks / Assumptions / Open Questions

- **BLOCKER** RFC-006 must resolve first, and if it keeps the reversal, an
  ADR-012 Revision must exist. Tracked via `related RFC-006`.
- **R1** Adopting a hand-made merge OID must not weaken CAS provenance — validate
  it is a true 3-way of recorded (base, source).
- **OQ-1** Ephemeral private worktree vs the parked candidate worktree for the
  hand-resolve surface (RFC-006 says ephemeral private). Resolve post-RFC-006.

## Follow-Ups

- Depends on RFC-006 direction; may spawn an ADR-012 Revision.
- Complements SL-211 (the already-landed row/recognition half).
