# Implementation Plan SL-048: Structural cross-corpus relation edges: governance seam + spec-ADR + product-product

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases implement ADR-010's relation contract bottom-up — spec, then the pure
vocabulary table, then the generic read seam, then the atomic cut-and-migrate,
then the write verbs and validation teeth, then the governance reconcile — so each
phase ends green and demonstrable and the one irreversible step (mutating
committed authored TOML) lands alone, fully gated, on machinery already proven by
tests. The design is locked at v3 (round-2 hostile pass integrated); this plan
sequences it and re-decides nothing. Where it departs from the design's §8 sketch
it is on **phase boundaries only** — the sketch's authority is explicitly
indicative, and the departure is forced by the design's own constraints, below.

## Sequencing & Rationale

**The load-bearing correction — the cut and the migration are one phase, not
two.** Design §5.1 mandates a *hard* parser cut to `[[relation]]` with **no
dual-read branch**, and the reader/`format_show` rewire deletes the typed
`[relationships]` struct fields (R2-C2). The §8 sketch put that rewire in PHASE-03
but stranded the on-disk corpus migration in a trailing PHASE-05. Those cannot be
separate green phases: a phase that makes readers expect `[[relation]]` while the
data is still typed leaves the repo's own corpus unreadable until a later phase —
a non-green intermediate the "each phase ends green" rule forbids. So the migrator
moves **into the cut** (PHASE-04): code-cut and data-migration land in one commit,
gated together by before/after byte-identical render goldens plus a storage-level
post-check. This is a boundary move, not a design change — the design's content
(what is built, the seam, the gates) is untouched.

**PHASE-01 (tech spec) first because semantics precede code.** The relation
contract currently lives inline in the design; PHASE-01 re-homes it to a technical
spec (design §5.6, D5) that points at ADR-010 + `RELATION_RULES` rather than
mirroring the enumeration (storage rule). Settling the model, tiers, and
validation policy in prose before the table is authored keeps PHASE-02 a
transcription of agreed semantics, not a place to re-litigate them.

**PHASE-02 (the table + enum) is pure and changes nothing observable.**
`RELATION_RULES` and the new `GovernedBy`/`Consumes` variants are leaf-layer code
(ADR-001) with no storage, reader, overlay, or render effect — so existing suites
stay green *unchanged*, which is itself the proof the phase is non-disruptive. Two
round-2 traps are disarmed here before any consumer relies on the table: the enum
`Ord` is pinned **== the table's label order** (R2-C1) so `inspect`'s
`BTreeMap<RelationLabel>` regroup stays canonical and new variants land at their
axis-run tail; and every pre-existing `sources` cell is audited against the
*shipped* accessor (R2-M2) — catching `members` = PRD·SPEC — *before* the
exact-coverage tests downstream can contradict live behaviour. `inbound_name` is
pinned `== name()` for every legacy label (R2-M3): generalising the `supersedes`
special-case must not silently re-word any other inbound render.

**PHASE-03 (the read seam) builds `read_block` and the table-driven overlay in
isolation — proven before it is wired.** This follows the project's leaf-first
pattern: the generic parser (source-kind-aware legality, illegal rows → findings
not edges, canonical emit order — X1/X2) is unit-tested against synthetic
`[[relation]]` fixtures while no live reader calls it and the corpus is untouched,
so a failure here cannot be confounded by the rewire or the migration. The overlay
rewrite (R2-M4) — `OverlayMap::build` iterating the table, the hardcoded const
deleted — is a behaviour-preserving refactor whose exact-coverage arm (b)
(overlay-backed labels == resolvable graph labels) proves the table matches the
shipped allocation before PHASE-04 leans on it. Keeping this phase off the live
read path is what makes PHASE-04's risk legible.

**PHASE-04 (the cut) is the one irreversible phase, isolated and fully gated.** It
flips every accessor and `format_show` to `read_block`, deletes the typed fields,
and runs the deterministic one-shot migrator over the committed corpus — atomically
(per the correction above). The gate is deliberately doubled because X1 decouples
on-disk row order from rendered output: render goldens (`inspect` / `*-show` /
`show --json`) are *necessary* but the storage-level post-check (R2-m3) is what
catches a migrator perturbing the TOML in ways the render normalises away — F1
ordering (typed leftovers precede all arrays), tier-1-labels-only in `[[relation]]`,
no migrated label left in a typed slot. The migration excludes the governance
supersession pair (OD-3): `superseded_by` has no transactional owning verb yet
(that is IMP-006, not this slice), so its pair stays typed; only `related` migrates.

**PHASE-05 (verbs + validation) is the write path, and depends on the migrated
corpus.** It is sequenced after the cut because `append_edge` reasons about a
document that already carries `[[relation]]` arrays — its EOF-append *defence*
(R2-m1) guards the F1 invariant against a later hand-edit, which only matters once
the shape exists. No shipped verb writes the migrating tier-1 fields today (they
were scaffold-seed/read-only), so the writer was never on the cut's critical path —
confirming the verbs belong here, not earlier. This phase adds the genuinely *new*
code the round-2 pass surfaced: the forward legal-**kind** check (R2-M1 —
`ensure_ref_resolves` only dir-probes, it does not check target kind), the new
corpus-edge `validate` walk (R2-M5 — `run_validate` had no edge walk to "extend"),
and the supersession cross-check reading the typed field through its own seam
(R2-m2). All validation is report-only — the reseat precedent: reveal drift, never
rewrite.

**PHASE-06 (governance reconcile) is docs-only and last because it ratifies what
shipped.** The ADR-010 amendment (D3 carried corpus-wide), the SPEC-005/006/016
references to the new contract spec, and the IMP-032/IMP-006 reclassifications all
describe decisions the prior phases made real — recording them before the
capability exists would be premature. It carries the whole-corpus `validate` /
final workspace gate.

## Notes

- **Criteria immutability.** `PHASE-NN` and `EN-/EX-/VT-/VA-` ids are fixed once
  authored — corrections append, they never renumber.
- **Boundary departure from design §8, recorded.** The sketch's PHASE-03 is split
  (read machinery / the cut) and its migrator is pulled forward into the cut; the
  sketch's PHASE-05 docs tail becomes PHASE-06. Rationale above; design content
  unchanged. The decomposition, not the design, is `/plan`'s authority here.
- **Exact-coverage lands where its driver is rewired.** Arm (a) (reader-capable
  labels == table labels) is by-construction once `read_block` is live, so it is a
  PHASE-04 VT; arm (b) (overlay coverage) is a PHASE-03 VT; both are seeded by the
  PHASE-02 R2-M2 source-cell audit. Splitting the two arms across the two rewire
  points keeps each assertion honest rather than tautological.
- **Behaviour-preservation is the migration's proof.** The cut changes the on-disk
  shape of already-authored edges; the existing per-kind, cordage, `backlog_order`,
  and golden suites — green *unchanged* — are the evidence the change is
  shape-only. Any tension surfaced against ADR-010/ADR-004 during execution returns
  through `/consult`; the slice re-decides nothing.
- **Deferrals have backlog homes** (no closure surprise): IMP-006 (transactional
  supersede verb / cross-kind lifecycle), IMP-032 (reclassified to the validate
  cross-check). F3 (is `related`/`consumes` semantically adequate for PRD↔PRD) was
  resolved to `consumes` at design lock (OD-1).
