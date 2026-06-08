# Implementation Plan SL-022: Technical-spec system support: descent, decomposition & integrity

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See doc/glossary.md § reference forms. -->

## Overview

Four phases build the tech-spec relational spine bottom-up, each ending green and
each a clean TDD unit. The ordering follows the data: first the fields exist at
rest (PHASE-01), then the registry can collect and resolve them (PHASE-02), then
the graph integrity over those edges (PHASE-03), then the one structural addition
— the severity tier — and closure (PHASE-04). Every phase is additive to the
SL-015 machinery; the behaviour-preservation gate (the SL-015 suites stay green)
is an exit condition throughout, with the single sanctioned change being REQ-084's
intended contract move (PRD-012 §6), not a divergence.

## Sequencing & Rationale

**PHASE-01 — at-rest first.** The two scalar fields are the foundation everything
else reads. Parsing and rendering them is fully testable with zero registry or
integrity risk, so it makes the cleanest first green. It also localises the only
mechanical edit to existing tests — the `None, None` constructor additions forced
by `Spec` deriving no `Default` (design finding C) — to one phase, so later phases
touch no existing `spec.rs` test. The `Some`-gating keeps existing render output
byte-identical, so the behaviour-preservation gate holds from the start.

**PHASE-02 — collect and resolve, before any graph.** The registry must see the
edges before it can check them. This phase adds the `product_specs` set and the
parent/descent edge collections, and — the correction the inquisition forced
(Charge I) — the new per-spec `spec-NNN.toml` parse in `build_registry`, which
parses no spec today. That parse widens the impure scan's error surface, so this
phase owns the Layer-C test that proves the new behaviour rather than assuming it
from green hand-built-registry units. The FK-resolution checks here are
flat set-membership only (descent and parent: clean / invalid-kind / dangling),
plus the REQ-084 interaction rewrite. No graph walk yet — kept separate so the
riskier traversal lands in isolation.

**PHASE-03 — integrity in isolation.** The decomposition tree is the one
graph-shaped concern and the locus of two inquisition findings, so it gets its own
phase. `parent_cycle` must report each cycle exactly once (the dedup that finding
F demands — the cycle-2/cycle-3 tests assert the *count*, not mere existence), and
the pre-parse `second_parent` guard must emit a *named* hard finding with a
non-zero exit (the synthesis of the User's rulings on finding D: structural
impossibility plus a named diagnostic, which promotes REQ-087 AC1 from deviation
to literal satisfaction). Isolating this phase keeps the traversal and the
raw-text guard from entangling the simpler FK work.

**PHASE-04 — the one structural addition, then close.** The severity split is the
sole departure from "additive checks," so it lands last, after every hard check is
proven. Deferring it keeps `validate()`'s signature — and therefore every SL-015
validate test — untouched through phases 01–03; the soft tier arrives only when
its single consumer (`descent_on_product`) needs it. Closure (`just check`, clippy
zero, the full crafted-violation sweep) rides this phase.

This ordering also matches the dependency chain in the entrance criteria: each
phase's EN-1 is the prior phase's completion, so a fresh agent can pick up any
phase knowing exactly what is already green beneath it.

## Notes

- No e2e harness exists for `spec`; verification is unit-test driven in three
  layers — A (`registry.rs` pure checks over hand-built registries), B (`spec.rs`
  parse/render), C (`build_registry` over a temp corpus, the seam Charge I
  exposed). The Layer-C tests are the only ones that exercise the new file read.
- Watch the rust-embed re-embed footgun in PHASE-01 (`spec-tech.toml` edits are
  invisible until the embedding crate recompiles) and the repo clippy ceilings in
  PHASE-02/03 (string-build and BTree-only collection bans) — both carried as
  design risks R3/R4.
- `specs` / `requirements` arrays stay empty per v1 (no registry to point at yet),
  matching the slice's reserved relationships block.
