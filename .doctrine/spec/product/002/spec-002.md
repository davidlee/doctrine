# PRD-002: Specifications

## 1. Intent

A governed codebase accumulates intent faster than it can write it down. The
durable *what* and *why* of an enduring capability — the need it serves, the
boundary it holds, the obligations it must satisfy, and the bar by which it is
judged correct — too often lives only in a single change's design, in chat
history, or in a long prose document that nobody trusts to be current. When that
intent is missing, an agent scoping the next change has nothing upstream to
descend from: it re-invents the product contract, or worse, infers one and gets
it wrong.

A **specification** answers this need. It is the durable, evergreen statement of a
capability's product intent — distinct from any one change that touches it and
distinct from the technical design that realises it. Its value is that intent
becomes a first-class artefact a later agent can read, scope a slice against, and
verify a shipped change for: the product contract is stated once, where it
endures, instead of being rediscovered every time work begins.

The specification's most load-bearing decision is *how its requirements are
held*. A requirement is referenced from many places — a spec's membership, future
coverage edges, change records, drift ledgers — so it must be a stable,
addressable thing, not a row of prose buried in a document. This capability exists
to make requirements first-class entities and to keep relational requirement data
out of narrative, where it would silently drift from the structured copy.

## 2. Scope

In scope:

- Declaring an evergreen capability's product intent — its need, value, scope
  boundary, and the principles that constrain every valid realisation — distinct
  from any single change and from the technical design.
- Carrying functional and quality requirements as durable peer entities with their
  own stable identity, each membered by a spec under a human-facing label.
- Reassembling a fragmented spec — identity, prose, and its membered requirements —
  into one readable whole on demand.
- Surveying the specifications in a project and checking referential integrity
  across the corpus.
- Two coordinated spec families: product intent and the technical specification
  that descends from it.

Out of scope:

- The technical *how* of a capability — architecture, mechanism, and per-change
  design — which belong to the technical spec and to per-change design, not to the
  product intent.
- A single bounded change's scope and lifecycle — that is a slice, which descends
  *from* a spec rather than living inside one.
- Runtime execution progress and agent-orchestration state — disposable runtime
  tier, never the authored contract.
- Coverage computation, cross-spec requirement reuse, capability grouping, and a
  materialised rendered copy — reserved or deferred until a consumer forces each.

Boundary: a specification owns the durable *what* and *why*; the technical spec
owns the durable *how*; a slice owns a single change. A requirement is owned by no
spec — it is a peer entity that specs *member*. A fact lives in exactly one tier:
structured obligations as entities, narrative as prose, and the two never restate
one another.

## 3. Principles

- **Requirements are peer entities, never embedded prose.** A functional or
  quality requirement is an addressable thing with durable identity; listing it as
  a prose row duplicates queryable data into narrative and manufactures drift.
- **Identity is durable; the label is mobile.** A requirement's identity is its
  permanent numeric id, owned by no spec. Its `FR-`/`NF-` label is a per-spec
  display convenience that may move or renumber without the identity dangling.
- **A missing canonical requirement is a stop condition, not licence to infer.**
  The spec is the upstream source of product intent; an agent that finds it absent
  records the gap, it does not fabricate the contract.
- **Integrity failures are surfaced, not silently repaired.** A dangling
  reference, a duplicate label, or an orphaned requirement is reported as a hard
  finding; the corpus never carries a broken reference quietly.
- **The readable whole is derived, never stored stale.** A spec's one-document
  view is reassembled on demand from present state, so it cannot drift from the
  fragments it composes.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- A requirement must be reserved through the shared identity-reservation primitive,
  so concurrent agents in one working tree never hand-assign a colliding id.
- Requirement membership and its display label must live on the spec-side edge, not
  on the requirement, so the same requirement can be membered by more than one spec.
- The corpus-wide orphan check is only meaningful across the whole corpus; a
  single-spec check resolves outbound references and label uniqueness only.
- Structured, queryable requirement data lives in TOML; narrative lives in markdown;
  neither tier restates the other.

Invariants:

- A requirement's identity is permanent and owned by no spec; every reference to it
  stores that durable identity, never a compound or spec-scoped key.
- A display label is unique within a single spec's membership.
- Every requirement is membered by at least one spec; an unmembered requirement is a
  torn write, not a valid state.
- The readable whole is a pure function of present local state — it cannot be stale.

## 5. Success Measures

- An agent scoping a change can read a capability's product intent, scope boundary,
  principles, and requirements from the spec alone, without recourse to chat history
  or a single change's design.
- A requirement can be re-labelled, re-ordered, or membered by a second spec without
  any reference to it dangling and without altering its identity.
- A reviewer asking "what does this spec require, and what verifies it" reads one
  reassembled document, not a scatter of fragments parsed by hand.
- A dangling reference, duplicate label, or orphaned requirement is caught by an
  integrity pass before it is trusted, with a non-zero result an automated gate can
  read.
- No queryable requirement datum appears in prose; the structured copy is the single
  source of truth and the two never disagree.

## 6. Behaviour

Primary flow — declare a spec: an operator names a capability under a subtype; the
system reserves the next free identity in that subtype's namespace, scaffolds the
spec's durable home with its identity record and a prose body seeded for the eight
canonical sections, and reports where it lives. The spec opens in the draft stage.

Primary flow — record a requirement: an operator adds a functional or quality
requirement to a spec; the system reserves the requirement as a peer entity with
its own durable identity and, in the same operation, appends a membership row that
binds it to the spec under the next free label for its kind. The requirement opens
pending.

Primary flow — read the whole: an operator asks to see a spec; the system
reassembles its identity, its prose body verbatim, and each membered requirement
(in membership order, under its label and durable id, with the requirement's
statement and acceptance criteria) into one document. The view is recomputed each
time and never materialised stale.

Survey flow: an operator asks for the specs in a project, per subtype, and receives
each with its standing, alias, and member count; the survey can be narrowed by
standing.

Integrity guard: an operator validates the corpus, or one spec. The whole-corpus
pass reports every dangling reference, every duplicate label within a spec, and
every orphaned requirement as a hard finding with a non-zero result; a single-spec
pass resolves that spec's outbound references and label uniqueness only, since
membership is unknowable from one spec.

Edge cases and failure modes: an empty project yields the first identity; reserving
a requirement and then failing to append its membership row leaves the requirement
orphaned and uncommitted, surfaced by the integrity guard rather than silently
carried; two branches that each add a label of the same kind to one spec merge into
a duplicate the guard flags.

## 7. Verification

Verification confirms that a specification durably carries its product intent, that
requirements hold stable identity while membership and labels stay mobile, that the
readable whole cannot go stale, and that corpus integrity is enforced — without
binding the spec to a particular implementation.

Requirement identity and membership are proven by exercising reservation and the
two-step add directly: a reserved requirement gains a durable identity and a
membership row in one operation, the same requirement can be membered under more
than one spec, and a label change or re-order never disturbs the identity. The
readable whole is proven by confirming the reassembled view composes identity,
prose, and membered requirements purely from present state, so repeating the read
yields the same document with no stored copy to age. Corpus integrity is proven by
seeding the registry with dangling references, duplicate labels, and an orphaned
requirement and confirming each is reported as a hard finding with a non-zero
result, while a single-spec check is confirmed to resolve only outbound references
and label uniqueness. The storage-tier separation is proven by confirming no
queryable requirement datum appears in prose and that the structured entity is the
sole source of truth.

Where a check must reference a specific obligation, it cites the durable
requirement entity (REQ-NNN), never a mobile membership label. Coverage of the
functional and quality requirements is tracked against those entities, not
duplicated here.

## 8. Open Questions

- Should a capability be a first-class entity that groups requirements, or remain an
  informal grouping expressed by tags on requirements? This blocks whether coverage
  and traceability attach to a capability or only to individual requirements.
- Once a change process exists, where is a requirement's lifecycle standing
  authoritative — on the requirement itself, or derived from the changes that
  complete it? This blocks any gate that reads requirement standing.
- When the same requirement is to be shared by a second spec, what relabelling
  policy governs the new membership? This blocks the cross-spec reuse verb and any
  convention that depends on label stability across specs.
