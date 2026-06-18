# REV REV-002 — Reconcile PRD-014 estimation unit + Value facet coverage (SL-101 drift)

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

**Origin.** SL-101 (`done`) renamed the estimation unit default
`high_caffeine_hours → espresso_shots` and introduced a sibling **Value facet**
(single magnitude, unit `magic_beans`). The reconciliation under SL-101 (REV-001)
amended SPEC-020's *prose* but left drift behind:

1. **PRD-014** was never amended — it still read `high_caffeine_hours` throughout
   and carried no Value coverage at all.
2. **PRD-014 REQ-263** (FR-003) still read `high_caffeine_hours`.
3. **SPEC-020 REQ-271** (FR-003) still read `high_caffeine_hours` — the REV-001
   prose amend missed the requirement entity.
4. **SPEC-020 REQ-278/279/280** (the Value reqs) were bare label stubs — no
   description or acceptance criteria.

Per the standing ruling (memory `mem_019edacf…`): SL-101's design is the
forward-looking authority and the specs catch up at reconciliation. This REV is
that catch-up for the product spec and the two missed requirement entities.

**Value product intent (decided this REV).** The Value facet is the deliberate
counterpart to the Estimate's attention-burden *cost*: a single magnitude of
expected **worth/payoff**, so cost can be weighed against value across the graph.
Cost-vs-value arithmetic (ROI, ratios, ranking) stays a caller/Cordage concern,
never baked into the facet.

### Staged delta

**Unit rename `high_caffeine_hours → espresso_shots`** (modify rows):

- PRD-014 §4 Constraints, §6 Configuration + display examples (prose).
- REQ-263 — PRD-014 FR-003 statement + acceptance criteria.
- REQ-271 — SPEC-020 FR-003 statement + acceptance criteria.

**PRD-014 Value integration** (prose rewrite + new members):

- Title `Estimation → Estimation & Value`; responsibilities + tags broadened.
- §1–§8 prose: Value woven through Intent, Scope, Principles, Requirements
  (constraints/invariants), Success Measures, Behaviour, Verification; OQ-6 added
  (uncertainty-aware cost-vs-value pairing, deferred).
- Five new Value functional requirements added to PRD-014 via `spec req add`
  (the REV `introduce` action is SPEC-only and cannot model PRD members, so they
  are recorded here): **REQ-281** (FR-006, record), **REQ-282** (FR-007, validate),
  **REQ-283** (FR-008, configure unit), **REQ-284** (FR-009, display),
  **REQ-285** (FR-010, graph-expose).
- NF reqs broadened to cover both facets (modify rows): **REQ-266** (optional /
  non-gating), **REQ-267** (reusable / kind-agnostic), **REQ-268** (forward-compatible).

**SPEC-020 Value stub enrichment** (modify rows): **REQ-278/279/280** — added
description + acceptance criteria.

### Scope boundary — NOT in this REV

The Value facet (and the Estimate facet) are **not integrated into `main`**:
`src/estimate.rs` is a dead, unwired module and `src/value.rs` does not exist on
`main` (it sits stranded on the `dispatch/101*` branches). That integration is a
separate code change — a new slice citing SL-101 — tracked outside this spec-only
reconciliation. This REV ships the spec truth; the code catches up next.
