# IMP-310: Author selector-conformance tech spec

**Source:** RFC-004 spec-coverage assessment (2026-07-24). **Home:** RFC-004.
Full census: `.doctrine/state/coverage-selector-conformance.md` (runtime tier,
regenerate via `/spec-coverage-assessment` if lost).

## Problem

The path-intent selector + conformance mechanism shipped via RFC-004 (SL-147 v0.1,
SL-154 registry capture, SL-180 hardening) is **essentially ungoverned**:

- **Dark:** `src/conformance.rs` (~581 loc, the pure set-algebra core) and
  `src/boundary.rs` (~80 loc, `BoundaryRow`/`Provenance` leaf) — zero spec anchors.
- **Prose-dark:** selector authoring (`selector add|note|list|rm`), the `slice
  conformance` shell (`slice.rs`), and the recorded source-delta registry
  (`state.rs`) ride SPEC-014's kind-surface anchors but are undescribed — a reader
  would not learn the capability exists.
- The two promising spec hits are false positives: SPEC-013 "conformance" = CLI
  listing-grammar (`tests/e2e_list_conformance.rs`); SPEC-023 "selector" =
  prompt-cascade. Neither PRD-001 nor PRD-013 mentions it. RFC-004 is the only
  "why" and is governance-neutral (ADR-014), not durable spec intent.

A future refactor of the conformance engine has no spec to preserve behaviour
against.

## Fix direction

**One new component-level tech spec** for the conformance capability — the pure
algebra (`conformance.rs`), the `boundary.rs` leaf, the selector-declaration
contract on `SliceDoc`, and the registry data-contract + degrade-honest semantics.
Not a fold into SPEC-014 (misplaces the dispatch-capture half, bloats a kind-surface
spec); not a three-way split (fragments the algebra). Stated **joints**, not descent:
- SPEC-014 — selectors ride on `SliceDoc`; slice-namespace verbs.
- SPEC-021/022 — capture beats fire there (funnel `integrate` / solo `/execute`),
  shared `BoundaryRow`; this spec owns the registry *shape + read semantics*, the
  dispatch specs own *when it is written*.

- **C4:** component (not code-level).
- **Product intent:** graft a capability-level requirement onto an existing PRD —
  **OQ: PRD-013** (path-conformance as a sibling drift signal to requirement
  reconciliation; sibling to SPEC-002) **vs PRD-001** (authored/consumed on the
  slice lifecycle). Resolve in `/spec-product`. No new PRD required.
- **Drafting sources:** RFC-004 (resolved OQ-1/2/3/8/9/11/11a); SL-147 design
  (D6/D7 algebra, D8 registry, D12 provenance); SL-154 design (capture-leak fix);
  SL-180 design (design-time dry-run + import-belt refusal); `conformance.rs` /
  `boundary.rs` module doc-comments.

## Cheap partial (splittable sub-item)

Independent of the full spec: anchor-repair prose + `[[source]]` anchors for the
selector-declaration contract into SPEC-014, so the authored slice surface at least
describes that selectors exist. `/spec-tech` on an existing spec, not new-spec
authoring — worth doing first if the full spec stalls.

## Out of scope

The deferred RFC-004 generalizations (OQ-6 non-entity edge target sum type, OQ-7
prose anchoring, OQ-10 refinement persistence) — unbuilt, no code surface to govern.

Related: RFC-004 (resolved), SPEC-014, SPEC-021, SPEC-022, SPEC-002, PRD-013, PRD-001,
SL-147, SL-154, SL-180.
