# SPEC-019: Knowledge-record entity surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The knowledge-record entity surface is doctrine's epistemic-and-governance capture
layer realising **PRD-010**: the durable, typed, citable home for the assumptions,
decisions, questions, and constraints that *shape* work without being work. It is a
component of the entity engine (**SPEC-004**) — four `record_kind`s riding four engine
`Kind`s over the same kind-blind materialiser, the structural sibling of the backlog
surface (**SPEC-015**). All shared mechanism (identity, the atomic claim, id
allocation, edit-preserving status transition, and the scaffold/render pipeline) lives
in the parent container and is used here unchanged; this spec carries only what is
specific to this family: the four-kind discrimination, the **per-kind lifecycle
vocabularies**, the per-kind typed `[facet]` blocks plus the shared evidence structure,
prefix→kind resolution on the read path, the outbound relation seam over the SPEC-018
contract, and the cross-kind supersession lifecycle verb.

It is **forward-intent**: no code is shipped yet, so this spec describes the planned
mechanism. The per-slice `/design` owns the concrete fileset names, module placement,
and code-impact; what is fixed here is the durable architecture and the contracts that
outlive any one change. The truth/work boundary is the spine — this family records what
*shapes* slices, the backlog, and governance; it never becomes them.

## Responsibilities

Mirrors the structured `responsibilities` list: bind the four record kinds onto the
engine, carry the per-kind lifecycle vocabularies and hide-set, hold the per-kind typed
facets and the shared evidence structure, resolve kind from the id prefix, hold the
outbound relation seam over the contract, and own the cross-kind supersession verb.

### Four kinds, one engine

A knowledge record exists in four subtypes — assumption (`ASM`), decision (`DEC`),
question (`QUE`), constraint (`CON`) — each a data-valued engine `Kind` with its own
tree under `.doctrine/knowledge/<kind>/` and its own reservation namespace, so `ASM-001`
and `DEC-001` coexist with independent counters. The subtypes share one
`record-NNN.{toml,md}` fileset and an `NNN-slug` symlink alias, diverging only in their
prefix, their lifecycle vocabulary, and the `[facet]` block the scaffold seeds. The
discrimination is the **one-entity, one-schema** rule (PRD-010 §4): `record_kind` is a
facet on one entity, never a fork of parallel per-kind schemas — the same discipline the
backlog applies to its five item kinds. The `record_kind` enum is the `clap::ValueEnum`
positional on capture and the kebab serde of the stored `record_kind` field.

```text
.doctrine/knowledge/<kind>/NNN/
  record-NNN.toml      # identity, record_kind, status, summary, tags, [facet], [evidence], [relations]
  record-NNN.md        # prose body
```

This is the **divergence from the backlog**: the backlog's five kinds share one closed
status vocabulary and only the risk kind carries a `[facet]`; here every kind carries
both its **own** lifecycle vocabulary and its **own** facet shape. The shared
materialiser is unchanged — the variation is data (per-kind status set, per-kind facet
seed) keyed by `record_kind`, not a parallel implementation.

### Per-kind lifecycle vocabularies

Unlike the backlog's single `Status` enum, lifecycle here is **keyed by `record_kind`**
(PRD-010 §6, the "lifecycle belongs to the kind" principle):

- **assumption** — `held → testing → validated | invalidated | obsolete`
- **decision** — `proposed → accepted | rejected | superseded`
- **question** — `open → answered | obsolete`
- **constraint** — `active → waived | superseded | retired`

Capture seeds the kind's default first state (held / proposed / open / active). Status
is hand-settable and ungated, as slices, ADRs, specs, and backlog items ship today. The
transition verb validates the target against `vocab(record_kind)` and **refuses** a
foreign-kind state rather than coercing it — the validation is kind-relative, the one
mechanism this surface adds that the backlog's closed-global-enum model does not need. A
per-kind `is_terminal` predicate (`validated`/`invalidated`/`obsolete` ·
`rejected`/`superseded` · `answered`/`obsolete` · `waived`/`superseded`/`retired`)
drives the default-list hide-set through `listing::retain`, exactly as the backlog's
`is_terminal` does — terminal records drop from the default survey but stay addressable
under `--all` or an explicit `--status`. These predicates are deliberately neither the
backlog's nor the slice's: each family owns its own terminal set.

### Typed facets and the evidence structure

Each kind seeds its own typed `[facet]` block; there is no untyped frontmatter bag
(PRD-010 NF-001):

- **assumption** — the claim, a `confidence` (low / medium / high), a `basis`
  (observation / prior-art / design-inference / external-source / operator-judgement), a
  validation plan, and the validated/invalidated by-and-on.
- **decision** — context, choice, alternatives, rationale, consequences, and the decided
  by-and-on.
- **question** — the question, why it matters, the answer, and the answered by-and-on.
- **constraint** — the statement, its `source` (canon / adr / external / technical /
  legal / compatibility / operator), what it applies to, and the waiver reason and
  waived by-and-on.

`confidence` is an **assumption-only** facet, not a common field (PRD-010 OQ-004) — the
common schema is identity, summary, and tags only. The closed enums (`confidence`,
`basis`, constraint `source`) ride the same `"" -> None` optional seam the backlog's
`Resolution`/risk axes use: a tolerant raw parse reads them as strings, a separate
`validate` pass maps empty to absent and parses any non-empty token to its variant,
erroring on an unknown one. This is the parent container's three-layer parse model,
specialised for seeded-empty optionals.

Evidence is a single **shared** minimal support structure across all four kinds — a
typed `[evidence]` table of `supports` / `contradicts` / `notes` citations (PRD-010
NF-001). It is **not** its own entity kind and **not** a free-form blob; in v1 it is a
minimal citation structure, never queryable graph machinery (PRD-010 §2 out-of-scope).

### Prefix→kind resolution on the read path

Capture takes `record_kind` explicitly and derives the prefix from the engine `Kind`
(the single source), as the backlog does. The **read** path inverts this: `show`,
transition, relate, and supersede name a record by id and resolve `record_kind` from the
id **prefix** (`ASM`/`DEC`/`QUE`/`CON` → the kind), so one verb set dispatches across all
four prefixes and selects the right lifecycle vocabulary and facet shape. Identity — the
kind prefix plus number — is permanent; the slug is never authoritative; and
`record_kind` is fixed at capture and never silently changes (PRD-010 NF-003). This
prefix→kind resolver is the half PRD-010 §2 explicitly handed to the technical spec.

### The outbound relation seam

A record's relations follow the **cross-corpus relation contract specified in SPEC-018**
(governed by ADR-010, composing ADR-004's outbound-only rule) — this spec does not
re-tell that model. The record's own surface within it: outbound forward links to
backlog items, risks, slices, specs, ADRs, requirements, and drift records, plus the
**spawn-work** edge (e.g. `ASM-001 → RSK-004`, `DEC-003 → SL-020`) authored once on the
record (the side that shows what it affects); the item's inbound origin is **derived** by
the registry scan, never stored on the item (PRD-010 §6, ADR-004). The seam is always
present, even empty, so linkage and spawn machinery have a stable attachment point.
Relating or spawning never mutates the linked or spawned artefact's lifecycle. Admitting
`knowledge_record` source kinds and their legal targets into `RELATION_RULES`
(`src/relation.rs`) is the contract-side work this surface depends on; the vocabulary is
authored there, never transcribed here (storage rule).

### Cross-kind supersession

Supersession is a **replacement edge distinct from a relation** (PRD-010 §6), admitted
only when the successor becomes the authoritative continuation of the predecessor. It is
the `supersedes` / `superseded_by` pair — the ADR-004 §5 reverse carve-out, co-written on
both records, sanctioned because the supersession already moves the predecessor to a
terminal status and rewrites its file, so the reverse edge adds zero marginal coupling.
Per SPEC-018 this pair is `LifecycleOnly` in the relation contract — never plain-`link`,
owned by a **transactional supersede verb**. That verb is the cross-kind lifecycle axis
the corpus has been deferring as **IMP-006** (cited by SPEC-005 and SPEC-018 as the
unbuilt owning verb for governance supersession); this surface is its first real
consumer. The verb co-writes both edges atomically, moves the predecessor to a terminal
status valid for **its own** kind without changing that kind, and admits a `record_kind`
crossing only along the **§6 allowed matrix** when the successor is the authoritative
continuation — refusing a reopening direction (`constraint → assumption`, `decision →
question`) as a relation, not a supersession.

## Concerns

- **Kind-relative status validation.** The transition vocabulary is selected by
  `record_kind`; the known-set per kind must mirror that kind's status enum exactly (a
  per-kind analogue of the backlog's `backlog_statuses_matches_the_variants` guard). The
  failure mode is a kind's known-set drifting from its enum, or a transition validated
  against the wrong kind's vocabulary after a prefix→kind misresolution.
- **Disjointness with the backlog.** No `record_kind` may be admitted as a
  `backlog_item.item_kind` and no `item_kind` as a `record_kind` (PRD-010 §4); the two
  families are disjoint by the work-intake membership test. The two prefix sets and two
  `KINDS` partitions must not collide.
- **Supersession atomicity.** The two-record co-write plus the predecessor's terminal
  transition must be one transactional unit, or a crash leaves a half-written lineage
  (forward edge without reverse, or reverse without the terminal status). This is why
  supersession is a bespoke verb, not two plain `link`s.
- **Matrix enforcement is semantic, not structural.** The §6 matrix bounds *which*
  cross-kind crossings are valid; the verb must enforce it, since the relation contract's
  kind-legality check alone cannot distinguish authoritative continuation from mere
  influence.
- **Behaviour preservation.** This family rides the shared entity scaffold; introducing
  it must leave the existing slice / ADR / spec / backlog / memory suites green unchanged
  (PRD-010 NF-002).

## Hypotheses

- **One entity discriminated by `record_kind` beats four schemas.** The four kinds share
  enough structure (fileset, identity, evidence, relations, reassembly) that one
  kind-blind materialiser serving all four — diverging only by prefix, lifecycle
  vocabulary, and facet seed — is preferred over four parallel implementations, exactly
  as the backlog's single-entity discipline (SPEC-015) proved for its five kinds.
- **Per-kind lifecycle is data, not a new engine.** The divergence from the backlog —
  four status vocabularies instead of one — is a `record_kind`-keyed lookup over the same
  edit-preserving transition seam, not a second transition mechanism. The engine stays
  kind-blind; the kind table carries the per-kind status set, as `integrity::KINDS`
  already carries a stateful status set per kind.
- **Supersession reuses IMP-006, not a bespoke fork.** The transactional supersede verb
  is the same cross-kind lifecycle axis governance supersession needs; building it for
  this family and for ADR/POL/STD as one verb avoids the parallel implementation SPEC-018
  OD-3 warns against.

## Decisions

- **D1 — `parent = SPEC-004`, `descends_from = PRD-010`; the thin-not-anaemic component
  shape.** Identity, claim, id allocation, edit-preserving transition, and the
  scaffold/render pipeline are the parent container's and are restated nowhere here; this
  component owns only the four-kind discrimination, the per-kind lifecycles and facets,
  the evidence structure, prefix→kind resolution, and the supersession verb. It is the
  structural sibling of SPEC-005 (ADR) and SPEC-015 (backlog).
- **D2 — lifecycle and facets are `record_kind`-keyed, not global.** Each kind owns its
  status vocabulary, its `is_terminal` set, and its `[facet]` shape; no single shared
  status set is imposed and no kind is forced into another's states. This is the
  deliberate divergence from the backlog's one-closed-enum model, demanded by PRD-010's
  "lifecycle belongs to the kind" principle.
- **D3 — relations follow the SPEC-018 contract; supersession is `LifecycleOnly`.** The
  outbound relation seam is authored over `RELATION_RULES`, outbound-only with reverse
  derived (ADR-004); the `supersedes`/`superseded_by` pair stays the sanctioned typed
  carve-out, owned by the transactional supersede verb (IMP-006), never plain-`link`. The
  cross-corpus model, vocabulary, and validation policy live in SPEC-018, not here.
- **D4 — capture takes the kind, the read path resolves it from the prefix.** Capture is
  kind-explicit (the prefix derives from the engine `Kind`); `show`/transition/relate/
  supersede are kind-implicit, resolving `record_kind` from the id prefix so one verb set
  serves all four kinds. `record_kind` is fixed at capture and identity is permanent.
- **D5 — evidence is a shared minimal typed structure, not a kind.** All four kinds share
  one `[evidence]` table of typed `supports`/`contradicts`/`notes` citations; it is never
  its own entity kind and never a free-form blob, and graph/search machinery over it is
  out of v1 (PRD-010 §2).

## Open Questions

- OQ-1 — the **memory-interaction seam** is deferred to v2 and out of this spec's
  mechanism. PRD-010 §3 fixes the record↔memory boundary *policy* (govern → record;
  recall → memory; promote by linking, never mutate), and PRD-010 OQ-006 (promotion path,
  memory → record) and OQ-007 (recall bridge, `memory retrieve` surfacing record refs)
  hold the open design. Both need the memory↔record link to cross the named-identity /
  numbered-kind divide — memory is `mem_<uid>`, while the SPEC-018 contract targets
  numbered kinds in `integrity::KINDS` — so the seam is cross-spec (SPEC-007 + SPEC-018 +
  this surface) and is not specified here. This spec assumes only that a record is a
  *citable* target; how memory cites or is promoted into one is owned elsewhere when v2
  opens it.
