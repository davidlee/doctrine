# PRD-012: Technical Specifications

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See doc/glossary.md § reference forms. -->

## 1. Intent

A governed codebase records its product intent — the *what* and *why* — in
product specs (PRD-002). But the durable *how* of a capability — the
architecture that realises it, the components and their boundaries, the
mechanisms and invariants the implementation must honour — has no evergreen
home. It lives in a single change's design, in `doc/*` prose nobody trusts to be
current, or only in the code itself. When an agent needs to descend from
"what this capability is" to "how it is built" before scoping a change, there is
nothing upstream to read: it re-derives the architecture from source, or infers
one and gets the boundaries wrong.

A **technical specification** answers this need. It is the durable, evergreen
statement of *how* a capability is built — distinct from the product intent it
descends from (PRD-002) and from any single change's design. Its value is that
architecture becomes a first-class artefact: a later agent can read the
component boundaries, the decomposition, and the code anchors before touching
the system, instead of reverse-engineering them every time.

The technical spec is a **superset** of the product spec. It reuses the shared
spec machinery — identity, requirements as peer entities, on-demand reassembly,
corpus integrity (all PRD-002) — and adds what only the *how* needs: a place on
the **C4 ladder** of abstraction, **decomposition** into a hierarchy of finer
specs, typed **peer relations** to sibling specs, and **anchors** to the code it
governs. Those anchors are also the convergence seam for a future capability:
generating technical specs *from* code structure, so a hand-authored corpus and
an imported one meet on one entity rather than two parallel surfaces.

## 2. Scope

In scope:

- Declaring the durable architecture and mechanism of a capability — its
  components, boundaries, data flow, and invariants — descending from the
  product intent (PRD-002) it realises and distinct from any single change's
  design.
- Placing a technical spec on the C4 ladder of abstraction (context, container,
  component, code) so its altitude is explicit.
- Decomposing a coarse spec into a single-parent, acyclic hierarchy of
  finer-grained specs, and relating specs as architectural peers distinct from
  that containment.
- Anchoring a technical spec to the concrete code it governs, and admitting both
  hand-authoring and import from code structure onto that one anchor.
- Tracking the living architecture as code refactors: a spec can be retired,
  superseded, merged, or split, with lineage recorded.

Out of scope:

- The **shared spec entity machinery** — identity reservation, requirements as
  peer entities and their membership/labels, on-demand reassembly, and corpus
  referential integrity. These are owned by PRD-002 and reused unchanged; this
  spec does not restate them.
- The **product *what/why*** — intent, scope, and success measures of a
  capability — which is the product spec's, not the technical spec's.
- A **single change's design** — current-vs-target behaviour, code impact, and
  verification of one bounded change — which is a slice's design (PRD-001), not
  an evergreen technical spec.
- The **drift ledger** — recording a mismatch between a technical spec and the
  code it anchors (a stale anchor, a "ship of Theseus" divergence) is the
  cross-cutting drift capability's job (`doc/drift-spec.md`), referenced here,
  not owned. This spec *surfaces* the obligation to detect such drift; it does
  not specify the ledger.
- The **importer's source and shape** — the concrete mechanism that reads an
  external code model and emits technical specs is unresolved (OQ-2) and not
  specified here; only the convergence requirement its output must satisfy is.

Boundary: a product spec owns the durable *what/why*; a technical spec owns the
durable *how*; a slice owns one change; a drift ledger owns a recorded mismatch.
A technical spec descends from a product spec — it does not restate the *what*,
and the *how* and the *what* never share a home.

## 3. Principles

- **A technical spec is the durable *how*, descending from a product *what*.**
  It realises a product capability's intent; it never re-derives or restates that
  intent, and it never collapses into a single change's design.
- **Containment and peering are different relations.** Decomposition is a
  single-parent, acyclic hierarchy — a spec has at most one parent. Peer
  interactions (uses, calls) are many and typed. Conflating the two loses the
  architecture's shape.
- **Reciprocity is derived, never stored.** A parent's children and a peer's
  inbound edges are computed from the outbound records, never written twice
  (ADR-004).
- **The code anchor is the single convergence seam.** Whether a spec is
  hand-authored or imported from code structure, it anchors the same governed
  code through one mechanism; the two paths reconcile to one entity, never a
  parallel surface.
- **Evergreen is not immortal.** A technical spec endures across changes to the
  code that realises it, but architectural refactoring — merge, split,
  extraction, dissolution — can legitimately transform or end it. Identity
  across those transitions is governed by recorded lineage, never a silent
  rewrite and never an orphaned child.
- **Spec–code drift is surfaced, not silently carried.** When the code a spec
  anchors moves or disappears, that divergence is a finding for the drift
  capability to record, not a quiet inconsistency the corpus tolerates.

## 4. Requirements

The functional and quality requirements this capability must satisfy are
recorded as requirement entities and appear under the synthesized Requirements
section below. This section carries only the constraints and invariants that
bound every valid implementation.

Constraints:

- The C4 level is a closed set — context, container, component, code — added
  deliberately, never improvised per spec.
- Containment and peer interaction are distinct stored relations and must not be
  conflated: containment is single-parent and outbound; peer interactions are
  many and typed.
- Cross-family descent (a technical spec to the product capability it realises)
  stores the target's durable peer id only, never a compound or owner-qualified
  key.
- A technical spec must not hold a single change's design; that is the slice's
  design artefact.
- Recording a spec–code mismatch is the drift capability's surface, not this
  one; this capability only exposes the anchor data a drift pass reads.

Invariants:

- Decomposition forms a tree: every technical spec has at most one parent, and
  no chain of parents forms a cycle.
- Containment and peer relations are stored outbound-only; the reciprocal view
  (a parent's children, a peer's inbound edges) is always derived, never stored.
- A code anchor identifies the same governed code whether its spec was
  hand-authored or imported; the two paths never produce parallel entities for
  one anchor.
- The technical *how* and the product *what* never share a home; a technical
  spec descends from a product spec without restating it.
- A transformed spec (retired, superseded, merged, split) never silently
  disappears and never orphans its decomposition children; its lineage remains
  recoverable.

## 5. Success Measures

- An agent descending from a capability's product intent can read its
  architecture — components, boundaries, decomposition, and the code each part
  governs — from the technical spec alone, without reverse-engineering source.
- The decomposition of a subsystem is legible at a glance: which spec is the
  coarse parent and which are its finer children, and which relations are peers
  rather than containment.
- A spec's altitude is unambiguous: its C4 level tells a reader whether it
  describes a system, a deployable unit, a component, or code.
- When the architecture refactors — two components merge, one splits — the
  corpus reflects the new shape with lineage intact, and no child is left
  pointing at a vanished parent.
- A future importer generating specs from code structure lands on the same code
  anchors a hand author would use, producing no duplicate or competing entity
  for an already-specified unit.
- A spec whose anchored code has moved or disappeared is detectable, so the
  drift capability can record the divergence rather than the corpus carrying it
  silently.

## 6. Behaviour

Primary flow — place and anchor: an operator authors a technical spec for a
capability's architecture, declares its C4 level, and anchors it to the code it
governs. The spec records the product capability it descends from. It opens in
the draft stage and reassembles (PRD-002) as one readable whole.

Primary flow — decompose: an operator marks a finer-grained spec as a child of a
coarser one. The containment is stored once, outbound, on the child; the
parent's set of children is derived. A coarse spec and its children together
describe one subsystem at successive C4 levels.

Primary flow — relate as peers: an operator records a typed interaction (a spec
uses or calls another) between two technical specs. The edge is a peer relation,
distinct from containment, and resolves to an existing technical spec.

Transform flow — the living architecture: as code refactors, a spec is retired,
superseded by another, merged with siblings into one, or split into several. The
transition records lineage — what became what — rather than deleting or
rewriting the entity in place; a parent that is removed reattaches or
re-parents its children rather than orphaning them.

Import flow (forward-looking): a tool reads an external code model and emits
technical specs anchored to code structure. Where an anchor already has a
hand-authored spec, the import reconciles to that one entity; it does not create
a parallel spec for the same governed code.

Integrity and drift guards: a containment that would introduce a cycle or a
second parent is rejected as a hard finding. A spec whose code anchor no longer
resolves to live code is surfaced as drift for the drift capability to record;
the corpus does not silently carry the mismatch.

Edge cases and failure modes: a root spec has no parent and that is valid; a
peer interaction targeting a product spec rather than a technical one is a
dangling reference; a merge that leaves a child pointing at a superseded parent
is an orphan the integrity guard flags.

## 7. Verification

Verification confirms that a technical spec durably carries the *how* of a
capability, that containment is a single-parent acyclic tree with derived
reciprocity, that peer relations stay distinct from containment, that code
anchors converge hand and import paths, and that architectural transforms
preserve lineage — without binding the spec to a particular implementation.

Placement and anchoring are proven by confirming a technical spec records its C4
level from the closed set, the product capability it descends from, and the code
it governs, and reassembles as one readable whole. Decomposition is proven by
confirming containment is stored outbound on the child, a parent's children are
derived rather than stored, and a containment that would form a cycle or a
second parent is rejected. Peer relations are proven by confirming a typed
interaction resolves to an existing technical spec and is reported dangling
otherwise, and that it is never mistaken for containment. The hand/import
convergence is proven by confirming an import onto an already-anchored unit
reconciles to the existing entity rather than creating a parallel one. The
transform behaviour is proven by confirming a retired, superseded, merged, or
split spec preserves recoverable lineage and never orphans its children. Drift
detection is proven by confirming a spec whose anchor no longer resolves to live
code is surfaced, deferring the recording itself to the drift capability.

Where a check must reference a specific obligation, it cites the durable
requirement entity (REQ-NNN), never a mobile membership label. Coverage of the
functional and quality requirements is tracked against those entities, not
duplicated here.

## 8. Open Questions

- OQ-1 — Cross-family descent has no edge mechanism yet: a technical spec must
  record the product capability it realises, but the shipped spec→spec edge
  (`interactions`) is technical-only. Does descent reuse a generalised typed
  edge, a dedicated field, or a new relation kind? Blocks wiring descent and any
  coverage gate that reads it.
- OQ-2 — The importer's source and shape are unresolved: what external code
  model does code-structure import read (likely an integration with another
  tool rather than much new code), and at what C4 granularity does it emit —
  code level only, or up to component? Blocks the import verb; the convergence
  requirement constrains its output regardless.
- OQ-3 — "Ship of Theseus" identity: when incremental refactoring wholly
  replaces a component's mechanism while keeping its name and anchor, is it the
  same spec or a successor? What threshold distinguishes an in-place evolution
  from a supersede? Blocks the merge-versus-supersede policy.
- OQ-4 — Merge and split lineage representation: how are "N specs become one"
  and "one spec becomes N" recorded over the decomposition tree and the status
  lifecycle — typed lineage edges, status plus interactions, or a dedicated
  lineage facet? Blocks the transform verbs.
- OQ-5 — Hand-authoring depth versus import: should the hand-authored corpus
  stop at component level and reserve code-level, per-unit specs for the
  importer, or is hand-authoring at code level ever warranted? Blocks the
  backfill's altitude boundary (SL-021).
