# PRD-009: Backlog

## 1. Intent

Work surfaces faster than it can be scoped. A bug noticed mid-task, an improvement
worth remembering, a chore that keeps being deferred, a risk that must not be lost, a
half-formed idea — all of it needs somewhere durable to land *before* anyone decides
to act on it. Without a first-class capture layer, this intent is stranded in chat
history, scattered `TODO`s, and ad-hoc notes (`backlog.local.md`); it is invisible to
tooling, impossible to triage or prioritise as a set, and routinely lost.

The **backlog** is doctrine's capture layer — the first step of the spec-driver loop
(`capture → scope → design → implement → audit → close`). It makes work intent a
durable, queryable, governed artefact: every item is captured the moment it surfaces,
carried through a triage-and-resolution lifecycle, ordered by priority, and — when it
earns the work — promoted into a scoped slice. Its value is that intake stops leaking:
nothing worth doing is held only in conversation, the whole body of pending work is
reviewable and prioritisable at a glance, and the bridge from "noticed" to "scoped" is
explicit rather than improvised.

## 2. Scope

In scope:

- Capturing a unit of work intent as a durable first-class item across the kinds the
  glossary reserves — issue, improvement, chore, risk, idea — as **one** entity
  discriminated by an `item_kind` facet.
- Holding kind-specific facets, notably the risk facet (likelihood, impact, controls,
  origin, acceptance), as typed storage.
- Carrying an item through its work-intake status lifecycle.
- Surveying and filtering the backlog as a set.
- Ordering the backlog by priority.
- Bridging a captured item into a scoped slice — the capture→scope hand-off.
- A forward relation seam linking items to slices, specs, and drift records.

Out of scope:

- Non-work / epistemic records — assumptions, decisions, questions, findings,
  tradeoffs, constraint statements. They fail the work-intake membership test
  (§3) and belong to the decision/governance family (where ADR already lives) or a
  future epistemic group (OQ-005), never to the backlog.
- The `problem` kind — excluded until it earns a reserved id and a decomposition
  boundary distinguishing it from `issue` (a broad problem decomposes into issues /
  improvements / ideas; an issue is one concrete broken thing).
- The technical design and storage mechanism of items — engine, fileset descriptors,
  reservation primitive (those belong to `/spec-tech` and per-slice `/design`).
- Runtime execution progress of any work an item spawns — that lives on the slice and
  its phase state, not the backlog item.
- Migrating an external backlog corpus into doctrine (importer).
- A terminal/TUI artefact browser.
- An auto-generated `backlog.md` summary index (a derived view).
- Lifecycle gating or approval workflow — status is hand-settable, ungated, as
  slices, ADRs, and specs ship today.

Boundary: the backlog owns *captured intent and its standing* — what work exists, of
what kind, at what priority, in what state. It does not own the change contract (that
is a slice) nor the work's execution. Once an item is promoted, the slice is
authoritative for the change; the backlog item remains the record of where it came
from.

## 3. Principles

- **One entity, one schema.** Every kind of work item is the *same* `backlog_item`
  entity discriminated by an `item_kind` facet — never parallel per-kind schemas or
  directories. Kind variation is facet fields on one entity, not a fork.
- **Backlog holds latent work, not epistemic state.** This is normative, not
  advisory: a backlog item *is* a latent unit of work intent that can be triaged,
  prioritised, and promoted into a slice. **If a candidate does not fit the
  work-intake lifecycle, it is not a backlog item.** This excludes assumptions,
  decisions, questions, findings, tradeoffs, and constraint statements — their
  standings are epistemic or governance lifecycles (an assumption is held→validated, a
  decision proposed→accepted→superseded), not work to be done. They live in their own
  families (a risk facet, the separate decision/governance family, or a future
  epistemic group — OQ-005), never as an `item_kind`.
- **Risk belongs because it is unresolved work-risk.** A risk is admitted not as a
  general epistemic record but because it is uncertain future harm that may require
  mitigation, acceptance, expiry, or promotion into scoped work — work-intake
  adjacent. Strip that and it would not qualify.
- **Each kind has a discriminating boundary.** Adjacent kinds are separated by an
  explicit test, not left to judgement, so an item resolves to exactly one kind; where
  more than one fits, a fixed precedence decides.
- **Capture is cheap; losing intent is not.** The bar to record an item is low, so
  intent lands the moment it surfaces rather than being deferred into oblivion.
- **Canon fixes the vocabulary.** The kind set and id schemes (glossary) and the
  status lifecycle (`entity-model`) are closed, deliberately-recorded sets — not
  per-item invention, and not silently re-mapped from any source corpus.
- **Approval is not lifecycle.** Accepting or expiring a risk is a facet field, not a
  status state; the status vocabulary is uniform across all kinds.
- **Typed storage, never a bag.** Every facet is enumerated, typed storage; there is
  no untyped frontmatter catch-all holding product data.
- **The structure anticipates the bridge without prebuilding it.** Prioritisation,
  the slice hand-off, and the relation seam are first-class product intent; each
  attaches to a stable item without reshaping it.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below. This
section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- Every `item_kind` is carried by the single `backlog_item` entity discriminated by an
  `item_kind` facet; no implementation may introduce parallel per-kind schemas or
  directories.
- The kind set is exactly the five glossary-reserved kinds — issue (`ISS`),
  improvement (`IMP`), chore (`CHR`), risk (`RSK`), idea (`IDE`), three-character
  prefixes — and may not be extended without a reserved id. `problem` is excluded
  until it earns one.
- Every `item_kind` must pass the work-intake membership test (promotable to a slice,
  standing fits the work-status lifecycle) and carry a discriminating boundary test
  that resolves an item to exactly one kind; a candidate that fails either is not an
  `item_kind`.
- When an item satisfies more than one kind's boundary, a fixed precedence resolves it
  deterministically: `risk > issue > improvement > chore > idea`.
- Why an item left the active flow is recorded as a `resolution` field orthogonal to
  `status`: `status` answers whether the item is active, `resolution` answers why it
  stopped (e.g. fixed, done, mitigated, accepted, duplicate, wont_do, obsolete,
  promoted). No close-reason is ever encoded as a `status` state.
- The status lifecycle is the closed canon set `open | triaged | started | resolved |
  closed`; no kind may add or rename a status state. `risk` is the lifecycle outlier
  — a risk is mitigated/accepted/expired rather than "resolved"; that standing is
  absorbed by the risk acceptance facet, not by extending the status vocabulary.
- Risk acceptance is a facet field (`accepted` / `expired` / none), never a lifecycle
  state.
- The capability must reuse the shared entity/scaffold substrate; extending it must not
  regress the existing entity callers (slice, ADR, spec, memory).
- An item's status is hand-settable and ungated, consistent with how slices, ADRs, and
  specs ship today.

Invariants:

- Every backlog item is a latent unit of work intent — promotable into a slice and on
  the work-status lifecycle; nothing that fails the membership test is ever a backlog
  item.
- An item's identity — its kind prefix plus number — is permanent; the slug is never
  authoritative and tooling resolves an item only by its id.
- An item's `item_kind` is fixed at capture; an item never silently changes kind.
- Every facet is typed, enumerated storage; untyped product data never persists.
- A resolved or closed item remains durably stored and addressable — "hidden by
  default" is a view, never deletion.
- An item's relation seam is always present, even when empty, so the bridge and linkage
  machinery have a stable attachment point.

## 5. Success Measures

- Work intent stops leaking: a contributor can record an issue, improvement, chore,
  risk, or idea the moment it surfaces, and it survives outside the conversation that
  produced it.
- The whole body of pending work is reviewable as a set — an operator can survey it,
  narrow by kind, status, and tag, and see at a glance what is open versus settled.
- The backlog can be ordered, so "what to pick up next" is answerable from the artefact
  rather than from memory.
- The capture→scope hand-off is explicit: an item can be promoted into a slice, and the
  resulting slice records where it came from.
- Risk is first-class: a risk item carries its likelihood, impact, controls, origin,
  and acceptance standing without contorting the shared model.
- A reader (human or agent) can trust that every field is typed and that an item's kind
  and identity are stable — no untyped bags, no kind drift, no id churn.

Acceptance gates:

- Capturing each `item_kind` yields a durable item with a reserved kind-correct id and
  kind-correct default facets.
- A survey filters by kind, status, and tag, and hides resolved/closed by default while
  keeping them addressable.
- A status transition is atomic and edit-preserving — it round-trips without dropping
  comments or unknown keys.
- Extending the capability leaves the existing slice/ADR/spec/memory suites green
  unchanged.

## 6. Behaviour

Primary flow — capture: a contributor names a unit of work and its kind; the system
reserves the next free id in that kind's namespace, materialises the item's durable
home seeded with the kind's default facets, sets status `open`, and reports where it
lives.

Primary flow — survey: an operator asks for the backlog and receives items they can
narrow by kind, status, tag, and title; resolved and closed items are hidden by default
and revealed on request. Ordering reflects recorded priority where set.

Primary flow — inspect: an operator names an item; the system detects its kind from the
id prefix and renders the item's identity, its kind-specific facets, its timestamps,
and its outbound relations.

Primary flow — transition: an operator moves an item to another state in the canon
lifecycle; the change is atomic and preserves the rest of the item verbatim. When an
item leaves the active flow (resolved or closed), a `resolution` records *why* —
distinct from the `status` that records *whether* it is active.

Kind boundaries — an item resolves to exactly one kind: an **issue** is expected
behaviour gone wrong; an **improvement** works but should be better; a **chore** is
maintenance or housekeeping work; a **risk** is uncertain future harm that may need
mitigation, acceptance, expiry, or promotion into scoped work; an **idea** is a
speculative possibility not yet shaped as work. When more than one boundary fits,
precedence decides: `risk > issue > improvement > chore > idea`.

Primary flow — prioritise: an operator establishes a relative ordering across backlog
items so the set carries a defensible "what next"; the ordering is recorded against
durable item identities.

Primary flow — promote: an operator promotes a captured item into a scoped slice; the
new slice records the originating item, and the item reflects that it has been carried
into scope, its `resolution` recording `promoted`.

Risk flow: a risk item carries likelihood, impact, controls, and origin as facet
fields, and its acceptance standing (`accepted` / `expired` / none) as a facet field —
distinct from its lifecycle status. A risk leaves the active flow by mitigation,
acceptance, or expiry rather than being "resolved" like a defect; that close-reason is
the pressure point reconciled in OQ-006.

Edge cases and guards: an empty backlog yields the first id in each kind's namespace; a
resolved or closed item stays addressable and re-openable; promoting an already-promoted
item is recognised rather than silently duplicating scope; a hand-edited slug may go
stale while the canonical id remains authoritative.

## 7. Verification

Verification confirms that captured intent is durable and queryable, that the canon
vocabulary holds, that kind-specific facets (notably risk) are carried without forking
the model, and that the lifecycle, prioritisation, and bridge behaviours hold — without
binding the spec to a particular implementation.

Capture is proven by confirming each `item_kind` produces a durable item with a
reserved kind-correct id, kind-correct default facets, and an initial `open` status,
persisting across reads. Survey is proven by confirming items render filterable by kind,
status, and tag, with resolved/closed hidden by default yet still addressable under an
explicit reveal, and ordered by recorded priority. Inspection is proven by confirming
kind is resolved from the id prefix and the item's identity, facets, timestamps, and
outbound relations render. The lifecycle is proven by confirming a status transition is
atomic and edit-preserving — it round-trips without dropping comments or unknown keys —
that only canon states are reachable, and that a closed item carries a `resolution`
distinct from its `status`. The kind boundaries are proven by confirming an item
satisfying more than one boundary resolves deterministically under the precedence
`risk > issue > improvement > chore > idea`. The single-entity discipline is proven by
confirming every kind, including risk with its extra facets, is carried by one entity
discriminated by `item_kind`, with no parallel schema. The bridge is proven by
confirming a promoted item yields a slice that records its origin. The behaviour-
preservation obligation on the shared substrate is proven by the existing entity suites
staying green unchanged.

Where a check must reference a specific obligation, cite the durable requirement entity
(`REQ-NNN`), never a mobile membership label. Coverage of the functional and quality
requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

(OQ-001 — the `item_kind` set — is resolved: exactly the five glossary-reserved kinds;
`problem` excluded pending a reserved id and a decomposition boundary; `assumption` /
`decision` and other epistemic records excluded by the membership test. Recorded in §2,
§3, and §4 rather than left open.)

- OQ-005 — A future home for non-work / epistemic items. ADRs capture high-impact,
  architecturally-significant decisions; lower-stakes decisions, assumptions, and open
  questions have no home today and might warrant a risk-analogous treatment (typed
  facets, their own lifecycle). They are deliberately excluded from the backlog by the
  work-intake membership test. Does a distinct epistemic/governance entity group earn
  its own entity later — `assumption` (held → testing → validated / invalidated /
  obsolete), `decision` (proposed → accepted → superseded / rejected), `question`
  (open → answered → obsolete)? Out of scope for the backlog; recorded so the exclusion
  is a decision, not an omission.
- OQ-006 — `resolution` vs the risk `acceptance` facet. Both can carry "accepted" /
  "expired" for a risk, which risks two homes for one fact (one-fact-one-artefact).
  Reconcile: is `resolution` the single generic close-reason and the risk facet holds
  only likelihood/impact/controls/origin, or does the risk facet own accepted/expired
  (with dates/rationale) and `resolution` derive from it for risks? Blocks the
  `resolution` domain for the risk kind.
- OQ-002 — What is the product shape of priority: a single global total order, a
  per-kind ordering, or a head-tail partition (an explicitly-ranked head over an
  unranked tail)? Blocks the prioritise behaviour and how survey renders order.
- OQ-003 — On promotion, is the backlog item consumed (moved to a terminal state) or
  kept live and linked to its slice? Blocks the bridge's exit semantics and whether a
  promoted item still appears in the default survey.
- OQ-004 — Are backlog↔artefact relations reciprocal (an item shows inbound references
  from slices/specs/drift), or outbound-only in the durable item with reverse lookups
  deferred to a registry surface? Blocks the inspect view's completeness claims.
