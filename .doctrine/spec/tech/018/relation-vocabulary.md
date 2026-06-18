# Relation vocabulary — semantic classes

<!-- Companion to SPEC-018 (Cross-corpus relation contract). This is a reference
     taxonomy of the 17 RelationLabel variants by what they *mean*, not by their
     storage tier or legal source→target pairs (those live in RELATION_RULES,
     src/relation.rs). -->

The 17 relation labels fall into five semantic classes. A class groups labels
that share the same kind of *relationship* — composition, authority, work
association, peer association, or succession — regardless of which entities
author them or what tier they use.

## 1. Composition / lineage (part-of, derives-from)

These edges express how entities are *composed* from or *derived* from other
entities — structural decomposition and descent, not work association.

| Label | Wire name | Inbound | Meaning |
|---|---|---|---|
| `DescendsFrom` | `descends_from` | `descends_from` | "this tech spec realises that product spec" (arity ≤ 1) |
| `Parent` | `parent` | `parent` | "this spec decomposes under that parent spec" (arity ≤ 1) |
| `Members` | `members` | `members` | "this spec contains these requirements" (via `members.toml`) |
| `Consumes` | `consumes` | `consumed_by` | "this product spec depends on a seam from that product spec" |

**Why these together.** All four are about structural decomposition or
derivation, not about work. A spec's `parent` and `descends_from` fix its place
in the spec tree; `members` fixes what it contains; `consumes` crosses trees.
They share arity constraints (lineage edges are single-valued) and typed storage
(tier-2), unlike the multi-ref work-association edges.

## 2. Authorization / governance (is-bound-by)

These edges express that an entity is *constrained by* or *accountable to*
governance — ADRs, policies, standards — or that a reconciliation record belongs
to a slice.

| Label | Wire name | Inbound | Meaning |
|---|---|---|---|
| `GovernedBy` | `governed_by` | `governs` | "this artefact is constrained by that ADR/policy/standard" |
| `OwningSlice` | `owning_slice` | `owning_slice` | "this reconciliation record belongs to that slice" |

**Why these together.** Both express an authority or ownership relationship.
`GovernedBy` is the broad cross-corpus axis (slice, spec, concept-map → ADR/POL/STD);
`OwningSlice` is the narrow rec→slice axis. They do not imply work — being
governed by an ADR is not the same as implementing it.

## 3. Work → artefact association (realises, addresses, is-about)

These edges connect *work* (slices, backlog items, reviews, revisions) to the
*artefacts* they realise, address, or change. The source is always a work
entity; the target is always an artefact.

| Label | Wire name | Inbound | Meaning |
|---|---|---|---|
| `Specs` | `specs` | `specs` | "this slice/backlog-item realises/is-scoped-to those specs" |
| `Slices` | `slices` | `slices` | "this backlog item is implemented by those slices" |
| `Requirements` | `requirements` | `requirements` | "this slice addresses those requirements" |
| `Drift` | `drift` | `drift` | "this backlog item is about this free-text drift reference" |
| `Reviews` | `reviews` | `reviews` | "this review targets that entity" |
| `Revises` | `revises` | `revises` | "this revision changes that authored truth" |

**Why these together.** All six share the same semantic shape: a *work entity*
points at what it *acts on*. The inbound rendering reflects this ("specs" →
"specs" reads as "these entities spec this spec" — loose but tolerable within
the work-artefact class). The class boundary matters because adding a
non-work source (e.g. a knowledge record) to one of these labels would
collapse the inbound distinction — a record *informing* a spec is not the same
as a slice *realising* it.

**Inbound collision risk.** `Specs` inbound currently renders `specs: [SL-046]`
— "these entities spec this spec." That works for slices and backlog items but
would read as nonsense for records ("these entities spec this spec" for an
assumption). Similarly, `Slices` inbound on a slice renders `slices: [ASM-001]`
— "these entities sliced this slice" — incoherent. This is why knowledge
records need their own labels (class 6 below), not source-set extensions of
existing work→artefact edges.

## 4. Peer association (relates-to, within-category)

These edges express association between peers — entities of the same kind, or
cross-kind associations that are neither compositional nor authoritative.

| Label | Wire name | Inbound | Meaning |
|---|---|---|---|
| `Related` | `related` | `related` | "this governance entity is associated with that one" (symmetric) |
| `Interactions` | `interactions` | `interactions` | "this spec interacts with that spec" |
| `Contextualizes` | `contextualizes` | `contextualized_by` | "this concept map contextualizes that concept" |

**Why these together.** All three are about *association* without hierarchy or
work implication. `Related` is symmetric governance↔governance; `Interactions`
is spec↔spec with a free-text payload; `Contextualizes` is concept-map → any
with a distinct inbound name (`contextualized_by`).

## 5. Replacement / succession

These edges express that one entity *replaces* another as the authoritative
source — the successor carries forward the predecessor's intent.

| Label | Wire name | Inbound | Meaning |
|---|---|---|---|
| `Supersedes` | `supersedes` | `superseded_by` | "this entity replaces that one as the authoritative source" |

**Why its own class.** `Supersedes` is unique: it is a lifecycle transition, not
a static association. It carries the only sanctioned reverse edge
(`superseded_by`, ADR-004 §5 carve-out), co-written atomically. It spans
multiple source groups (slice→slice is writable; governance→governance is
lifecycle-only) and will extend to knowledge-record cross-kind supersession
(SPEC-019 FR-006, via IMP-006). No other label moves the predecessor to a
terminal status.

## 6. Free-text / external citation

These labels carry unvalidated free-text targets — they dangle by design and
have no inbound rendering through the graph.

| Label | Wire name | Inbound | Meaning |
|---|---|---|---|
| `DecisionRef` | `decision_ref` | `decision_ref` | "this reconciliation record cites that external decision-log ref" |
| — (drift is already in class 3) | | | |

**Why separate.** `DecisionRef` carries external DEC-NNN-XX citations (the
3-part form) that are not doctrine entities — it is unvalidated free text, not
a cross-corpus edge. `Drift` is also unvalidated but fits class 3 semantically
(backlog item → drift reference).

## Gap: epistemic / knowledge-record association

None of the existing classes capture "truth that shapes work." A knowledge
record (assumption, decision, question, constraint — SPEC-019) *informs*,
*constrains*, *grounds*, or *motivates* — it is epistemic input to work, not
execution output. The existing work→artefact labels (`specs`, `slices`,
`requirements`) carry the wrong inbound semantics for records.

This gap is addressed by SPEC-019's relation seam (FR-005), which mints two
new labels for the RECORD source-group:

- A **record→backlog-item relate label** — "this record informs/bears-on that
  backlog item" (IMP-053).
- **`spawns`** — "this record spawned that work item" (a distinct origin edge;
  e.g. `ASM-001 → RSK-004`).

These will form a sixth semantic class: **epistemic → work** (informs,
constrains, spawns). They are distinct from class 3 (work→artefact) because the
source is *truth*, not *work* — a record is never actionable (SPEC-019 D7).

## Relation to SPEC-018 and RELATION_RULES

This taxonomy is **descriptive**, not prescriptive. The legal vocabulary lives
in `RELATION_RULES` (`src/relation.rs`) — the code-authoritative table of
`(source ∈ sources, label, target, tier, link, inbound_name)`. This document
groups the labels by semantic class to aid understanding; it does not define
new labels, constrain sources or targets, or replace the table.

When a new label is minted, it should be placed in one of the existing classes
or justify a new class. The classification is a design tool, not a runtime
constraint.
