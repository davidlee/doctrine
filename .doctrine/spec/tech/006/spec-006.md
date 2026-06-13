# SPEC-006: Spec composition machinery

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

A spec is the aggregate root a requirement is woven into. This container owns the
machinery that *composes* one — assembling a spec from its identity, its prose, its
requirement peers, and its relational spine, and then reassembling and validating
the whole on read. It sits beneath the whole-system root (SPEC-003) and rides the
shared entity engine (SPEC-004) for materialisation, identity, and the atomic claim;
this spec restates none of that and owns only what is specific to *spec composition*:
the two spec subtypes, the requirement-as-peer model, the membership and interaction
edge tables, the spine fields, and the registry reassembly and FK-integrity pass.
The tech-spec spine itself — `descends_from`/`parent`/`c4_level`/anchors as a
capability — is detailed by its own descending component; this container owns the
composition mechanism those fields ride on.

## Responsibilities

Mirrors the structured `responsibilities` list: assemble the spec as a composite
entity across two subtypes; own the relational spine; carry the requirement-as-peer
composition and its membership edges; mediate requirement authoring through
`spec req add`; carry the tech-only interaction peer-edge table; reassemble the whole
spec on `show`; and run the corpus FK-integrity pass on `validate`.

### Two subtypes, one engine

A spec exists in two subtypes — product (`spec/product`, `PRD`) and tech
(`spec/tech`, `SPEC`) — riding two `entity::Kind`s over the same kind-blind engine.
Each subtype is its own directory tree and its own reservation namespace; they
diverge only in their scaffold fileset and in the tech-only flat fields. A product
spec scaffolds three content files plus a seeded `members.toml`; a tech spec adds a
seeded `interactions.toml`. The directory holds the identity-and-prose pair plus the
edge tables:

```text
spec/<subtype>/NNN/
  spec-NNN.toml        # identity, lifecycle, the spine + flat fields (tech)
  spec-NNN.md          # prose body
  members.toml         # [[member]] rows — the spec→requirement edges
  interactions.toml    # [[edge]] rows — tech-only outbound spec→spec peers
```

### The relational spine

A spec's relations sit within the **cross-corpus relation contract (SPEC-018**,
governed by ADR-010) — this spec does not re-tell that model. Its axes are the
contract's **tier-2 typed** edges: single-valued, payload-carrying, or
required-field relations that keep a bespoke shape rather than the uniform tier-1
`[[relation]]` block (lineage `descends_from`/`parent`, `members`, `interactions`).
That partition rationale and the outbound-only rule are SPEC-018's; the kind-specific
mechanism is below.

The spine places a spec in the corpus tree, authored as TOML and gated by `validate`:
`descends_from` is the single-valued tech→product capability link; `parent` is the
single acyclic decomposition parent; `c4_level` is the closed C4 set; `[[source]]`
is the repeatable code-anchor list. There is no CLI flag for any spine field — the
engine round-trips this TOML but never generates it, and a spine field is never
expressed as an interaction edge (containment and peering are distinct axes).

### Requirements as peer entities

A requirement is not a sub-kind of a spec — it is its own `REQ-NNN` entity, woven
onto a spec through a `members.toml` `[[member]]` row. That row carries the foreign
key to the requirement, a sticky `FR`/`NF` membership label, and an advisory order.
The label is **membership state on the edge**, not the requirement's identity: the
same requirement could in principle be membered under different labels, and the
durable `REQ-NNN` is what every reference targets. `spec req add` is the
spec-mediated producer — a requirement has no standalone create verb. It reserves the
requirement, overwrites its seeded kind, allocates the next-free `FR-`/`NF-` label
among the spec's existing members (or honours an explicit `--label`), and appends the
membership row edit-preservingly. The sequence is deliberately non-transactional: a
failure after the reserve leaves an uncommitted orphan requirement that the integrity
pass flags hard rather than one the producer silently rolls back.

### Interaction peers

Tech specs carry an `interactions.toml` of outbound `[[edge]]` rows — `uses`/`calls`
peer relations to other tech specs, the relation distinct from `parent` containment.
Product specs have no interactions file at all (absent, not empty). Edges are
outbound-only per the SPEC-018 / ADR-004 contract; the reverse direction is derived,
never authored twice.

### Reassembly and integrity

`spec show` reassembles the whole spec on read: it splices the identity TOML, the
prose body, the members resolved to their requirement entities by FK in advisory
order, and (for tech) the interaction edges — opening only this spec's directory and
the requirement directories reached by FK. `spec validate` builds the parsed
registry and runs the corpus FK-integrity pass: dangling or invalid member FKs,
descent targets, parent targets, and interaction targets, plus self-parent detection
and the `second_parent` parse-error classification. Validation is scoped to one spec
or run corpus-wide. This pass is the registry's headline value, and it is gated not
on scale but on the first authored cross-entity edge — which the spec composition is.

## Concerns

- **Orphan window on a non-atomic compose.** `spec req add` reserves a requirement
  before it appends the membership edge; a crash in the gap leaves an unmembered
  requirement. This is resolved by `validate` reporting it, not by a transaction.
- **FK integrity across trees.** Members, descent, parent, and interaction targets
  are foreign keys into separate directory trees; an unresolved or wrong-kind target
  is a defect the registry must surface as dangling vs invalid-kind, not parse past.
- **Tech-only field discipline.** The spine, the interaction table, and the
  flat fields are tech-only; a product spec carrying any of them is an invalid-kind
  finding, not a tolerated extra.

## Hypotheses

- **A requirement is a peer entity, not a sub-kind.** Modelling the requirement as
  its own addressable entity woven by a membership edge — rather than an inline
  block on the spec — is preferred so a requirement has a durable identity and the
  spec→requirement relation is a typed edge the registry can validate.
- **Two subtypes over one engine beats two engines.** Product and tech specs share
  enough structure (identity, members, reassembly) that one kind-blind materialiser
  serving both, diverging only in fileset and flat fields, is preferred over two
  parallel implementations.
- **A typed sister-TOML edge table is index-cheap.** Keeping members and
  interactions as small typed TOML files separate from prose means the integrity
  pass parses only the edge tables, never the bodies — the lever that keeps the
  full-corpus graph parse fast without a cache.

## Decisions

- **D1 — the requirement is a peer entity membered by a typed edge.** A `REQ-NNN` is
  its own directory; the spec→requirement relation is a `members.toml` row with a
  sticky `FR`/`NF` label and advisory order. The label is membership state on the
  edge, never the requirement's identity.
- **D2 — composition is non-transactional, integrity is the safety net.**
  `spec req add` reserves then members in sequence; the orphan window is closed by
  `validate` flagging the dangling state, not by a rollback the producer cannot
  guarantee across trees.
- **D3 — containment and peering are separate axes.** `parent` decomposition and
  `interactions` peering are distinct relations; neither is ever encoded as the
  other, so the architecture's shape is preserved in the edges themselves.
- **D4 — FK validation co-lands with the cross-entity edge.** The registry's
  integrity pass is gated on the first authored foreign key, not on scale; because
  the spec composition is what introduces the member and interaction edge tables,
  `spec validate` ships as part of the minimum spec bundle.
