# Relation vocabulary — semantic classes

<!-- Companion to SPEC-018 (Cross-corpus relation contract). This is a reference
     taxonomy of the RelationLabel variants by what they *mean*, not by their
     storage tier or legal source→(role,target) triples (those live in
     RELATION_RULES, src/relation.rs — the authoritative set; this doc does not
     enumerate counts). -->

The relation labels fall into semantic classes. A class groups labels that share
the same kind of *relationship* — composition, authority, work association, peer
association, or succession — regardless of which entities author them or what tier
they use. Where one label serves several intents it is refined by a closed `Role`
(ADR-016); the `references` axis (class 3) is the one such label.

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

## 3. Work → canon/artefact association (realises, addresses, is-about)

These edges connect *work* (slices, backlog items, reviews, revisions) to the
canon or *artefacts* they realise, address, or change. The source is always a work
entity; the target is canonical truth or an artefact.

| Label | Wire name | Role | Inbound | Meaning |
|---|---|---|---|---|
| `References` | `references` | `implements` | `implemented by` | "this slice builds the capability that SPEC/PRD/REQ defines" |
| `References` | `references` | `scoped_from` | `scoped into` | "this slice was scoped from that backlog item" |
| `References` | `references` | `concerns` | `concerned by` | "this work bears on / is about that numbered entity" |
| `Slices` | `slices` | — | `slices` | "this backlog item is implemented by those slices" |
| `Drift` | `drift` | — | `drift` | "this backlog item is about this free-text drift reference" |
| `Reviews` | `reviews` | — | `reviews` | "this review targets that entity" |
| `Revises` | `revises` | — | `revises` | "this revision changes that authored truth" |

**The `references` collapse (SL-149 / ADR-016).** The old noun-named `specs`
(SL→`{SPEC,PRD}`) and `requirements` (SL→`REQ`) labels named the *target kind*,
never the verb — the missing verb *is* the role. They folded onto one structural
`references` label refined by a closed `Role`: `implements` (SL → canon),
`scoped_from` (SL → backlog), `concerns` (work → any numbered). The target gate
re-keyed from `(source, label)` to `(source, label, role)`; type safety is
preserved, relocated from label to role. `concerns` also absorbs the lightweight
`reviews` *role* RFC-003 floated — heavyweight, dispositioned review stays the
first-class RV `reviews` label above.

**Why role, not source-set.** Refining intent by role keeps inbound coherent:
`references(implements)` renders "implemented by", `references(concerns)` renders
"concerned by" — a slice *realising* a spec and a chore *bearing on* one no longer
collapse to one nonsense inbound ("specs this spec"). The remaining class-3 labels
(`slices`/`drift`/`reviews`/`revises`) keep their own structural identity; they did
not fold because their inbound is already coherent. Adding a new intent is a code
change (a new `Role` variant), so the closed set stays auditable.

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
execution output. The work→canon labels (`references`, `slices`) carry the wrong
inbound semantics for records — a record *informing* a spec is not a slice
*implementing* it.

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
`(source ∈ sources, label, role, target, tier, link, inbound_name)`. This document
groups the labels by semantic class to aid understanding; it does not define
new labels, constrain sources or targets, or replace the table.

When a new label is minted, it should be placed in one of the existing classes
or justify a new class. The classification is a design tool, not a runtime
constraint.
