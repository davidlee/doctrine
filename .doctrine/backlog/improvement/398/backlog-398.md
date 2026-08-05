# IMP-398: Knowledge record discoverability

## Motivation

The `/knowledge` skill now creates knowledge records (assumptions, decisions,
questions, constraints, evidence, hypotheses, concepts) as part of the design
workflow. These records link to slices, RFCs, ADRs, specs, and each other —
but they're hard to *find* after creation. An agent who wants to answer "what
do we assume about this slice?" or "what decisions shaped this spec?" has no
direct CLI path; they must either know the ids or grep raw files.

The aim is to make knowledge records as legible as the entities they annotate.

## Gaps (from exploratory survey 2026-08-04)

### 1. `knowledge list` has no `--type` filter

Rows carry `record_kind` but there's no flag. You can't list only decisions or
only assumptions.

### 2. No `--related-to` filter on `knowledge list`

No way to ask "what knowledge records are attached to SL-226?" (or RFC-008,
ADR-016, PRD-004, etc.). The relation edges exist (`shapes`, `governed_by`,
`references(concerns)`, `related`) but `knowledge list` can't filter by them.

### 3. `doctrine graph --kind` excludes the focus node

`doctrine graph SL-226 --kind ASM --kind DEC ...` fails with "SL-226 excluded
by --kind". You can't say "show me the knowledge neighbourhood of this slice."

### 4. No collective `KN` kind prefix

To include all knowledge subtypes in a graph or list you need 7 separate
`--kind` flags (`ASM DEC QUE CON EVD HYP CPT`).

### 5. `doctrine graph --label` takes exactly one label

Can't include all the edge types that connect knowledge ↔ entities
(`shapes`, `governed_by`, `references(concerns)`, `related`) in one graph.

### 6. No combined entity + knowledge view

`doctrine slice show` (or `adr show`, `spec show`, `rfc show`) doesn't
surface linked knowledge records. You stitch it manually.

## Worked instance (2026-08-05)

`QUE-206` (*is the interview substrate separable from slice design*) was
captured with `shapes` edges onto `PRD-019` and `SL-244`. What each surface
shows:

| view | result |
|---|---|
| `knowledge show QUE-206` | `shapes: [PRD-019, SL-244]` — correct |
| `spec show PRD-019` | **nothing** — no sign QUE-206 exists |
| `relation list --target PRD-019` | both inbound edges, correctly |

Three things this pins down that the gap list above states only abstractly:

- **The capability exists; the discoverability does not.** `doctrine relation
  list --target <ref>` already answers "what points at this?" — including the
  knowledge records. It lives in a third command family (`relation`), reachable
  from neither `knowledge` nor `<kind> show`, so nothing leads you to it from
  where the question arises. Part of this item is routing to an existing verb,
  not building a new one.
- **Gap 6 is the same defect ISS-306 reports, one entity-kind wider.** ISS-306
  scopes the missing inbound render to `knowledge show/inspect`. The identical
  hole is on `spec show` / `slice show` / `adr show` / `rfc show` — an entity
  never renders the knowledge that shapes it. Whatever derives reciprocity
  should be fixed once at the render seam both share, not twice.
- **The asymmetry has a direction that matters.** The entity is where the
  question gets asked ("what's unresolved about PRD-019?") and it is the side
  with no answer. The side that renders correctly is the side you already had
  the id for.

## Correction to the `--related-to` sketch

The design consideration below reads *"resolve the ref, walk outbound edges,
return knowledge records on the other end."* That will return nothing for the
common case. Relations are stored **outbound-only** (ADR-004) and knowledge
records author their own edges — `shapes` lives on `QUE-206` pointing at
`PRD-019`; `PRD-019` stores no edge at all. Walking the ref's outbound edges
finds knowledge records only when the *entity* happened to author the link.

So `--related-to <ref>` is a reverse scan — find knowledge records whose
outbound edges target the ref — or it reuses whatever index already backs
`relation list --target`. That is the same derivation ISS-306 needs, which is
further reason to treat the two as one fix.

## Scope

This item covers the **discoverability** surface — filtering, listing, and
graph projection — for knowledge records in relation to the entities they
annotate. It applies to all entity kinds that carry knowledge relationships
(slices, RFCs, ADRs, specs, backlog items — anything linkable to a knowledge
record).

## Design considerations (for the eventual slice)

- **`--type` on `knowledge list`**: straightforward — filter by `record_kind`.
- **`--related-to <ref>` on `knowledge list`**: reverse scan — find knowledge
  records whose outbound edges target the ref (see *Correction* above; walking
  the ref's own edges does not work under outbound-only storage). Should this be
  one-hop or transitive? Which edge labels qualify?
- **Graph `--kind` not filtering the focus**: the focus should anchor the
  neighbourhood regardless of its kind; `--kind` should filter *other* nodes.
- **`KN` kind prefix**: a convenience alias for all seven knowledge subtypes.
- **`--label` repeatable**: or a label-set syntax, so you can include multiple
  edge types.
- **Combined views**: could be `doctrine slice show --with-knowledge` or a
  separate `doctrine knowledge related-to <ref>` that renders the entity +
  its knowledge neighbourhood. Worth surveying actual usage patterns before
  committing.
- **Also consider**: `doctrine search` (should knowledge records surface in
  results?), `doctrine slice plan` / phase sheets (should linked knowledge be
  visible during planning?), and the `/design` skill workflow (should it
  auto-surface existing knowledge when resuming a design run?).

## Related

- ISS-306: `knowledge show/inspect` render no inbound reciprocity — same
  defect as gap 6, scoped one entity-kind narrower; fix together
- IDE-009: Knowledge read-path validation / lint verb
- IMP-120: Transitive impact query on relation graph
- QUE-206: the worked instance above — `shapes PRD-019`, invisible from
  `spec show PRD-019`
