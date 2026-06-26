# Implementation Plan SL-138: Relation-transitive walk for inspect — analogous to blockers --transitive

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, one per architectural layer (ADR-001: leaf ← engine ← command). The
work is naturally bottom-up: the CLI surface (PHASE-03) renders what the engine
(PHASE-02) computes, which walks the primitive the leaf (PHASE-01) provides. Each
phase ends green and is **file-disjoint** from the others — PHASE-01 lives entirely
in `crates/cordage/`, PHASE-02 in `src/relation_graph.rs`, PHASE-03 in
`src/commands/{inspect,cli}.rs` — so a phase can be reviewed and landed in
isolation, and a later phase never reopens an earlier file.

`design.md` §5 is the authoritative reference for every signature and the output
contract; this plan does not restate it.

## Sequencing & Rationale

**PHASE-01 (cordage primitive) first because everything depends on it and nothing
depends on the rest.** The depth cap and truncation indicator cannot be expressed
over today's flat `reachable` (it carries no depth), so the primitive must exist
before the engine can walk with a bound. The load-bearing risk is here, not in the
CLI: re-expressing the existing `reachable` over `reachable_bounded` must be
behaviour-identical (the blockers/inspect suites are the proof, VT-1), and the
`truncated` flag rests on a BFS min-depth ordering argument (VT-3). Landing this as
an isolated, fully-tested cordage change keeps that risk off the later phases. The
codex inquisition's blocker (C1) lives here — the surface is a public `Graph`
method, not a free function over crate-private indices.

**PHASE-02 (engine) second — the query + renders over the now-stable primitive.**
It owns the two corrections the inquisition forced: the table-derived overlay-backed
predicate (C2 — the no-overlay set is `{contextualizes, drift, decision_ref}`, read
from `RELATION_RULES`, never hardcoded) and the pinned output contract (C4). Kept
separate from the CLI so the transitive logic is unit-testable at the engine layer
(`transitive_from` returns a struct) before any golden-tested command surface —
the direction semantics, per-label sectioning, and role-collapse (F3) are proven
here, not through the CLI.

**PHASE-03 (command) last — flags, parsing, routing, e2e.** Depends on PHASE-02's
view + renders. Carries the memory-ref gate (F2: `--transitive` on a `mem_*` ref is
rejected before the memory early-return), the `--max-depth` parse (absent→5,
`0`/`all`→unbounded), and the clap `requires`/`default_value_t` interaction (VT-3).
The regression guard — bare `inspect <ID>` byte-unchanged — is a PHASE-03 exit
criterion because this is the phase that touches `run_inspect`'s branch.

## Notes

- **Conduct.** Slice is `self/gate` — solo execution, gated. Phases are
  file-disjoint and serial (each `after` the prior); no parallel dispatch needed.
- **Behaviour-preservation gate.** PHASE-01 changes shared cordage machinery; the
  existing suites must stay green unchanged (AGENTS.md gate). This is the single
  most important check in the slice.
- **Out of scope (design §2).** No tree/path render (the `depths` field ships
  unconsumed for a future view), no per-role transitive `references`, no graph
  export, no memory-graph transitivity.
