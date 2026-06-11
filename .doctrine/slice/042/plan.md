# Implementation Plan SL-042: Reconciliation observe substrate (SPEC-002 A)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases build SPEC-002's **observe** substrate, mirroring `design.md` §1's
P1–P4 one-to-one. The arc is **store → read → derive → decay**: stand up the
record kind, store evidence beside it, derive the composite and drift reads over
that evidence, then wire staleness decay onto the reads. Each phase ends green and
realises a named FR/NF requirement, so the plan needs no re-decision — the design
is locked (two external passes integrated). The reconcile *writer*, the
spec-truth write seam, and the closure gate are the dependent **Slice B**,
deliberately out of scope: SL-042 has **no write path** to authored truth, which
is the structural reason NF-001 holds here.

## Sequencing & Rationale

The order is forced by data dependency, not preference:

- **PHASE-01 (REC kind) first** because it is the only phase touching the shared
  numbered-kind machinery (`integrity::KINDS`, `meta::read_id`, manifest +
  gitignore wiring), and because PHASE-02's coverage entries are *cited* by REC
  `evidence_refs` — the kind must exist before the thing that points into it.
  This is also where the live coordination risk lives (**R-a**): SL-040 edits the
  same `meta.rs`/`integrity.rs`. SL-040's phases now read 6/6 in the state tree,
  so the primary path — ride its landed seam verbatim — holds; the fallback (land
  the small `read_id` seam here, SL-040 rebases) stays in reserve if a merge
  conflict surfaces. Sequencing P1 first concentrates that risk in one phase.

- **PHASE-02 (coverage substrate) second** — the store the later folds read.
  It is where the SL-028 `CoverageStatus` stub becomes a real consumer, so the
  `expect(dead_code)` removal (**R-b**) lands here, not before. The 4-tuple entry
  key `(slice, requirement, contributing_change, mode)` is fixed here because both
  fan-in identity (PHASE-03) and citation (PHASE-01 `evidence_refs`) depend on it;
  getting it wrong is the X-2 collision the design closed. Authored-tier residency
  is a phase-local proof obligation (the `check-ignore` VT), not an assumption.

- **PHASE-03 (composite + drift) third** — pure folds over what PHASE-02 stores,
  plus the conservative total coherence predicate. It is sequenced after the store
  exists but before staleness decay because the folds are written to consume an
  **already-resolved** `IsStale` (F1: staleness is a git read, resolved in the
  shell, never in the pure fold). The predicate is the design's most-scrutinised
  surface (the C-II mortal + X-1 totality hole); the verdict-matrix VT is its
  proof. The R2 perf spike rides here because the corpus scan it measures is born
  here — and it measures the two cost axes separately (X-4) so the staleness
  subprocesses do not mask the scan question.

- **PHASE-04 (staleness decay) last** because it is the only phase that *needs*
  PHASE-03's shell already resolving an `IsStale` per entry — it makes that
  resolution real against `src/git.rs`. Its first task is the **R-e** hypothesis
  check: the design *inspected* `git::commits_touching` and believes the
  granularity fits, but "reuse unchanged" is a hypothesis until the wiring proves
  it. Widen at the leaf if it does not fit; never fork (a fork is the parallel
  impl the house rules forbid). PHASE-02–04 are independent of SL-040, so only
  PHASE-01 carries the coordination dependency.

## Notes

- **Pure/imperative split is load-bearing** (ADR-001, F5): `coverage.rs` is the
  pure leaf (`CoverageEntry`, `composite`, `drift`); `scan_coverage` and the
  staleness resolution are the impure shell above it. No clock/RNG/git/disk in the
  fold — staleness arrives as a resolved `IsStale` input.
- **NF-001 is proven structurally, not by a test of absence** (F4/X-3): the
  checkable obligations are the `Verdict` return type (no truth-write in the
  signature), the SL-028 two-enum non-reference (compile/grep), and the distinct
  stores. The coverage→status-writer *import-edge* guard is vacuous here — no
  writer exists to wall off — so its enforcement lands with Slice B.
- **Deferred, captured, not forgotten** (per `defer-needs-backlog-before-close`):
  OQ-2 (knowledge_record evidence-type sequencing), OQ-3 (composite precedence —
  `Indeterminate` reads as drift at the gate for v1), and the two conditioned perf
  backlog triggers (reverse-index on a scan-axis cliff, batching on a
  staleness-axis cliff) are recorded as the spike runs, not after close.
- **`audit.md` is hand-made** — no `slice audit` scaffold exists yet (known CLI
  gap); the close path is `/audit` → reconcile → `/close`.
