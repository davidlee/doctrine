# Dispatch orchestrator state-visibility verbs

## Context

The `/dispatch` + `/worktree` orchestrator surface is the most complex workflow in
the platform, and the RFC-011 token-efficiency benchmark (case notes,
`.doctrine/rfc/011/2026-07-02.case-notes.md`) found its dominant token sinks are
**coordination-state visibility** problems, not command-shape problems. IDE-027
analysed the fix and split it into a "now" half (CLI-first query/reconcile verbs,
topology-independent, all-arm benefit) and a deferred half (an MCP projection,
gated on RFC-005 topology settling). **This slice is IDE-027's "now" half.**

The verbs are additive CLI surface that any orchestrator arm (claude, codex, pi)
benefits from — the MCP projection is out of scope and deferred. They are also
the dependency for that later MCP work.

## Scope & Objectives

Four orchestrator-facing verbs. All read-mostly; exactly one mutation (reconcile).
All operate on **doctrine-native concepts** (worktrees, slice phase-state,
selectors) — never host-project build conventions (POL-002; see Non-Goals).

1. **Worktree inventory / provenance** (read-only) — enumerate the project's linked
   worktrees with their doctrine provenance (which slice / branch / coordination-vs-
   fork role). Fixes the audit-time "read the main tree, conclude nothing to audit"
   blindspot when the implementation lives on a dispatch worktree (case notes,
   worktree-discovery blindspot). No existing coverage.

2. **Phase-status query** (read-only) — report which tree's **runtime** phase-state
   is canonical and the delta between trees (the `2/5`-primary vs `4/4`-coord
   split-brain, case-notes sink #1). Runtime-tier only — distinct from IMP-174's
   authored-tier divergence.

3. **Phase-status reconcile** (the one write) — pull phase-completion from the
   canonical source (coordination tree) into the primary tree before
   `prepare-review`, so the pre-audit rollup is coherent. **Coordination point:**
   overlaps IMP-174 in spirit (which-side-is-truth), but on a different tier —
   IMP-174 is authored `.doctrine/**` fork-vs-edge divergence; this is runtime
   phase-state. Design/inquisition must confirm these are complementary (runtime
   reconcile here, authored close-loop guidance there) and not a parallel
   implementation.

4. **Selector doctor** (read-only diagnostic) — surface selector health for a slice
   (stale / unmatched-glob / empty-fence conditions). Doctrine-native, POL-002-safe.
   **Adjacency:** SL-180 (selector-conformance hardening: design-time dry-run +
   import-belt scope-creep refusal) is in design and touches selector validation —
   design must carve the boundary (diagnostic-report here vs conformance-gate there)
   to avoid duplication.

**Objective:** cut the orchestrator's per-occurrence investigative call count on the
named sinks from ~4–6 down to one queryable verb, without adding host-coupling.

## Non-Goals

- **The MCP projection / tool descriptions / funnel-beat wrapping** — deferred per
  IDE-027's timing gate (RFC-005 topology + SL-180/181/189 close + IMP-171/D6
  arm-record convergence). This slice is CLI-only.
- **Binary-freshness assert** and **provisioning/artifact check** — excluded on
  **POL-002** (platform independence): both encode host-project build conventions (a
  Rust binary on PATH; a RustEmbed `dist/`-style embedded root). A platform-neutral
  framework must not bake these in. Provisioning divergence is IDE-017's axis.
- **Authored-tier split-brain** close-loop guidance — IMP-174 owns it.
- **Any change to dispatch topology, fork/import modes, spawn choreography, or the
  funnel cadence** — those verbs are actively churning (RFC-005; SL-180/181/185/187/189)
  and are explicitly not touched here.

## Summary

<!-- filled at close -->

## Follow-Ups

<!-- filled as they surface -->
