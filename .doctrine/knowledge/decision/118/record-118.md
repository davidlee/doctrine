# DEC-118: PRD-012 gains three requirements for the anchor report, and an OQ-2 edit

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

PRD-012 gains three requirements, authored by `/spec-product` from the intent
below. Wording is that skill's to fix; what is settled here is which
requirements exist and at what altitude.

1. **FR — report which governed units are anchored, by which spec, and which are
   unanchored.** The inverse view. This is the gap.
2. **FR — enumerate governed units through a project-declared adapter, never by
   inferring the host project's layout.** Stays at PRD altitude, naming the
   project-declared adapter.
3. **NF — never present an uninventoried surface as an ungoverned one.** The
   product-level statement of DEC-113.

`REQ-085` (anchor a spec to the code it governs) and `REQ-088` (hand-authored and
imported specs converge on one anchor) already exist and must **not** be
re-required.

Authored directly via `spec req add`, not through a Revision: ADR-013 routes
governance dependency through a REV for corrective reconciliation, and this is
additive authoring on a PRD the slice is extending by design.

## Why the second one stays at product altitude

It was flagged as the arguable one. The case against: "a project-declared
adapter" names a mechanism, and PRD-012's other requirements stay resolutely at
the *what* — `REQ-085` says "anchor a technical spec to the code it governs" and
says nothing about how. A reworded version — *enumerate governed units without
depending on any host project's layout conventions* — would state the constraint
and leave the adapter to the tech spec.

Decided against that softening. Naming the project-declared adapter is
legitimate product intent here, not implementation leakage: it is the commitment
that makes the capability portable at all, and a PRD that only forbade
layout-dependence would admit implementations that satisfy the letter by
guessing more cleverly. The constraint and the shape that discharges it are the
same statement.

The alternative of dropping it entirely — on the grounds that POL-002 already
binds it, so a requirement restating a policy is redundant — was also declined.
A policy binds the platform's conduct; a requirement states what the capability
must do. The overlap is real but they are not the same claim, and the PRD should
be readable without the policy corpus beside it.

## The gap is real and narrow — established, not assumed

`REQ-081`…`089` cover C4 placement, descent, decomposition, typed peer relations,
the code anchor, lineage on retire/supersede, acyclic decomposition integrity,
hand/import convergence, and evergreen altitude. None requires the inverse view.
Confirmed by the `/spec-coverage-assessment` census, which also found the
capability **fully dark** — no spec, no PRD requirement, not even a prose claim.

## OQ-2 is partly answered — a status edit, not a requirement

PRD-012 `OQ-2` asks what external code model a code-structure importer reads —
"likely an integration with another tool rather than much new code" — and at what
C4 granularity it emits.

The adapter contract answers the first half: a project-declared external command
emitting a containment tree of units. SL-243's commitment that granularity is
the adapter's decision, with the engine owning only rollup and collapse, bears on
the second without closing it.

So `OQ-2` moves to partially-resolved with that recorded, in the shape `OQ-1` and
`OQ-5` already use. This is an edit to an existing open question, not new
requirement surface, and the pre-design research round did not surface it — the
census did.

## Provenance

Settled at the `inq-2` fork of design run `dr-019fc13a` (SL-243), after the
`/spec-coverage-assessment` pass the fork was deferred to. Coverage map at
`.doctrine/state/sl-243-spec-coverage-map-anchor-map.md` — runtime tier and
disposable; this record is part of its durable residue.

## Related

- [[DEC-117]] — the spec home for the capability these requirements govern.
- [[DEC-113]] — the decision requirement 3 states at product altitude.
