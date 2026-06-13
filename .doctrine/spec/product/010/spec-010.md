# PRD-010: Epistemic and Governance Records

## 1. Intent

Work is not the only thing worth keeping. Alongside the units of work the backlog
captures, every project accumulates *truths it is acting on but has not proven*: an
assumption held while progress depends on it, a lightweight decision taken below the
weight of an ADR, an open question that bounds what can be settled, a durable
constraint that quietly shapes every valid solution. Today these have no home. They
are buried in design prose, misfiled as backlog risks, scattered across chat history,
or simply forgotten — and so they are rediscovered, re-litigated, or silently
violated.

The backlog (PRD-009) deliberately excludes them: a backlog item *is* a latent unit of
work, and these records fail the work-intake membership test. Their lifecycle is
epistemic or governance-oriented, not work-oriented — an assumption is *held* then
*validated* or *invalidated*; a decision is *proposed* then *accepted* or *superseded*;
a question is *open* then *answered*; a constraint is *active* then *waived* or
*retired*. PRD-009 OQ-005 recorded the gap as a decision, not an omission, and pointed
here.

This capability is the home OQ-005 anticipated: a durable, typed, citable family of
**epistemic and governance records** for the claims, choices, questions, and rules
that shape work without being work. Its value is that what is *believed, decided,
uncertain, or constrained* becomes a first-class, queryable artefact — held below ADR
weight and above disposable notes — that can be tracked, cited, revised, superseded,
and related to the work it shapes, rather than re-derived from memory.

## 2. Scope

In scope:

- A durable entity family — `knowledge_record` — for epistemic and governance records,
  discriminated by a `record_kind` facet.
- The four initial record kinds: **assumption**, **decision**, **question**,
  **constraint**.
- A common record schema (identity, summary, tags) plus a typed, kind-specific facet
  block for each kind. Confidence is an assumption-only facet, not a common field; a
  record's scope over other artefacts is expressed through typed relations, not a field.
- A distinct lifecycle vocabulary per kind — truth lifecycles, not the work-intake
  lifecycle.
- A minimal evidence support structure (`supports`, `contradicts`, `notes` of
  citations) attached to a record.
- An outbound relation seam linking a record to backlog items, risks, slices, specs,
  ADRs, requirements, and drift records.
- Capture, inspect, survey, and kind-valid status transition behaviour.
- Supersession, where a record replaces an aged or invalidated predecessor.
- Promotion/spawn semantics: a record can create or relate to backlog work, but is not
  itself backlog work.

Out of scope:

- **A generic `finding` record kind.** It is deliberately excluded from v1 (§3, §8) —
  it lacks a distinct lifecycle, stable citation value, and low semantic collision, and
  every legitimate "finding" already has a home (see the redirect in §3).
- **Replacing ADR.** Architecturally significant decisions remain ADRs; this family
  holds only lower-stakes, narrower-scope decisions.
- **Replacing the backlog risk.** A risk stays a backlog kind: it is uncertain future
  *harm* that may require mitigation, acceptance, expiry, or scoped work — work-intake
  adjacent. A `knowledge_record` may *link* to a risk; it does not subsume it.
- **A full evidence graph / knowledge-base search** — evidence is a minimal citation
  structure in v1, not queryable graph machinery.
- **Automatic inference** of assumptions, decisions, or constraints from prose.
- **Approval workflow** beyond explicit, hand-settable status transitions.
- **Importing historical assumptions / decisions** from existing documents.
- The technical design and storage mechanism — engine, fileset descriptors, reservation
  primitive, prefix→kind resolution (those belong to `/spec-tech` and per-slice
  `/design`).

Boundary: this family owns **truth-management and governance state** — the claims,
choices, questions, and rules that shape work. It does not own execution (that is a
slice and its phase state), prioritised work intake (that is the backlog), or scoped
implementation (that is a slice). It records what *shapes* those things; it does not
become them. The membership test below is the arbiter.

## 3. Principles

- **Truth is not work.** An assumption, decision, question, or constraint may *spawn*
  work, but it is never itself a backlog item. The spawned work is linked, not
  conflated; the record keeps its own lifecycle.
- **ADR keeps the high ground.** Architecturally significant decisions remain ADRs.
  This family handles decisions below that threshold; when scope or consequence crosses
  the architectural line, the decision escalates to an ADR rather than living here.
- **Lifecycle belongs to the kind.** Unlike the backlog's single uniform status
  vocabulary, this family does **not** force one status set across all kinds. Each kind
  carries the lifecycle its meaning demands; collapsing them would obscure meaning.
- **Typed records, not loose notes.** Every kind has explicit typed facet fields and an
  enumerated lifecycle. There is no untyped frontmatter bag — the storage rule holds.
- **Relations make consequences visible.** A record shows what it affects: the risks,
  backlog items, slices, specs, ADRs, requirements, and drift records downstream of it.
- **Supersession is normal, and it may cross kinds.** These records are expected to
  age, be replaced, or be invalidated; supersession is a first-class, traceable
  transition, not a deletion. It is the answer to "what should I read now — is this
  still authoritative, and what replaced it?", which a plain relation cannot give.
  Supersession is *replacement lineage*: the successor takes over the authoritative
  role for the same underlying claim, question, choice, or rule — and may be a
  different kind from its predecessor (a believed assumption hardening into a
  constraint, an open question answered by a decision). Mere influence, evidence,
  consequence, or origin is a relation, **not** supersession.
- **Membership test — what belongs here.** A candidate belongs to this family if its
  primary lifecycle is one of: a belief becoming validated or invalidated; a question
  becoming answered or obsolete; a decision becoming accepted, rejected, or superseded;
  a rule becoming active, waived, superseded, or retired. A candidate does **not**
  belong here if it is primarily a unit of work to be triaged and promoted into a slice
  — that is a backlog item.
- **Knowledge is not recall — the memory boundary.** Memory (PRD-004) and this family
  are adjacent homes for durable knowledge, separated by *governance need*, not by
  topic. The decision rule: if a unit must be **cited, linked, transitioned, superseded,
  or used to govern work**, it is a `knowledge_record`; if it only needs to be
  **recalled as scoped knowledge**, it is a memory. A small durable fact that needs no
  lifecycle, evidence, or relations stays a memory — the *fact sink* — which is the same
  redirect the §3 table already makes ("a project fact worth retaining → memory"; "a
  fact supporting/refuting a claim → evidence on an assumption"). A memory that later
  becomes load-bearing is **promoted by linking, never mutated in place**: the memory
  keeps its identity, a kind-correct record is minted, and the two are related — the
  mirror of the record→backlog spawn (truth is not work; recall is not governance). The
  mechanics of that promotion and of surfacing records during recall are deferred
  (OQ-006, OQ-007).
- **No generic `finding`.** A "finding" has no distinct lifecycle, decays against repo
  state without heavy provenance, and collides with audit/inquisition output. There is
  no `finding` record kind. A finding is always one of its real homes:

  | A finding that is…                       | belongs to                                      |
  | ---------------------------------------- | ----------------------------------------------- |
  | review output needing disposition        | a review / audit / inquisition finding          |
  | a project fact worth retaining           | memory                                          |
  | a fact supporting / refuting a claim      | evidence on an assumption                       |
  | the answer to a question                  | the question's `answer`, status → `answered`    |
  | the reason for a choice                   | a decision's rationale / alternatives           |
  | the source of a rule                      | a constraint's source                           |
  | an implementation-local observation       | slice / design / phase notes                    |
  | an actionable consequence                 | a backlog item                                  |
  | uncertain future harm                     | a backlog risk                                  |

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below. This
section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- The family is one `knowledge_record` entity discriminated by a `record_kind` facet;
  no implementation may introduce parallel per-kind schemas or directories. (Mirrors
  the backlog's single-entity discipline and `entity-model.md`'s "fewer entity kinds,
  more facets" direction.)
- The kind set is exactly the four initial kinds — assumption (`ASM`), decision
  (`DEC`), question (`QUE`), constraint (`CON`), three-character per-kind prefixes —
  and may not be extended without a reserved id. `finding` is excluded (§3, §8).
- No `knowledge_record` kind may be admitted as a `backlog_item.item_kind`, and no
  backlog `item_kind` may be admitted as a `record_kind`; the two families are disjoint
  by the work-intake membership test.
- Each kind must have a distinct lifecycle vocabulary; no kind is forced into another
  kind's states, and no single shared status vocabulary is imposed across the family.
- Each kind must hold typed, enumerated facet fields; there is no untyped frontmatter
  catch-all. Evidence is a minimal typed support structure (`supports` / `contradicts`
  / `notes` of citations), never its own entity kind and never a free-form blob.
- A decision admitted here must be below the ADR threshold; an architecturally
  significant decision is an ADR, not a `knowledge_record`.
- A constraint admitted here must bound more than one artefact or need independent
  identity; a constraint local to a single requirement stays in requirement/spec space
  and is linked, not duplicated.
- The capability must reuse the shared entity/scaffold substrate; extending it must not
  regress the existing entity callers (slice, ADR, spec, backlog, memory).
- A record's status is hand-settable and ungated, consistent with how slices, ADRs,
  specs, and backlog items ship today.
- Supersession is a replacement edge distinct from a relation, and is admitted only
  when the successor becomes the authoritative continuation of the predecessor — never
  for mere influence, evidence, consequence, or origin. It may cross `record_kind`
  boundaries only along the allowed matrix (§6); reopening directions (e.g.
  `constraint → assumption`, `decision → question`) are reconsideration, not
  replacement, and are not supersession.

Invariants:

- Every `knowledge_record` is on an epistemic or governance lifecycle, never the
  work-intake lifecycle; nothing that fails the membership test is ever a
  `knowledge_record`, and a record never becomes a backlog item by spawning work.
- A record's identity — its kind prefix plus number — is permanent; the slug is never
  authoritative and tooling resolves a record only by its id.
- A record's `record_kind` is fixed at capture; a record never silently changes kind.
  Cross-kind supersession is lineage between two records, not mutation: the predecessor
  keeps its kind and moves to a terminal status valid for that kind, while the successor
  is a separate record of its own kind.
- A record is only ever in a state drawn from its own kind's lifecycle vocabulary.
- Every facet and evidence entry is typed, enumerated storage; untyped record data
  never persists.
- The relation seam is always present, even when empty, so linkage and spawn machinery
  have a stable attachment point.
- A record in a terminal state remains durably stored and addressable — "hidden by
  default" is a view, never deletion.
- When a record relates to or spawns another artefact, each artefact remains
  authoritative for its own lifecycle; relating never mutates the other's state.

## 5. Success Measures

- Held truths stop leaking: a contributor can record an assumption, decision, question,
  or constraint the moment it surfaces, and it survives outside the conversation,
  design doc, or commit that produced it.
- Consequences are visible: from a record, a reader can see the risks, backlog items,
  slices, specs, ADRs, requirements, and drift records it shapes — and from those
  artefacts, trace back to the belief or rule behind them.
- Beliefs are testable, not permanent: an assumption can move from held to validated or
  invalidated with its evidence attached, so stale beliefs are caught rather than
  silently relied upon.
- The decision/ADR boundary holds: lightweight decisions get stable identity here
  without diluting the ADR record, and significant decisions still escalate to ADRs.
- Truth and work stay separated: no assumption, decision, question, or constraint ends
  up as a backlog item, and a record that spawns work keeps a durable origin link to it
  without becoming work itself.
- A reader (human or agent) can trust that every field is typed and that a record's
  kind and identity are stable — no untyped bags, no kind drift, no id churn.

Acceptance gates:

- Capturing each of the four kinds yields a durable record with a reserved kind-correct
  id and the kind's default lifecycle state and typed facets.
- A kind-valid status transition is atomic and edit-preserving — it round-trips without
  dropping comments or unknown keys — and a state outside the record's kind vocabulary
  is rejected.
- Inspection renders the common fields, the kind facets, the evidence block, and the
  relations; a survey filters by kind, status, and tag and hides terminal records by
  default while keeping them addressable.
- A record can link to a backlog item and a slice without changing either artefact's
  lifecycle, and can spawn a backlog item while retaining a durable origin relation.
- An assumption, decision, question, or constraint is rejected as a `backlog_item`
  kind, and ADR remains the required home for architecturally significant decisions.
- Extending the capability leaves the existing slice / ADR / spec / backlog / memory
  suites green unchanged.

## 6. Behaviour

Primary flow — capture: a contributor names a record's kind and title; the system
reserves the next free id in that kind's namespace (`ASM`/`DEC`/`QUE`/`CON-NNN`),
materialises the record seeded with the kind's typed facet defaults and its default
lifecycle state, writes an empty relation seam and evidence block, and reports where it
lives.

Primary flow — inspect: a contributor names a record id; the system detects the kind
from the prefix and renders the record's identity, kind, lifecycle state, summary,
typed kind facets, evidence, and relations.

Primary flow — survey: an operator asks for the records and receives a set they can
narrow by kind, status, and tag; records in a terminal state are hidden by default and
revealed on request.

Primary flow — transition: a contributor moves a record to another state within its
kind's lifecycle vocabulary; the change is atomic and preserves the rest of the record
verbatim. A state outside that kind's vocabulary is rejected rather than written.

Primary flow — relate: a contributor links a record to backlog items, risks, slices,
specs, ADRs, requirements, or drift records. The record stays authoritative for its own
state; the linked artefact stays authoritative for its own lifecycle.

Primary flow — spawn work: a contributor creates a backlog item from a record; the
source record records the spawned item as an outbound relation — for example
`ASM-001 → RSK-004`, `QUE-002 → CHR-011`, `DEC-003 → SL-020`. Per ADR-004 the edge is
authored once, on the record (the side that shows what it affects); the item's inbound
origin is derived by the registry scan, not stored on the item. The record does not move
onto the work lifecycle and does not change kind.

Primary flow — supersede: a contributor records that a newer record takes over the
authoritative role of an older one. The supersession is written on both records
(`supersedes` on the successor, `superseded_by` on the predecessor); the predecessor
moves to a terminal status valid for its own kind while keeping that kind, and the
successor stays a separate record of its own kind. Co-writing the reverse `superseded_by`
is the ADR-004 §5 carve-out — sanctioned because supersession already moves the
predecessor to a terminal status and rewrites its file, so the reverse edge adds zero
marginal coupling. It is not a general licence for bidirectional relations; every
non-lifecycle relation stays outbound-only with its reverse derived. The successor may be a different kind from the predecessor — the canonical
case is an assumption hardening into a constraint:

```toml
# ASM-004              # CON-002
record_kind = "assumption"   record_kind = "constraint"
status = "validated"         status = "active"
[relations]                  [relations]
superseded_by = ["CON-002"]  supersedes = ["ASM-004"]
```

Cross-kind supersession is bounded by an allowed matrix — admitted only where the
successor is the authoritative continuation of the predecessor, never for mere
influence/evidence/consequence/origin:

| Predecessor  | May be superseded by                     | Reading                                          |
| ------------ | ---------------------------------------- | ------------------------------------------------ |
| `assumption` | assumption, decision, constraint         | claim revised, decided away, or hardened to rule |
| `question`   | question, decision, constraint, assumption | reframed, answered by choice/rule, or made a working claim |
| `decision`   | decision, constraint                     | choice revised, or hardened into a standing rule |
| `constraint` | constraint, decision                     | rule revised/retired; decision only when a rule is consciously waived and replaced by a choice |

Reopening directions — `constraint → assumption`, `decision → question`, and the like
— are reconsideration, not replacement, and are modelled as relations, not
supersession. (ADR escalation of a decision is likewise a relation, not supersession,
unless ADRs join the same replacement model — open.)

Kind lifecycles and facets — each kind resolves to its own vocabulary and typed facet
shape:

- **assumption** — a claim treated as true enough to proceed, but not yet proven.
  Lifecycle `held → testing → validated | invalidated | obsolete`. Facets: the claim, a
  **confidence** (low / medium / high) registering how firmly it is held, its basis
  (observation / prior art / design inference / external source / operator judgement), a
  validation plan, and the validated/invalidated by-and-on records.
  Boundary: if it implies uncertain future harm, link a risk; if validating it needs
  work, spawn a backlog item; if it settles into a durable rule, supersede it with a
  constraint.
- **decision** — a chosen direction below ADR weight, or a local decision inside a
  spec/slice needing stable identity. Lifecycle `proposed → accepted | rejected |
  superseded`. Facets: context, choice, alternatives, rationale, consequences, and the
  decided by-and-on. Boundary: if architecturally significant, use an ADR; if it is
  mere implementation progress, keep it in slice/phase notes; if it creates work, link
  to backlog or slice.
- **question** — an unresolved knowledge gap. Lifecycle `open → answered | obsolete`.
  Facets: the question, why it matters, the answer, and the answered by-and-on.
  Boundary: if answering needs investigation, spawn a backlog item; if answered with a
  durable claim, link or supersede with the relevant record; if it encodes a design
  choice, resolve it through a decision.
- **constraint** — a rule, limitation, obligation, compatibility requirement, or
  external condition that bounds valid work. Lifecycle `active → waived | superseded |
  retired`. Facets: the statement, its source (canon / ADR / external / technical /
  legal / compatibility / operator), what it applies to, and the waiver reason and
  waived by-and-on. Boundary: if it is a chosen architectural rule, consider an ADR; if
  it is a product requirement, keep it in requirement/spec space and link it; if
  satisfying it needs work, spawn a backlog item.

Edge cases and guards: an empty store yields the first id in each kind's namespace; a
record in a terminal state stays addressable and can be superseded; relating or
spawning never mutates the linked or spawned artefact's lifecycle; a hand-edited slug
may go stale while the canonical id remains authoritative; a transition to a foreign
kind's state is rejected, not coerced.

## 7. Verification

Verification confirms that held truths are durable and citable, that each kind carries
its own lifecycle and typed facets without forking the model, and that the truth/work
boundary holds — without binding the spec to a particular implementation.

Capture is proven by confirming each of the four kinds (REQ-060) produces a durable
record with a reserved kind-correct id, the kind's default lifecycle state, its typed
facet defaults, and empty relation and evidence seams, persisting across reads.
Inspection is proven by confirming kind is resolved from the id prefix and that
identity, kind facets, evidence, and relations render, including the empty seams
(REQ-061). The lifecycle is proven by confirming a transition stays within the record's
kind vocabulary, that a foreign-kind state is rejected, and that the change is atomic
and edit-preserving — round-tripping without dropping comments or unknown keys
(REQ-062). Survey is proven by confirming records filter by kind, status, and tag, with
terminal records hidden by default yet addressable under an explicit reveal (REQ-063).

The relation seam is proven by confirming a record links to backlog items, risks,
slices, specs, ADRs, requirements, and drift records without changing any linked
artefact's lifecycle (REQ-064). The truth/work boundary is proven by confirming a
record can spawn a backlog item — the spawn edge authored once on the record, the item's
inbound origin derived per ADR-004 — while the record itself neither changes kind nor
moves onto the work-intake lifecycle (REQ-065),
and by confirming an assumption, decision, question, or constraint is rejected as a
`backlog_item` kind. Supersession is proven by confirming a record can supersede and be
superseded — recording the link both ways and moving the superseded record to a
terminal status valid for its own kind without changing that kind — that a cross-kind
crossing is accepted only along the allowed matrix when the successor is the
authoritative continuation, and that a reopening direction is rejected as supersession
(REQ-066).

The single-entity, typed-storage discipline is proven by confirming every kind,
including its facet variation, is carried by one `knowledge_record` entity discriminated
by `record_kind` with no parallel schema, and that every facet and evidence entry is
typed with no untyped bag (REQ-067). The agent-readability obligation is proven by
confirming a record is addressable by a durable id whose kind prefix and number are
permanent, that the slug is never authoritative, and that `record_kind` is fixed at
capture (REQ-069). The behaviour-preservation obligation on the shared substrate is
proven by the existing slice / ADR / spec / backlog / memory suites staying green
unchanged (REQ-068).

Where a check must cite a specific obligation, reference the durable requirement entity
(`REQ-NNN`), never the mobile `FR-`/`NF-` membership label. Coverage of the functional
and quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- OQ-001 — Should `finding` ever earn a place in this family? It is excluded from v1
  (§3) because it lacks a distinct lifecycle, decays against repo state without heavy
  provenance, and collides with audit/inquisition output. Recorded so the exclusion is
  a decision, not an omission; revisit only if evidence-traceability demands a kind that
  the question `answer`, assumption evidence, decision rationale, memory, and review
  findings cannot jointly cover.
- OQ-006 — How is a memory *promoted* into a `knowledge_record` when it becomes
  load-bearing? The §3 memory boundary fixes the *policy* — promote by linking, never
  mutate in place, so the memory keeps its `mem_…` identity and a kind-correct record is
  minted and related. The *mechanism* is deferred: it is the mirror of the record→backlog
  spawn, but the memory↔record link crosses the named-identity / numbered-kind divide
  (memory is `mem_<uid>`, the relation contract targets numbered kinds in
  `integrity::KINDS`), so it needs memory admitted as a relation endpoint or memory's own
  typed relation seam. Cross-spec (PRD-004 / SPEC-007 / SPEC-018 / SPEC-019); out of v1.
- OQ-007 — Should `memory retrieve` surface relevant `knowledge_record` refs as recall
  context? The *recall bridge*: records appear as attributed citations/links, never
  absorbed into memory text (PRD-004's "attributed data, never instruction" extended to a
  new target). New scope on the memory side, gated by the same memory↔record link OQ-006
  needs; deferred to v2.
(OQ-002 — cross-kind supersession — is resolved: supersession is a first-class
replacement edge and may cross `record_kind` boundaries within the family. It is valid
only when the successor becomes the authoritative continuation of the predecessor — not
merely when one record is related to, caused by, supported by, or spawned from the
other. The predecessor moves to a terminal status valid for its own kind and never
changes kind; the successor is a separate record of its own kind. The allowed crossings
are bounded by the §6 matrix; reopening directions (`constraint → assumption`,
`decision → question`) are reconsideration modelled as relations, not supersession. The
§6 assumption→constraint case is the canonical cross-kind supersession, not a link.)
(OQ-003 — record↔artefact reciprocity — is resolved by ADR-004 (the same rule PRD-009
OQ-004 cites): relations are stored **outbound-only** on the durable record;
reciprocity is **derived** — an artefact's inbound references from the records that
shape it are computed by the registry scan, never authored on the artefact. Exactly one
side authors each relation (the record, here). Inbound completeness belongs to the
registry-backed inspect surface, not the one-way reader. Leans OQ-005 toward
relation-not-field: if scope is a typed relation, "what is scoped to me" is derived like
any other inbound reference.)
(OQ-004 — `confidence` placement — is resolved: confidence is an **assumption-only
facet**, not a common field. It measures the "true enough to proceed, not yet proven"
gap intrinsic to an assumption. The other kinds carry their nuance elsewhere — a
question's weight in `why it matters`, a decision's in rationale/consequences — and a
`proposed → accepted` decision or an `active` constraint is authoritative once in state,
so a confidence scalar there would be malformed. The common schema is identity, summary,
and tags only.)
(OQ-005 — `scope` field vs relation — is resolved by ADR-004: there is **no free-text
`scope` field**. A record's scope over an artefact is a typed outbound relation (the seam
already carries it); "what is scoped to me" is derived like any other inbound reference,
so a `scope = "slice-020"` string would be a stringly-typed, unvalidatable shadow of that
relation. The one scope that is not an artefact pointer — a rule bounding a path/glob
region like `src/**` — is out of v1; if it is ever needed it borrows memory's typed
`scope.globs` shape, never a prose field.)
