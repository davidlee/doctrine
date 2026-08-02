# IMP-382: Govern the phase-plan content model

## Source

IMP-381's specification coverage assessment, commissioned by RFC-027 Stage 1
after EVD-005 retained opt-in criterion lineage.

## Problem

Doctrine has no product or technical specification for the authored phase-plan
content model. PRD-001 governs slice entities but contains no requirements for
plans or criteria. PRD-015 delegates slice and phase planning semantics to
PRD-001, making that delegation presently incomplete. SPEC-014 owns only the
`plan.toml` scaffold and fileset boundary, not its contents.

The gap is broader than lineage. Criterion identity, immutability, modes,
ordering, active meaning, validation, rendering, and migration are normative
only in shipped reference prose. `src/plan.rs` and `src/vtgate.rs` have no spec
source anchors, while plan verbs in otherwise anchored modules are prose-dark.
That leaves no governed host in which predecessor/successor disposition can be
specified.

## Intended outcome

1. Run `/spec-product` to establish product ownership. Amend PRD-001 unless the
   product-altitude review demonstrates that a sibling PRD is the cleaner owner.
2. Run `/spec-tech` to author one component specification, provisionally “Phase
   plan surface”, descending from that product owner and parented by SPEC-004.
3. Govern the complete plan content contract before adding criterion lineage:
   identity, immutability, modes, order, active/historical interpretation,
   validation, rendering, migration, and evolution acts.
4. Anchor the owned implementation surfaces, including `src/plan.rs`,
   `src/vtgate.rs`, `install/templates/plan.toml`, and the plan-specific portions
   of shared modules.

## Ownership joints

- SPEC-014 retains scaffold and fileset ownership; the new spec owns plan
  contents.
- SPEC-002 retains executable coverage procedures and observed evidence. The VT
  gate projects plan criteria into that seam; it does not create a second
  command schema.
- SPEC-018 remains a corpus-entity relation contract. Criterion lineage is
  document-local unless a later consumer earns corpus identity.
- IMP-310 and ISS-251 continue to own selector-conformance and its derived read
  surface.

## Boundaries

- Do not settle the lineage schema or implement it in this item.
- Preserve zero ceremony for unchanged criteria.
- Do not promote criterion ids into corpus entity ids without a proven consumer.
- Carry IMP-381's deferred design questions into product/spec design rather than
  guessing their answers here.

## Next action

Enter `/spec-tech` to create the component specification "Phase plan surface",
descending from PRD-001 and parented by SPEC-004. It must own the plan content
schema and mechanisms, settle PRD-001's OQ-2 through OQ-4, preserve the ownership
joints above, and anchor the implementation surfaces named in the intended
outcome.

## Product-spec result

REV-045 amended PRD-001 rather than creating a sibling PRD. PRD-001 now owns the
durable phase-plan product contract through REQ-439 to REQ-447: ordered plan
content, EN/EX and VT/VA/VH semantics, immutable identities and order, opt-in
criterion evolution, validation, governing-versus-historical rendering,
compatibility and migration, unambiguous agent/human reads, and single ownership
across authored and observed tiers.

DEC-119 records the local Revision workaround: `revision change introduce`
rejects PRD members under the known IMP-297 gap, so REV-045's primary
`modify PRD-001` row umbrellas the requirements created by `spec req add`.
