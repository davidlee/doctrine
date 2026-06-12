# Implementation Plan SL-044: Reconcile writer + closure gate (SPEC-002 B)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-044 builds SPEC-002's **reconcile + close** half on top of SL-042's observe
substrate: the authored-truth write seam, the sole-author reconcile writer, and the
closure gate that refuses unreconciled drift. The design (`design.md`) is locked —
all four OPEN questions resolved (D-B5..D-B8) and a codex adversarial pass (3
blockers + 4 majors) integrated. This plan splits the work along the design's own
B·P1 → B·P2 → B·P3 boundary, one phase each, because the phases form a strict
**build-up dependency chain**: the writer needs the setter, the gate needs the RECs
the writer emits.

## Sequencing & Rationale

**Why three phases, in this order.** The dependency is mechanical, not stylistic:

- **PHASE-01 (B·P1 — write seam)** is the primitive both reconcile moves stand on.
  Until `spec req status` can transition authored truth edit-preservingly, the writer
  has nothing to call. It lands first and alone because it carries no reconcile logic
  — it is the `set_slice_status` pattern mirrored onto `ReqStatus`, with the one
  deliberate divergence (D-B6): a **free any→any** setter, not an FSM, because
  `revise` exists to move *any* direction (an ordered FSM refuses exactly the
  mis-claim corrections reconcile needs). Reusing `governance::set_status`'s shape
  (the adr free-setter precedent) over `set_slice_status`'s ordered shape is the
  no-parallel-implementation call. Smallest, most isolated, zero coverage in scope.

- **PHASE-02 (B·P2 — reconcile writer)** is the heart: the sole author, accept/
  revise/redesign, exactly one REC per act. It depends on PHASE-01's setter and on
  SL-042's reads. The **NF-001 wall is built here** because this is the first slice
  with a status-writer that also reads coverage — the seam where the no-derivation
  invariant must be proven. The wall is layered deliberately (D-B7): the design
  *rejected* both the import-edge ban and the return-type argument as false (a `match`
  over the verdict launders one and compiles), so the load-bearing mechanism is
  **signature isolation** — `select_status(to, prior)` whose parameter list excludes
  every coverage-derived type — backed by the verdict-consumed-by-prompt routing and
  the residual-site verdict-independence test. No single mechanism suffices; the
  combination is the guarantee. This phase's structural and behavioural VTs are the
  load-bearing proof, not an afterthought.

- **PHASE-03 (B·P3 — closure gate)** is the consumer: it scans the RECs PHASE-02
  emits, so it must come last. It extends the *existing* `run_status` close-gate shell
  — a second predicate beside the D-C9b RV-blocker scan on the same
  `crosses_closure_seam` — rather than a parallel gate (reuse, per design §3). Two
  genuinely new pieces of machinery surface here, both flagged in the design's risk
  register: a **slice-local coverage reader** (R-B4 — `scan_coverage` enumerates one
  req across slices, not one slice's reqs) with `key.slice == S` validation, and the
  **strengthened discharge predicate** (R-B3 — latest-REC + `move==accept` +
  evidence-coverage, so a stale affirm cannot excuse live drift). The reverse req→REC
  lookup stays an on-demand corpus scan, never a stored link (D-B3/ADR-004 anti-
  desync); perf escalation is explicitly RSK-006, not built blind here.

**The gate-scope declaration is a /plan deliverable (D-B5).** The closure gate's req
set includes a `declared` term — an authored, additive `[gate] extra_reqs` list in
`slice-044.toml`, set and peer-reviewed **at this stage, ahead of any REC**, because
the right drift scope is a per-slice risk judgement that belongs in a reviewable
authored artifact, not in gate code. For SL-044 itself the declared set is the
requirements it realises and strengthens — **REQ-112, REQ-113, REQ-114, REQ-116** —
so the slice's own closure gate (which runs the very machinery it builds) checks the
reqs this slice is answerable for. This declaration is authored into `slice-044.toml`
alongside this plan and is the peer-review surface; the gate can never check *less*
than `covered ∪ declared ∪ reconciled`.

**What is deliberately not here.** No `/reconcile` skill or audit/reconcile seam
disentanglement (IMP-008, downstream — this ships the CLI surface the skill drives).
No staged draft/approve Revision vehicle for prose (IDE-003); `revise` is a direct
structural `ReqStatus` write only. No composite precedence engine (OQ-3 — the writer
judges `Indeterminate` as drift in v1). No coverage-scan perf hardening (RSK-006 — a
decision input, conditioned follow-up, not built blind).

## Notes

- **Behaviour-preservation gate.** PHASE-03 extends the shared `run_status` shell and
  PHASE-01 rides the `spec req` tree — both touch shared machinery. The existing
  suites (slice-status close-gate, spec req add) must stay green unchanged; new
  behaviour arrives only through new tests.
- **Purity discipline holds across all three** (ADR-001, design §3): move
  classification, RecDoc composition, and the gate predicate are pure over resolved
  inputs; the only clock/git/disk lives in the thin shell — resolve every ref before
  the pure compare (`mem.pattern.safety.resolve-every-ref-before-pure-compare`).
- **Lint/format gate per house rules** — `cargo clippy` zero-warning bins/lib (not
  `--all-targets`), `just check` before every commit; conventional commits scoped
  `…(SL-044)`.
- Phase-plan each phase's runtime sheet with `/phase-plan` just before executing it;
  do not expand all three up front.
