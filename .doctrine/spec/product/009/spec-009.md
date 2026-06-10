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
- Holding kind-specific descriptive facets, notably the risk facet (likelihood,
  impact, origin, controls), as typed storage.
- Carrying an item through its work-intake status lifecycle.
- Surveying and filtering the backlog as a set.
- Ordering the backlog by priority.
- Bridging a captured item into a scoped slice — the capture→scope hand-off.
- A forward relation seam linking items to slices, specs, and drift records.
- Optional priority-engine capture enrichment on an item: hard `needs` / soft `after`
  item→item ordering edges, and a `triggers` list of architectural prefactor riders.
  Optional, never required (§4); consumed by the derived priority engine (PRD-011,
  SPEC-001), minted here.

Out of scope:

- Non-work / epistemic records — assumptions, decisions, questions, findings,
  tradeoffs, constraint statements. They fail the work-intake membership test
  (§3) and belong to the decision/governance family (where ADR already lives) or the
  epistemic/governance records family (PRD-010, the OQ-005 resolution), never to the
  backlog.
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
  families (a risk facet, the separate decision/governance family, or the
  epistemic/governance records family — PRD-010, the OQ-005 resolution), never as an
  `item_kind`.
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
- **Approval is not lifecycle.** Accepting or expiring a risk is a `resolution` (the
  reason it left active attention), not a `status` state and not a kind facet; the
  status vocabulary is uniform across all kinds.
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
  — a risk is mitigated/accepted/expired rather than "resolved"; that exit reason is
  recorded via `resolution`, never by extending the status vocabulary.
- A kind facet holds only descriptive facts about an item's *shape*; no close-reason
  (accepted, expired, mitigated, …) is ever stored in a kind facet — close-reasons are
  `resolution` values. The risk facet therefore carries likelihood, impact, origin,
  and controls only; its acceptance/expiry is the item-level `resolution`.
- The capability must reuse the shared entity/scaffold substrate; extending it must not
  regress the existing entity callers (slice, ADR, spec, memory).
- An item's status is hand-settable and ungated, consistent with how slices, ADRs, and
  specs ship today.
- The priority-engine enrichment — `needs`/`after` edges and `triggers` — is optional:
  no capture path may require it, and an item carrying none is fully captured, surveyed,
  and (absent triggers) actionability-eligible. The authored field names are
  capture-surface ergonomics decoupled from the engine's internal overlay vocabulary;
  classifying `needs`/`after` into the engine's edge species, the glob matcher, the
  file-set sources, and the actionability mask are all the engine's (SPEC-001), not this
  spec's. `after`'s `rank` is a pairwise-edge attribute, never the item-level
  authored-priority scalar (PRD-011 OQ-001).

Invariants:

- Every backlog item is a latent unit of work intent — promotable into a slice and on
  the work-status lifecycle; nothing that fails the membership test is ever a backlog
  item.
- `status` records lifecycle position, `resolution` records why an item left active
  attention, and kind facets record kind-specific descriptive facts only. The three
  never overlap — no close-reason is ever stored in a kind facet.
- An item's identity — its kind prefix plus number — is permanent; the slug is never
  authoritative and tooling resolves an item only by its id.
- An item's `item_kind` is fixed at capture; an item never silently changes kind.
- Every facet is typed, enumerated storage; untyped product data never persists.
- A resolved or closed item remains durably stored and addressable — "hidden by
  default" is a view, never deletion.
- An item's relation seam is always present, even when empty, so the bridge and linkage
  machinery have a stable attachment point.
- The priority-engine enrichment is never authored truth the engine writes back: the
  item authors `needs`/`after`/`triggers`; everything the engine derives from them
  (actionability, blocking, ordering, the trigger mask) is computed and disposable,
  never persisted onto the item.

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
- Risk is first-class: a risk item carries its likelihood, impact, origin, and controls
  as descriptive facets, and its accepted/expired exit as a `resolution`, without
  contorting the shared model.
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

Risk flow: a risk item carries likelihood, impact, origin, and controls as descriptive
facet fields — the risk's shape, not its closure. A risk leaves the active flow by
mitigation, acceptance, or expiry rather than being "resolved" like a defect; that exit
reason is the item-level `resolution` (e.g. `mitigated` / `accepted` / `expired`), not a
field on the risk facet.

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

(OQ-005 — a future home for non-work / epistemic items — is resolved: it earned its
own entity family, specified in PRD-010 (Epistemic and Governance Records). ADRs keep
high-impact, architecturally-significant decisions; lower-stakes decisions,
assumptions, questions, and durable constraints become one `knowledge_record` entity
discriminated by `record_kind` — `assumption` (held → testing → validated / invalidated
/ obsolete), `decision` (proposed → accepted / rejected / superseded), `question`
(open → answered / obsolete), `constraint` (active → waived / superseded / retired) —
each with typed facets and its own lifecycle. They remain deliberately excluded from
the backlog by the work-intake membership test; the backlog links to them, never
admits them as an `item_kind`. The exclusion was a decision, not an omission, and
PRD-010 is where these records now live.)
(OQ-006 — `resolution` vs the risk `acceptance` facet — is resolved: `resolution` is
the single generic close-reason for every kind; the risk facet holds only descriptive
shape (likelihood/impact/origin/controls) and owns no accepted/expired field. The
invariant "no close-reason is ever stored in a kind facet" (§4) records this; the risk
kind stays first-class without a lifecycle-special field.)
(OQ-002 — the product shape of priority — is resolved by PRD-011 as a two-layer
head-tail model. The **authored** layer is minimal and durable: a ranked head over an
unranked tail, the operator-authored priority seam this spec's FR-006 (`REQ-054`)
owns. The **derived** layer is registry-backed and disposable: PRD-011 computes
actionability, blocking, consequence, and explanation from the typed relation graph to
order survey and next-work views without persisting any of it as backlog truth. This
rejects a mandatory global total order (capture must stay cheap) and per-kind ordering
as the sole model (a single "what next?" must compare risks, issues, chores,
improvements, and ideas in one view). The exact authored field — `rank` / `band` /
`pin` / ordering file — stays open as PRD-011 OQ-001.)
(OQ-003 — promotion exit semantics — is resolved: promotion **consumes** the item. It
moves to a terminal status carrying `resolution = promoted` (the close-reason already
defined in §4/§6), so it leaves the active flow; the default survey hides it (PRD-011
REQ-075 excludes promoted items from default active output) while it stays addressable
under an explicit reveal. The link is the slice→item origin edge, authored on the slice
(ADR-004 §1); the item's "what it became" is derived, never stored on the item.

The item is **not independently reopened** as a backlog operation — promotion is a
one-way bridge, and recurred, regressed, or follow-on work is captured as a *new* item,
since the slice now owns that work's lifecycle. The single exception is a *mistaken*
promotion (wrong item, premature, slice never started): it is corrected **slice-side**,
by abandoning the slice. Because the slice is the authoring side of the origin edge
(ADR-004 §1), tearing the slice down removes the edge and releases the item back to the
active flow — so there is no dangling slice→item edge and the backlog defines no special
un-promote verb. This leans on slice lifecycle transitions (still nascent — ADR-003
close/reconcile is deferred); in v1's hand-settable, ungated posture an operator clears
the resolution and abandons the slice by hand, and this OQ fixes only which side owns the
correction.)
(OQ-004 — backlog↔artefact reciprocity — is resolved by ADR-004: relations are stored
**outbound-only** on the durable item; reciprocity is real but **derived** — inbound
references (which slices/specs/drift point at an item) are computed by the registry
scan, never authored on the item. Exactly one side authors each relation type. The
inspect view's inbound-completeness claim is the registry-backed surface's to make, not
the sync-free reader's; reverse lookup is correct-but-uncached until the feature DAG
lands.)

(OQ-007 — the **authored** priority-engine capture schema (raised by PRD-011 /
SPEC-001 D4·D6) — is resolved: the backlog item seam admits it, as **optional**
enrichment minted here and consumed by SPEC-001. Recorded in §2, §4, and the new
FR-010 (`REQ-097`) / FR-011 (`REQ-098`). The shape was settled on agent-UX grounds —
token cost, traversability, self-describing authored names:

- **Edges (FR-010 / `REQ-097`).** Two optional item→item edges on the outbound seam:
  a hard `needs` (payload-free id list — blocked until those land) and a soft `after`
  (inline-table list `{ to, rank }` — prefer this item after those). Both live in the
  item's one relation block (one read, greppable); `after` is an inline-table list, not
  a sister file or block-tables, so it stays one line per edge and the array order is
  the adapter's stable `age` source (SPEC-001 D5). **The authored names deliberately do
  not leak the engine's `dep`/`seq` overlay vocabulary** — that classification is
  policy/adapter's (SPEC-001 D4). Optional throughout (§4): an edgeless item still
  surveys by derived consequence + fallback, so PRD-011 §4 "capture must never require
  dependency modelling" holds.
- **Triggers (FR-011 / `REQ-098`).** An optional `triggers` **list** of `{ globs, note }`
  riders. A **list**, not one field, so an item can rider several independent code
  surfaces each with its own note; the engine masks the item non-actionable until a
  file set matches ≥1 entry's globs and surfaces that entry's note (SPEC-001 D6). The
  matcher, file-set sources, and mask are SPEC-001's; this spec mints only the field.
  Promotes IMP-013/014's coarse `trigger` tag to typed structure.
- **rank scope.** `after`'s `rank` is a **pairwise-edge** attribute (preference
  strength), categorically distinct from the still-open **item-level** authored-priority
  scalar (PRD-011 OQ-001, FR-006/`REQ-054`). OQ-007 mints only the edge attribute;
  OQ-001 stays open and, when it lands, must not reuse a bare `rank` for the item scalar
  — the two are different scopes (edge vs node), not one field.

Unblocks SPEC-001 FR-005 (`REQ-096`) and the D6 trigger mask (`REQ-093`), which consume
this schema. SPEC-001's D4/D6/REQ-093 wording, written against a singular `trigger`
field, should be reconciled to the `triggers` list when SPEC-001 next moves.)
