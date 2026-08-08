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

## Tech-spec result

SPEC-031 — *Phase plan surface* — authored 2026-08-08, `descends_from PRD-001`,
`parent SPEC-004`, C4 component, status **draft** pending review. It carries
REQ-462 to REQ-475 (ten functional, four quality) and anchors `src/plan.rs`,
`src/vtgate.rs`, `install/templates/plan.toml`, and the plan-specific portions
of `src/slice.rs` and `src/state.rs`.

It settles all three questions PRD-001 left for its descending technical spec:
`OQ-2` (no separate evolution act identity — predecessor/successor sets plus
disposition carry the audit history) as D9; `OQ-3` (the governing set orders by
authored position of the governing rows, never by identifier and never inherited
from a predecessor) as D10; `OQ-4` (additive changes get a compatibility read;
only meaning-changing ones get an authored migration) as D11.

It also settles the question RFC-029 § 6 left open for whoever wrote this spec:
the verification-to-exit link lives **on the verification row**, with the inverse
derived (D6) — the fact IMP-409 needs and deliberately did not decide.

Interaction edges state the joints this item required: `bounds` SPEC-014
(fileset versus contents), `projects-into` SPEC-002 (no parallel command or
evidence schema), `uses` SPEC-018 (criterion lineage stays document-local),
`read-by` SPEC-012 (dispatch projects runtime status read-only over the plan).

Boundaries held: the lineage *storage location* is left open (spec OQ-2), no
criterion id is promoted to a corpus entity id, and unchanged criteria stay
zero-ceremony.

Outstanding before this item resolves: flip SPEC-031 `draft` → `active`, and
repoint `glossary.md` / `using-doctrine.md` at it so criterion identity and
immutability stop having two independently authoritative homes.
