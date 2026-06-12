# Implementation Plan SL-047: Cross-kind actionable survey/next/explain/blockers CLI

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-047 is slice 2 of 3 in the graph-relations work: a thin **policy + surface**
layer over the graph spine SL-046 ships. It introduces **no new graph mechanism**
(design §1/§4) — every surface composes `cordage` + the `backlog_order` dep/seq
pattern + SL-046's all-kind scan. The slice's novelty is *policy* (the OQ-8
partition, the channels, the two sort keys) and *surface* (`survey` / `next` /
`blockers` / `explain`, plus the `inspect` actionability extension).

The phases follow the design's three natural seams (§5.1 affected surface), which
also happen to be a clean dependency chain and a coupling/cohesion boundary:

- **PHASE-01 — the adapter.** The impure shell: reuse the scan, build the third
  graph, read node attributes, tally consequence. Everything that touches disk and
  cordage construction lives here and nowhere else.
- **PHASE-02 — the pure core.** The partition table and the channel synthesis —
  the slice's correctness heart, exercised by unit tests over a constructed graph
  with no I/O.
- **PHASE-03 — the surfaces.** Structured reasons, render, the four verbs, and the
  black-box CLI goldens that prove the end-to-end behaviour.

This ordering puts the impurity at the bottom, the pure policy in the middle tested
in isolation, and the rendering/CLI at the top — honouring the pure/imperative
split (§3) and keeping each phase's test strategy distinct (build-integration →
pure unit → black-box golden).

## Sequencing & Rationale

**Why adapter first (PHASE-01).** Both the partition and the channels read a
`PriorityGraph`; nothing pure can be tested until that type exists and builds. The
adapter is also where the **SL-046 coordination** lands. DD-1 sequenced SL-047's
implementation *after* SL-046 lands, and it has — `relation_graph` (the all-kind
scan), `projection`, and `inspect` are on `main`. But SL-046 landed **without** a
reusable raw-scan seam: `build_relation_graph` is private and `outbound_for` is the
only `pub(crate)` primitive. So D5's "scan seam fed into SL-046" did not
materialise as a callable seam, and PHASE-01 carries the **D5 fallback** — extract
the shared all-kind scan into a `pub(crate)` seam as a *pure refactor* behind the
behaviour-preservation gate, leaving `inspect` byte-identical. This is exactly the
contingency the design anticipated; it is bounded refactor work, not new design.

The consequence pre-pass is in PHASE-01 deliberately (I2): tallying inbound over the
work/lineage label subset needs only the scanned outbound edges, computed *before*
any graph is built — that breaks the mint-order ↔ consequence ↔ graph cycle and
feeds the deterministic `(consequence desc, canonical-id asc)` mint order. The
`reviews`/`owning_slice` bookkeeping labels are excluded from the tally (Charge V).

**Why the pure core second (PHASE-02).** With the adapter producing a graph, the
partition and channels become pure functions over it — no disk, no clock. This is
where D12's `actionable = eligible ∧ ¬blocked` synthesis and the I1
direct-blocker-suffices invariant are proven, and where the partition **drift
canary** guards the OQ-8 table against status-enum drift. Splitting this from the
adapter keeps the heart of the slice testable without fixtures on disk and matches
the cohesion boundary in §5.3 (partition owns the policy table, channels own the
synthesis, graph owns the adapter).

Three inquisition-settled facts are bound into this phase's criteria and must not be
relitigated: **RV is admitted to v1** — an `Active` RV is `eligible`, its status
read via `review::derived_status` over the *authored* finding ledger (Charge I);
**REC** is status-less → non-eligible via the status-less path, no diagnostic
(DD-4); and the **slice** kind is stringly-status, so its canary binds to the
ADR-009 lifecycle status set rather than a closed enum (Charge VII).

**Why surfaces last (PHASE-03).** The render layer is the source-of-truth inversion
(REQ-072): rows and prose are produced *from* the structured `ReasonKind`, never the
reverse. That only makes sense once the channels that produce those reasons exist.
The four verbs and the `inspect` extension sit on top, and the black-box CLI goldens
over a seeded multi-kind corpus drive the design §9 validation list — the
`survey`/`next` **divergence** test (D10), the blocking-display depth test (D11),
the cycle degrade (REQ-076), determinism (REQ-077), and the cross-kind actionability
proof. The phase ends on the full behaviour-preservation gate.

**The v1 honest contract (DD-2 / Charge II).** The dep/seq engine is built
**kind-agnostic** but only backlog authors `needs`/`after` in v1 (non-backlog kinds
cannot author a `dep` edge — `parse_ref → ItemId`). So `¬blocked` is real for
backlog and vacuous for other kinds, whose actionability reduces to `eligible`. The
cross-kind half that every PRD-011 / SPEC-001 acceptance gate rests on is the
**status** half, which is fully cross-kind here. The dep-blocked verification fixture
is therefore **backlog-scoped** by design — PHASE-03's VT-3 must not assert a
non-backlog dep-blocked item. Cross-kind blocking lights up for free when IMP-033
authors edges; no change here.

## Notes

- **Determinism & lint** (§3): no clock/RNG/map-order; `BTreeMap`/`BTreeSet` not
  Hash; no `as` casts, indexing-slicing, or `print_stdout`. `just check` before
  every commit; the gate runs plain `cargo clippy` — **not** `--all-targets`.
- **Behaviour-preservation** is a verification row on every phase that touches shared
  machinery (the scan-seam extraction in PHASE-01, the final gate in PHASE-03):
  `backlog_order` + `cordage` suites green **unchanged**, `backlog order` output
  byte-identical.
- **Deferred, captured** (out of v1, design §6 / scope Follow-Ups): cross-kind
  dep/seq capture (IMP-033 + governance), slice phase-rollup actionability,
  persisted policy-stamped cache, coverage-driven requirement actionability,
  authored-priority scalar (PRD-009). None block this slice; the engine is shaped so
  each lights up additively.
