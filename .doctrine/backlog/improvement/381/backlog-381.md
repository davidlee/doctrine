# IMP-381: Assess specification coverage for plan criterion lineage

## Source

RFC-027 Stage 1 and EVD-005. The bounded history trial retained opt-in criterion
lineage for specification coverage after testing replacement, withdrawal, split,
merge, cross-phase relocation, and an unchanged negative control.

## Problem

Plan criteria currently have immutable phase-local ids but flat, mutable rows.
Evolution is expressed through in-place prose, repeated successor text, and Git.
Only VT rows have machine consumers, while EN, EX, VA, and VH remain normative
instructions for human and agent readers. No durable specification clearly owns:

- what a criterion's identity and immutability cover;
- how a changed criterion identifies predecessors and successors;
- which leaves govern after replacement, withdrawal, split, merge, or relocation;
- how historical criteria render without appearing active;
- which behaviour belongs to plan structure versus verification or coverage.

RFC-027 is deliberative and cannot supply that governance.

## Assessment scope

Run `/spec-coverage-assessment` for the criterion-evolution capability. Inventory
at minimum:

- PRD-001 and SPEC-014 as the current slice/plan container owners;
- RFC-023's criterion-evolution direction;
- RFC-027 H4 and EVD-005;
- the plan read model and consumers in `src/plan.rs`, `src/vtgate.rs`, and
  `src/state.rs`;
- the real histories in SL-233, SL-241, and SL-224;
- existing supersession/read patterns only where they clarify a reusable joint,
  not as assumed implementation.

Decide the product altitude and C4 boundary, whether existing specifications can
be amended or a new product/technical specification is required, and where these
facts belong:

- phase-qualified criterion identity;
- revision versus id immutability;
- predecessor/successor disposition;
- active-leaf derivation;
- historical rendering;
- split/merge cardinality and cross-phase relocation;
- validation and migration boundaries.

## Boundaries

- Assessment only: do not draft a final schema, design, slice, or implementation.
- Unchanged criteria must remain zero-ceremony.
- Criterion prose remains the normative semantic statement; lineage need not
  prove semantic completeness mechanically.
- A possible VT pattern-set conservation check is separable and should not define
  the general lineage boundary.
- H3 change claims remain rejected by EVD-004.
- Selector/read-surface work stays with ISS-251 and IMP-310.
- Persisted criterion evidence and RFC-027 H6 remain separate unless the coverage
  census finds an unavoidable ownership joint.

## Exit

Produce the normal cited coverage assessment with:

1. governed versus dark behaviour and code surfaces;
2. recommended product and technical specification homes;
3. explicit amendment-versus-new-spec rationale;
4. ownership joints with SPEC-014, SPEC-002, and any relevant relation contract;
5. remaining questions that must enter product/spec design rather than being
   guessed during assessment;
6. the next governing skill (`/spec-product` or `/spec-tech`) if ready.

## Result

Assessed 2026-08-02. Coverage map (runtime tier, regenerate via
`/spec-coverage-assessment` if lost):
`.doctrine/state/imp-381-coverage-map-criterion-lineage.md`.

Headline: criterion lineage has **no host spec to amend**. `src/plan.rs` and
`src/vtgate.rs` are anchor-dark corpus-wide; SPEC-014 describes `plan.toml` only
as a sibling scaffold; PRD-001 carries no plan requirement; PRD-015 delegates
"slice/phase planning and execution semantics" to PRD-001, which does not hold
them. Criterion identity and immutability are normative only in `glossary.md` /
`using-doctrine.md` / the boot snapshot — reference prose, not a spec.

Recommendation: amend PRD-001 (feature altitude), then one new component tech
spec "Phase plan surface" (`descends_from = PRD-001`, `parent = SPEC-004`)
owning the plan content model and the lineage contract, with the VT gate as a
section projecting into SPEC-002's executable seam rather than a separate spec.
Next skill: `/spec-product` first — the amend-vs-sibling-PRD question is a
product-altitude call that must settle before the tech spec has a parent.
