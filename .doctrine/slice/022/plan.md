# Implementation Plan SL-022: Technical-spec system support: descent, decomposition & integrity

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See doc/glossary.md § reference forms. -->

## Overview

Four phases build the tech-spec relational spine bottom-up, each ending green and
each a clean TDD unit. The ordering follows the data: first the fields exist at
rest (PHASE-01), then the registry can collect and resolve them with the FK /
subject-kind checks (PHASE-02), then the graph integrity over those edges
(PHASE-03), then a cross-cutting end-to-end validation sweep and closure
(PHASE-04). There is **no severity tier** — the codex review showed the warn it
was built for was incoherent, so a tech-only field on a product spec is a plain
hard finding and `validate`'s signature is untouched. Every phase is additive to
the SL-015 machinery; the behaviour-preservation gate (the SL-015 suites stay
green) is an exit condition throughout, with the only changes being REQ-084's
intended contract move (PRD-012 §6) and two disclosed mechanical test edits (the
`Spec { … }` and `Registry { … }` literals gaining new fields).

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
from green hand-built-registry units. The FK / subject-kind checks here are
flat set-membership only — descent and parent each: clean / invalid-kind (wrong
target *or* a tech-only field on a product subject) / dangling — plus the REQ-084
interaction rewrite. No graph walk yet — kept separate so the riskier traversal
lands in isolation.

**PHASE-03 — integrity in isolation.** The decomposition tree is the one
graph-shaped concern and the locus of the integrity findings, so it gets its own
phase. `parent_cycle` must report each cycle exactly once — the dedup the codex
review sharpened: an ordered path plus a recovered cycle slice, correct even when
a non-cycle tail feeds the ring (the cycle-2/cycle-3/tail-fed tests assert the
*count*, not mere existence). The `second_parent` finding is born by classifying
the parse error (not a raw line-scan, which would false-hit the scaffold's own
commented example) and carried through a new `build_findings` field into
`validate` — the carrier the earlier draft never specified. Together these give
REQ-087 AC1 a literal named hard finding with a non-zero exit. Isolating this
phase keeps the traversal and the error-classification guard from entangling the
simpler FK work.

**PHASE-04 — prove it end-to-end, then close.** With no severity tier to add, the
last phase is the gate: one crafted corpus driving *every* hard violation through
the `doctrine spec validate` CLI, proving non-zero exit on each and zero on a
clean corpus. PHASE-03 proves the second-parent path end-to-end (it is the new
carrier path); the findings-list cases — self-parent, cycle, FK / subject-kind —
ride the existing non-zero bail and are swept here at the CLI level so no
acceptance criterion's exit-code claim rests on a pure-function test alone.
Closure (`just check`, clippy zero, the scaffold-comment and REQ-082-AC3 review
checks) rides this phase.

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
