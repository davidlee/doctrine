# Formal VT verification: executable check + coverage record surface (SPEC-002 test-run surface)

## Context

**SPEC-002** (Requirement Reconciliation Engine, descends **PRD-013**) shipped its
observe→reconcile→close machinery across **SL-042** (observe substrate: the
two-tier coverage store, derived composite + drift reads, git-anchor staleness)
and **SL-044** (reconcile writer + closure gate). Both deliberately deferred one
half of the design — the **test-run surface**.

Today a coverage evidence entry (`src/coverage.rs`, `CoverageEntry`) is
`(slice, requirement, contributing_change, mode∈{VT,VA,VH}, status, git_anchor,
touched_paths[], attested_date?)`. Two consequences motivate this slice:

1. **No production write path.** Coverage entries are created only by test
   helpers (`fs::write` of `[[entry]]` TOML); the design's "coverage is written
   at audit" surface was never built. In production, `coverage.toml` is
   hand-authored. SPEC-002 itself was made to show "met" by hand-authoring
   entries + running `reconcile` — the "manual backfill" the user observed.

2. **VT is attestation, not verification.** A VT ("by test") entry carries only a
   `git_anchor` SHA + `touched_paths` + a hand-set `status`. It names no test and
   nothing re-runs it; `Verified` is asserted, not checked. SPEC-002's "Contracts
   deferred" concern states VT entries "*become continuously re-derived when a
   contracts/test-run surface lands*" and that "the engine must not depend on
   contracts existing." This slice builds that **test-run surface** so VT becomes
   *checked against reality* — actually verifying, not minting paperwork.

**Terminology guard.** SPEC-002 "contracts" denotes a *distinct, separate*
deferral — deterministic per-file documentation of function signatures et al. It
is **not** this work and is **not** a dependency: VT execution is built
standalone, per the spec's own "must not depend on contracts existing" line.

**Governing decisions** (no re-deciding here; tensions → `/consult`): SPEC-002
(two-tier purity, freshness physics, NF-001), ADR-003 (canonical loop,
explicit-authorship-not-derivation), ADR-009 (FSM, conduct axis). The hard line
holds: this slice writes only the **observed** tier; `reconcile` remains the
**sole** author of authored requirement/spec truth (NF-001 / `REQ-105`).

**Relations** (prose — no structural slice-relation surface in v1; IMP-016):
- *Realises / completes* — SPEC-002 (its deferred test-run surface half).
- *Descends product* — PRD-013.
- *Governed by* — ADR-003, ADR-009.
- *Sibling / shared seam (reuse, do not re-implement)* — SL-042 (coverage store,
  `src/coverage.rs`; git-anchor staleness via `src/git.rs`), SL-044 (reconcile
  writer / closure gate; observed-tier discipline).
- *Reuses config seam* — the root `doctrine.toml` parse pattern (`src/conduct.rs`,
  pure parse + thin-shell read) is the prior art for the project verification
  command contract — extend it, no parallel config reader.

## Scope & Objectives

One coherent capability — **make VT coverage executable and recordable** — in
three stacked parts:

1. **Coverage record/write surface.** The missing production path to author a
   coverage entry (the deferred "written at audit" verb). Captures the git anchor
   + touched paths at record time via the existing `src/git.rs` born-frame seam;
   honours the no-clobber `upsert` 4-tuple-key semantics already in
   `src/coverage.rs`. Writes only the observed tier.

2. **Executable check identity on VT entries.** A VT entry gains a
   project-agnostic *runnable* identity (e.g. a verification command / token the
   project resolves), distinct from VA/VH which stay point-in-time attestations.
   Exact field shape is a `/design` decision.

3. **The verifier (the test-run surface).** A thin impure shell that *runs* the
   resolved check and **derives the observed `CoverageStatus`** from the real
   outcome (exit-0 ⇒ `Verified`; non-zero ⇒ `Failed`; unobtainable/no-command ⇒
   `Blocked`, never defaulted to `Verified` — SPEC-002 failure-mode line). The
   status-derivation fold stays **pure**; only the subprocess run is impure
   (mirrors `src/git.rs`). This makes VT *continuously re-derivable*: re-running
   reproduces the status from current reality rather than trusting a stored SHA.

**Project-agnostic contract.** The framework defines a generic verification
command contract (exit-code semantics); the *concrete* command is project
configuration in root `doctrine.toml`, resolved at run time. Doctrine's own repo
wires it to `cargo test <filter>` (or the `just` gate) — but nothing in the
engine is Rust- or cargo-specific. Pattern mirrors the harness-agnostic dispatch
posture (ADR-011 spirit).

**Closure intent.** "Done" = a VT entry can be recorded with an executable
identity, re-run via the new verifier, and its observed status reflect the actual
run; the existing `coverage` / drift reads consume the result unchanged; the
two-tier wall and `reconcile`-sole-writer invariant remain provably intact
(behaviour-preservation: SL-042/044 suites stay green unchanged). Verified by
test (VT) end-to-end CLI goldens for the record + run verbs, plus a derivation
test asserting status follows the real exit code, plus an NF-001 guard that the
verifier touches only the observed tier.

## Non-Goals

- **Historical backfill.** Recording/running evidence across the existing corpus
  to clear "unmet" status is *separate follow-on* (its own slice/backlog), not
  this slice. This slice ships the capability only.
- **"Contracts"** in the SPEC-002 sense (deterministic per-file signature
  documentation) — a distinct deferral; explicitly *not* built and *not* depended
  on.
- **Running VA / VH.** Agent/human attestations remain point-in-time and decay
  via the existing staleness seam; only VT is mechanically re-derivable.
- **Touching the authored tier.** No change to `reconcile`, requirement-status
  writes, or the closure gate's authoring authority. The verifier writes observed
  coverage only.
- **Composite precedence / multi-mode collapse** (OQ-3) and **scan/staleness perf
  hardening** (RSK-006) — pre-existing deferrals, untouched.

## Summary

Build the deferred SPEC-002 **test-run surface**: a coverage record/write verb, a
runnable identity on VT entries, and an impure verifier that derives observed
coverage status from a real command run — turning VT from a hand-set attestation
into continuously re-derived verification, while preserving the two-tier wall and
`reconcile`-sole-writer invariant.

## Follow-Ups

- Historical corpus backfill (clear "unmet" using this machinery) — capture as a
  follow-on slice/backlog item at close.
- Re-derive-on-read / scheduled re-verification cadence (when to re-run VT
  automatically) — likely a later concern; surface in `/design` if it pulls scope.
- "Contracts" (per-file signature docs) deferral remains open and independent
  (relates to IMP-027 deferred-doc-canon home).
