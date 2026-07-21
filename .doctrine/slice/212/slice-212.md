# Ingest hand-resolved trunk merge

Delivers **IMP-127**: a sanctioned path — the "it's complicated" path, *not* a
`--force` — that adopts an operator-performed (base, source) 3-way merge as the
candidate's Doctrine merge, so `admit` has a real `merge_oid` to validate when the
internal all-or-nothing auto-merge conflicts.

> **UNGATED (2026-07-21) — REV-030 dissolved the gate.** RFC-006 resolved
> **without** reversing ADR-012 D2/D4: the `IMP-127 → non-FF-integrate` dependency
> was false. When a candidate `base` is the current trunk tip, an operator-ingested
> `merge_oid` fast-forwards trunk exactly like a Doctrine-produced clean merge — no
> non-FF trunk mutation, no D2/D4 reversal. This slice is a **D4 candidate-merge
> extension** (REV-030 amended D4: `merge_oid` is validated by *provenance*, not
> *authorship*), with FF-only publication intact. See REV-030 for the rationale +
> adjudicated codex review. Split out of SL-211 (2026-07-10) as the row/refusal
> half shipped first.

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

## Affected surface (coarse — /design refines)

- `src/dispatch.rs` — the candidate create/admit seam: an **ingest verb** that
  flips a `Conflicted` row (`merge_oid=""`) to `Created` by recording the
  operator's resolved commit, gated by the provenance + content check. **Not**
  `plan_trunk_row`: integrate stays FF-only, untouched (REV-030).
- ADR-012 D4 — **already amended by REV-030** (provenance-not-authorship). No
  further governance change; this slice implements under it.
- On reconcile: SPEC-022 REQ-316 (FR-006) + candidate-layer prose — the tracked
  cascade (see Follow-Ups).

## Risks / Assumptions / Open Questions

- **R1** (design-load-bearing) Adopting a hand-made merge OID must not weaken CAS
  provenance. The contract is **not** parent-binding alone (codex F-1): a *true*
  3-way binds the resolved tree to the mechanical `merge-tree` on non-conflicting
  paths, admits operator freedom only at conflict loci (never an arbitrary tree),
  and requires ordered parents (first parent == `base_oid`). Exact predicate is a
  `/design` decision.
- **R2** (from codex F-2, IMP-303) Inspectable ≠ inspected: admit doesn't bind the
  admitted OID to an audit RV. Pre-existing (affects clean merges too); IMP-303
  should land before/with this slice's close path. `/design` decides whether to
  absorb it here or depend on it.
- **OQ-1** Parked candidate worktree vs a fresh ephemeral one for the hand-resolve
  surface. Leaning **reuse the parked worktree** (it already holds the conflicted
  state; a new ephemeral tree adds lifecycle machinery without added isolation or
  provenance). Confirm in `/design`.

## Follow-Ups

- Depends on RFC-006 direction; may spawn an ADR-012 Revision.
- Complements SL-211 (the already-landed row/recognition half).
- **REV-030 supersedes the gate** (2026-07-21): RFC-006 resolves *without*
  reversing D2/D4 — this slice is a D4 candidate-merge extension, FF-only intact.
  See REV-030 for the governance rationale + adjudicated codex review.
- **On reconcile — SPEC-022 cascade**: revise REQ-316 (FR-006) + the SPEC-022
  responsibilities line + candidate-layer prose to replace "Doctrine no-ff 3-way
  `merge_oid`" with the provenance-not-authorship framing, via a sibling REV once
  the ingest verb ships (SPEC-022 is retrospective — reconcile at ship, not ahead).
- **Design must resolve** (from codex review of REV-030): the "true 3-way" content
  bound (non-conflict paths match the mechanical merge; conflict loci free) + the
  ordered-parents check (parent-1 == base). R1's validation contract is *not*
  parent-binding alone.
- **IMP-303** (bind admitted OID → audit RV; should land before/with this close)
  and **IMP-304** (supersede clears a `Failed` trunk row) — pre-existing gaps
  surfaced by the REV-030 review.
