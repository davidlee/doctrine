# Reconciliation observe substrate (SPEC-002 A)

## Context

**SPEC-002** (Requirement Reconciliation Engine, draft, descends PRD-013) is the
observe→reconcile→close machinery ADR-003 names but says doctrine "inherits the
intent but not yet the machinery." This slice builds the **observe** half — the
two-tier truth substrate: store evidence, derive coverage and drift, and **never**
derive authored status from coverage (the load-bearing NF-001 line).

The spec is near build-ready (D1–D9, H1–H5, OQ-1..3, REC wiring checklist App. A).
Foundations it rests on already ship: the ADR-009 slice FSM (`slice status`, F12
closure seam), the `src/git.rs` git-anchor/staleness seam (SL-007/008), and the
PRD-002 spec/requirement entities. This slice is greenfield only where the spec is.

**Governing decisions** (no re-deciding here; tensions go back through `/consult`):
SPEC-002 (D1–D9), ADR-003 (canonical loop, explicit-authorship-not-derivation),
ADR-009 (FSM, two-tier truth model vocabulary). Realises requirements
**REQ-108, REQ-109, REQ-110, REQ-111** (FR), **REQ-114, REQ-115** (NF-001/NF-002).
Descends product **PRD-013**.

**Relations** (prose — no structural slice-relation surface in v1; IMP-016):
- *Realises* — SPEC-002 (this is its observe half).
- *Governed by* — ADR-003, ADR-009.
- *Descends product* — PRD-013.
- *Sibling / shared seam* — **SL-040** (RV review-ledger kind). RV is another
  first-class numbered **record kind** built end-to-end; its scope items are this
  slice's prior art and **must be reused, not re-implemented** (no parallel impl):
  - SL-040 #2 = the authored-entity-wiring seam (`integrity::KINDS` row, manifest
    dir, gitignore negation, render/`show`) REC rides in P1
    (`mem.pattern.install.authored-entity-wiring`).
  - SL-040 #3/#6 = the outbound-edge + reverse close-gate-as-corpus-scan pattern
    (no reverse index) that the *Slice B* closure gate will reuse.
  If SL-040 lands first, REC wiring follows its pattern verbatim.

## Scope & Objectives

Build the observe substrate as four phases (P1–P4), ending green with the two-tier
separation proven structurally.

- **P1 — REC entity kind** (D1, REQ-108, H2, App. A). New first-class numbered kind
  `REC-NNN`, own corpus `.doctrine/rec/NNN/` (`rec-NNN.toml` + `rec-NNN.md`).
  `rec-NNN.toml`: `status_deltas = [(requirement, from, to)]`, `move ∈
  {accept,revise,redesign}`, `evidence_refs`, **optional** `owning_slice`, optional
  `decision_ref → DEC`. Rides the standard wiring seam (SL-040 #2); `Kind` is data,
  not a trait. Scaffold/show/list/validate registration. OQ-1 (alias/symlink
  convention) settles here.
- **P2 — Coverage substrate** (D3, REQ-109, NF-001 store-separation). Mode-
  discriminated entries `(requirement, contributing_change, mode ∈ {VT,VA,VH},
  status ∈ {planned,in-progress,verified,failed,blocked}, git_anchor,
  [attested_date])`. Stored **slice-side** so several changes touching one
  requirement compose with no clobber; stored **independently** of authored status.
- **P3 — Composite view + drift surfacer** (D4, D6, REQ-110, REQ-111, NF-001). A
  **derived** per-requirement composite (pure fold over entries across changes,
  computed on read, never stored) and a **derived** drift read (authored status vs
  composite → a prompt, never a write to authored truth). Deterministic fold (no
  clock/RNG/map-order; pure/imperative split). OQ-3 → v1 surfaces all entries, the
  writer judges precedence later.
- **P4 — VH/VA staleness decay** (D5, REQ-115, H1). Wire VH/VA coverage
  attestations onto the existing memory git-anchor seam (`src/git.rs`): carry an
  anchor, flag stale when touched code moves past it, **surface — never
  auto-demote**. Reuse the SL-007/008 verify/staleness machinery, no parallel impl.

**Closure intent.** Evidence is recordable + readable; composite coverage and drift
compute deterministically; staleness surfaces; and the acceptance proof for NF-001
is **structural** — no function anywhere maps coverage → authored status; the two
occupy distinct stores. VT entries record last-green ref (contracts deferred).

## Non-Goals

- **The reconcile writer, the write seam, the closure gate** — Slice B. This slice
  has **no write path to authored requirement or spec truth** (that is the point of
  NF-001); it only stores evidence and derives reads. Building any authored-truth
  write here would violate the two-tier line.
- **`spec req status` / spec-truth-revise verbs** — Slice B P1 (R1, pre-decided).
- **PRD-010 `knowledge_record`** — forward dep; REC owns its evidence sub-structure
  inline, lifted to a shared type when knowledge_record lands (OQ-2/H4).
- **Contracts / live VT re-derivation** — deferred (ADR-003 §11); the engine must
  not depend on them.
- **Drift Ledger** (mass reconciliation) and the **RV review-ledger** itself
  (SL-040) — sibling families, out of scope.
- **Composite precedence rules** (OQ-3) — later refinement; v1 surfaces all.

## Summary

Builds SPEC-002's observe substrate: the REC record kind, the slice-side coverage
store, the derived composite/drift reads, and VH/VA staleness decay — the two-tier
truth machinery with the no-derivation line proven structurally. Dependent
follow-on **Slice B** adds the reconcile writer, the requirement-status/spec-truth
write seam, and the closure gate.

## Follow-Ups

- **Slice B** (reconcile + close) — depends on this slice's substrate shape; scaffold
  after this design locks so B's write-seam design sees the coverage/REC shapes.
- **R2 — cross-slice composite fan-in.** Coverage is slice-side (D3) but a
  requirement composes across N slices (REQ-110). How the derived reader enumerates
  entries across slices is unspecified → resolve in P3 design.
- **R3 / OQ-1 — REC alias/symlink convention** (mirror `mem.<key>` / `nnn-slug`) →
  P1 design.
- **OQ-2** — if knowledge_record lands first, REC consumes its evidence type instead
  of owning it; neither forks (H4).
