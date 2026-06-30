# Implementation Plan SL-179: Closure gate hardens on live Failed coverage cell; close the forget evidence-erasure leak

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases close two evidence-erasure leaks in the closure gate (RSK-008) and the
two adjacent holes the codex pass surfaced. The shape: **governance first** (a REV
authorizes the narrowed gate semantics), then the **type** that encodes the
Failed/Blocked distinction, then the **gate** that acts on it, then the **forget**
guard. Phases 02→03 are strictly ordered (the gate consumes the new verdict
variants and `has_fresh_vh`); phase 04 is file-disjoint and depends only on the REV.

## Sequencing & Rationale

**PHASE-01 (governance) lands first.** Canon forbids code ahead of governance, and
the slice dogfoods its own hardened gate — so the amended D8/REQ-113 must exist
before the code that enforces it, and SL-179's `[gate].extra_reqs` must be seeded so
its own close is answerable (else the dogfood VA passes vacuously — codex M8). The
REV is a spec amendment: human-approved (VH-1), agent-checked for consistency
(VA-1). No VT — a spec doc is not test-judged.

**PHASE-02 (verdict model) before PHASE-03 (gate).** The gate's new behaviour reads
`ObservedFailure` / `ObservedBlocked` and `has_fresh_vh()` — those must exist first.
This phase is otherwise behaviour-preserving: the only observable change is the
named reason split (labels + prompt register + view), so the existing drift/label
goldens update in lockstep (EX-5) and the refuse/coherent verdicts are unchanged.
Keeping the split isolated here means PHASE-03's diff is pure gate-policy.

**PHASE-03 (gate) is the slice's substance.** All four policy strands live in
`undischarged_drift` (codex M10 — `rec_discharges` stays the unchanged 3-clause
predicate, so the gate policy is in one place and the bool predicate stays pure,
NF-001): ObservedFailure hard-refuse, the VH-Blocked bar (D3), the M7 empty-keys
guard, and the D4 withdrawal-act check. The behaviour-preservation proof is EX-5 —
the existing *lag* discharge tests (VT-4/5/6) stay green untouched, because the
status-lag path is deliberately unchanged; no existing test discharges a Failed
cell (verified in the design's adversarial F2), so the hard-refuse adds tests
rather than flipping them. VA-2 re-pins the NF-001 wall after the reason split.

**PHASE-04 (forget guard) is independent.** It only inspects `CoverageStatus`, not
the verdict model, so it needs only the REV (the erasure prohibition it enforces).
The guard is atomic inside `forget` — the current remove-then-return contract can't
be post-checked, so the refusal must precede the mutation (`ForgetOutcome::Refused`,
no save). /phase-plan may run it in parallel with PHASE-03 (file-disjoint:
`coverage_store.rs` vs `slice.rs`) or after; the plan orders it last for clean
serial commits.

## Notes

- **Deferred (codex pass):** RSK-012 (gate-set scope is per-slice — a foreign Failed
  req can be omitted by not declaring it; no *silent* leak) and RSK-013
  (`scan_coverage` silently skips malformed coverage — closure needs a strict
  fail-closed scan mode). Both out of RSK-008's scope; tracked, not abandoned.
- **Phase ids immutable** — if the REV (PHASE-01) mints companion requirements that
  reshape later criteria, edits append; PHASE-NN never renumbers.
- **behaviour-preservation gate** binds PHASE-02/03 (shared entity machinery): the
  unchanged lag-discharge + drift-coherent suites are the proof.

## Risks & mitigations (critical pass)

- **PHASE-01 prose-vs-requirement (resolved-enough for /phase-plan).** REQ-113
  (FR-006, active) and the `revision` vehicle are confirmed. D8 is spec *prose* (a
  REV `prose` row, surfaced for manual handling); REQ-113 is a *modify* row. Open at
  phase-plan time: whether the REV amends REQ-113's text in place or mints a
  companion requirement for Failed-un-acceptability — both are supported; the
  former is lighter and preferred unless the clause set wants its own id.
- **PHASE-03 is the largest phase** (four policy strands in `undischarged_drift`).
  Cohesive (one function, one concern — discharge policy) so kept whole for a single
  clean diff; /phase-plan should split only if the task breakdown balloons.
- **D4 changes the Coherent path.** The gate currently `continue`s on `Coherent`; D4
  means a withdrawn req must still be coverage-scanned for a live contradiction —
  a small, bounded addition, reusing `recs_owned_by` + `evidence_ref` for the
  withdrawal-REC lookup.
- **VA-1 is the heaviest verification** (candidate binary refuses SL-179's own close
  over a seeded cell). Strongest evidence; operationally an agent-run integration
  check, appropriately VA not VT.
