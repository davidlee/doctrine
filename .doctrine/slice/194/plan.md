# Implementation Plan SL-194: Actionability interestingness findings

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

A text-first probe (RFC-007 workstream 2 "Legible", originates IMP-241): pure
finding-functions over `PriorityGraph`, surfaced through a new `findings` verb,
one line per finding. The slice validates whether *findings-over-picture* reads
more useful than the flat `next`/`survey` list **before** any rendering-policy or
visualisation follow-on is committed (design D1).

Two phases, each ending **runnable + green** — the probe wants an early verdict
on the core before investing in the β-family, which is the differentiated value a
flat list cannot show but is the more speculative, data-starved half.

## Sequencing & Rationale

### Why two phases, in this order

The catalogue splits cleanly along a substrate seam:

- **PHASE-01 (core)** needs only what exists on a single built graph today —
  structure (dep/seq overlays), the score/base maps, and cordage provenance. It
  ships a runnable verb and a judgeable verdict with zero new perturbation
  machinery.
- **PHASE-02 (β-family)** needs the *rebuild seam* — three graph builds at swept
  `estimate.skew` — and only fires when authored estimates carry intervals
  (starved-until-populated, R1). It is both more expensive and more speculative,
  so it is sequenced second and gated on the PHASE-01 verdict (`VH-1`).

Selling the core first means a failed probe costs one phase, not two.

### PHASE-01 internal ordering

Two **behaviour-preserving extractions land first**, because everything else
depends on them and they carry the strictest gate:

1. `build_from_with_cfg` — `build_from` currently loads config internally
   (`graph.rs`), so β cannot be injected. Lift the single config load to a
   parameter, threaded to *both* `base_score` and `consequence_post_pass` (else a
   β sweep would perturb base while silently using default consequence coeffs —
   the seam the external review flagged). `build_from` delegates; existing callers
   are byte-identical.
2. `order.rs` — `frontier_order` + `surviving_seq_predecessors` are private pure
   fns in `surface.rs`; the detectors need the same linear orders. Extract them so
   `surface::next` and the detectors share one implementation. `next` stays
   byte-identical.

Both are proven by the **existing `graph`/`surface` suites passing UNMODIFIED**
plus a fresh equivalence test — the behaviour-preservation gate (AGENTS.md). Only
then does the new surface get built on top: `findings.rs` (the pure catalogue),
`surface::findings` (the impure shell that owns disk), and the `findings` verb.

### Detector correctness carried from the reviews

The plan bakes in the disposition of both adversarial passes so execution can't
regress them:

- **Direction** — Fork/GatingFanOut read `blocking` (out; what settling the hub
  unblocks), Join reads `blocked_by` (in; its prerequisites). The dep overlay
  stores `needs` prereq→dependent (B→A flip), so these accessors are correct as-is
  — no re-derived edge logic.
- **Terminal arms + precedence** (F5/F6) — `blocking` does not filter terminal
  successors, so Fork/GatingFanOut filter arms to non-terminal; a gating-class hub
  with fan-out reports as GatingFanOut only.
- **ValueInversion compares `base`, not `score`** (F-ext-2) — `score(blocker)`
  folds leverage from the very dependent it gates, which would make a score-based
  test nearly dead. `base` is the intrinsic worth; `base` (not `value_dim`) so a
  high-risk low-value blocker isn't a false inversion.
- **Provenance dedupe** (F-ext-3) — `channels::evicted_seq_edges` is node-local
  (returns each edge for both endpoints); the detector dedupes globally.

### PHASE-02 β-family

`beta_endpoints` (shell) pre-builds `lo`/`hi` over the **same scan** at
`skew ∈ {0,1}` and hands raw graphs to `detect`; the detectors derive frontier
orders via `order.rs` and stay pure (the purity boundary — shell owns disk,
detectors read pre-built graphs). Two carried limitations, both accepted at probe
grade and documented in the design's risks:

- **R4** — the sweep re-reads dep/seq per build, so a concurrent mid-sweep commit
  could misattribute topology churn as β instability. Quiescent-tree precondition;
  IMP-243 retires it structurally later.
- **R5** — the `{0,1}` endpoint sample detects only endpoint-contested orderings;
  interior flips that share a sign at both endpoints are a known false-negative.
  OrderInstability's claim is narrowed accordingly; the finer flip-β grid is the
  deferred refinement.

## Notes

- `specs`/`requirements` stay empty (no registry in v1); SL-194's governing
  relations live as `doctrine link` edges, not typed plan keys.
- Closure hinges on the `VH-1` probe verdicts, not on the detectors alone — the
  slice's question is "does this read more useful?", answered against the live
  corpus, recorded in the design.
