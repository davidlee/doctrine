# PRD-011: Graph-Derived Priority and Actionability

## 1. Intent

Doctrine governs work across many kinds, not one. Backlog capture is deliberately cheap
(PRD-009): an issue, improvement, chore, risk, or idea lands the moment it surfaces, then
is triaged, resolved, or promoted into a slice. But captured backlog items are only part
of the live work — a slice can be awaiting design, a requirement can be pending, a spec
can be in draft, an open question can gate a decision. Across all of it sits one hard
question — *what should I look at next, and why?* — that a flat, per-kind priority field
cannot answer for a governed corpus. Some work is urgent because an operator ranked it;
some because it blocks a slice, satisfies a requirement, mitigates an active risk,
answers an open question, or clears drift; some is not actionable yet because the item's
own lifecycle is not in a workable state, or because a predecessor is unresolved or a
governance question is still open. A hand-set number per item collapses all of that into
one value that rots the moment the graph around it moves.

This capability adds a small, native, **graph-derived** priority and actionability layer
over doctrine's existing entity graph, spanning **every admitted kind** rather than
backlog alone. It keeps authored *priority* intentionally small (PRD-009 owns the backlog
priority rank/band seam) and owns the **cross-kind dependency/sequence capture surface** —
the hard `needs` and soft `after` edges a work-like entity authors as deliberate operator
intent. It derives richer ordering and actionability from those authored edges and the
lifecycle state already in the corpus, and produces **inspectable explanations** without
ever storing *derived judgement* as authored truth. **Actionability is a synthesis** — an
item is actionable when its own lifecycle status is in a workable, non-terminal state
*and* its admitted graph relations leave it unblocked; neither half alone decides it. Its
value is that prioritisation becomes *defensible without becoming heavy*: capture and
authoring stay cheap, yet survey and "next work" views account for blockers,
dependencies, lifecycle state, explicit ordering, and governance pressure across kinds,
and a reviewer can always see *why* an item ranked where it did. Doctrine gains a native
dependency-resolution spine that a richer evaluation engine can later back onto, while
the GPL core stays free of product-specific scheduling, time-pressure, or commitment
semantics.

## 2. Scope

In scope:

- A derived priority and actionability **view** over doctrine's registry graph —
  recomputed from authored state, never authored back into it.
- The **cross-kind dependency/sequence capture surface** — a work-like entity (a slice or
  any of the five backlog kinds) authors its own hard `needs` (prerequisite, payload-free)
  and soft `after` (sequence, per-edge `rank`) edges onto another work-like entity, as the
  authored input the derived view consumes. The backlog item→item instance of this surface
  is PRD-009's (REQ-097); this capability owns its **generalisation to work-like kinds**.
  Targets are restricted to work-like kinds — governance, spec, requirement, and knowledge
  kinds are refused at author time, so this surface never silently becomes a
  governance-gates-work mechanism.
- A typed graph abstraction over doctrine entities and relations, populated from backlog
  items, slices, specs, requirements, ADRs, knowledge records, and drift records.
- The generic dependency-resolution behaviours doctrine needs: reverse-edge indexes,
  reachability, blockers and blocked-by, cycle diagnostics for relation kinds that must
  be acyclic, eligibility masks, stable deterministic ordering, and machine-readable
  explanation paths.
- A doctrine policy mapping existing entity kind, lifecycle, resolution, and relation
  types into derived channels such as actionable, blocked, blocking, priority hint, and
  reason — including a **per-kind classification of which lifecycle statuses are workable
  versus terminal**, so actionability applies uniformly across kinds.
- Registry-backed survey / next-work / inspect / explain / blockers surfaces that rank
  actionable work **across all admitted kinds** in one comparable view and show derived
  inbound references, blocking state, and priority explanations.

Out of scope:

- The **authored priority rank/band scalar** itself — its on-disk shape and the verb that
  records operator rank/band are PRD-009's (FR-006, "order the backlog by priority"). This
  capability *reads* that scalar and contextualises it; it does not redefine or duplicate
  it. (This disclaim is the priority rank/band scalar only; the cross-kind
  dependency/sequence *capture* surface above is in scope.)
- A full scheduling or task-management engine.
- Time-pressure semantics — deadlines, best-before, scheduled-for, lateness cost,
  remaining-work estimates, commitment pressure — and sequential/parallel project,
  habit, recurrence, calendar, or capacity modelling.
- A scalar universal "urgency score" with product-specific weighting.
- Persisting derived priority, actionability, blockers, reverse references, or
  explanations as authored truth.
- Replacing explicit human priority, or replacing PRD-009's backlog lifecycle or
  PRD-010's epistemic/governance lifecycles.
- Embedding a richer external policy engine. This capability preserves a seam for one;
  it does not import it.

Boundary: this capability owns **derived graph interpretation for cross-kind
priority-and-actionability surfaces** — the read-side context computed over the existing
corpus, plus the cross-kind dependency/sequence edge-capture surface that feeds it. It
does not own work capture (the creation of the work items themselves), item kind
membership, any kind's lifecycle, slice execution, scheduling, or product-specific urgency
philosophy. Each authored entity
remains the source of its own intent and lifecycle; the technical shape of the graph
crate, its module boundary, and the v1 policy's channel, per-kind status-class, and
relation-kind layout are downstream `/spec-tech` concerns, not product intent.

## 3. Principles

- **Authored priority is small; authored dep/seq is operator intent.** Doctrine stores only
  deliberate operator intent — an optional rank, band, or pin (PRD-009's seam), and the hard
  `needs` / soft `after` dependency-sequence edges a work-like entity authors. It never
  stores computed urgency, pressure, blocker state, or "next" judgement as authored truth:
  authored dep/seq edges are operator-stated *input*, never the engine's *derived judgement*
  written back.
- **Derived priority is a view.** Derived actionability, blockers, inbound references,
  and graph priority are recomputed from the registry graph. They may be cached, but a
  cache is disposable and never authoritative — correctness is recomputation from
  authored state. No derived judgement is stored as authored truth.
- **The graph is generic; the policy is doctrine.** The reusable core understands nodes,
  typed directed edges, acyclicity, traversal, propagation, and explanation paths.
  Doctrine policy — not the core — interprets what a backlog item, slice, requirement,
  open question, risk, or drift record *means*.
- **No commercial scheduling semantics in the GPL core.** The core may traverse and
  reduce a graph; it must not know deadlines, best-before dates, lateness cost,
  commitment pressure, sequential-project philosophy, habit resurfacing, or any external
  prioritisation heuristic. A rule that cannot be stated without product nouns belongs in
  policy, not the core.
- **Actionability and priority are separate.** A blocked item may be important; an
  actionable item may be low priority. The view exposes both rather than collapsing them
  into one score.
- **Actionability synthesises status and relations.** An item is actionable only when its
  own lifecycle status is workable *and* its admitted relations leave it unblocked. A
  terminal status alone makes it non-actionable; an unresolved admitted blocker alone
  makes it non-actionable. **No kind is barred as a kind** — an accepted ADR is
  non-actionable because its status is terminal, not because ADRs are excluded; a proposed
  one may be actionable. The per-kind mapping of statuses to workable-versus-terminal is
  doctrine policy, not graph-core truth.
- **Explanations are structured, not prose magic.** A derived answer carries
  machine-readable reasons — local state, relation paths, blockers, masks, ordering
  contributors. User-facing prose is *rendered* from those reasons, never the source of
  truth.
- **Cycle handling is typed.** Some relation overlays must be acyclic; others may tolerate
  cycles. The graph layer reports cycle diagnostics under a per-relation-kind contract
  rather than pretending every edge carries the same invariant.
- **The richer-engine seam is intentional.** The graph core and doctrine policy are
  shaped so a future richer engine can consume the same graph view and emit additional
  channels without changing authored storage.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below. This
section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- The authored backlog model admits at most a small explicit priority seam (rank, band,
  or equivalent operator-authored ordering metadata, owned by PRD-009). Capture must never
  require estimates, due dates, schedules, or dependency modelling.
- The derived surface must be computed from the registry graph and entity lifecycle
  state; it must not introduce new durable reverse-link fields on authored entities.
- The graph core must operate over opaque node IDs and typed directed edges. It must not
  depend on doctrine entity structs, TOML layout, or file paths, and must contain no
  time, scheduling, commitment, urgency, or external-product vocabulary.
- Doctrine must supply an adapter from its registry scan to the graph core; policy
  interpretation (statuses, resolutions, relation kinds, item kinds, terminal states)
  must live outside the graph core.
- Derived outputs are either ephemeral or stored only as disposable cache/projection
  data; they are never authored into entity TOML as canonical truth.
- A default listing of any kind must remain useful without the derived engine — the
  derived surface enhances survey/next/explain/blockers behaviour; it is not required for
  capture or authoring.

Invariants:

- Each entity's lifecycle remains authoritative on the entity. Derived actionability
  never mutates `status`, `resolution`, `kind`, priority metadata, or relations.
- The same registry graph, policy version, and query options always produce the same
  derived ordering and explanations.
- Reverse references shown by the derived surface are always computed from authored
  outbound edges — never from a separately authored inbound field.
- A relation kind declared acyclic reports cycles as diagnostics rather than silently
  producing a misleading order.
- A terminal item of any admitted kind is hidden from default active derived output
  unless explicitly included; the per-kind terminal mapping is policy. A promoted backlog
  item is the backlog-specific instance — not treated as active work intake, since the
  promoted slice is authoritative for the scoped work.
- A derived reason always distinguishes local authored priority, lifecycle eligibility,
  relation-derived blockers, relation-derived pressure or consequence, and deterministic
  tie-breakers.

## 5. Success Measures

- An operator asking what to consider next receives an ordered, explainable list rather
  than an opaque score.
- An actionable "next" list spans every admitted kind — a pending requirement, a proposed
  slice, and an open issue compete in one comparable view rather than separate per-kind
  lists.
- An item blocked by unresolved upstream work, an open governance question, an active
  constraint, or an incomplete dependency is marked non-actionable while remaining visible
  when it is consequential.
- An item that blocks many active or important artefacts surfaces as consequential even
  when its authored rank is absent.
- The backlog stays cheap to capture: no operator must supply estimates, due dates,
  scheduling fields, or dependency metadata to create an item.
- Inspecting an item shows both its authored outbound relations and its derived inbound
  context through the registry-backed surface.
- A reviewer can tell whether an item's ordering came from explicit rank, kind/status
  filtering, relation-derived consequence, blocker state, or a stable tie-break.
- The graph core can be tested independently of doctrine policy and of file persistence.
- A future richer engine could replace or augment the doctrine policy by consuming the
  same graph adapter and emitting additional channels.

Acceptance gates:

- A backlog item captured without priority metadata still appears in the default survey.
- A backlog item carrying explicit authored priority orders ahead of its unranked peers
  in derived survey.
- A derived priority query excludes terminal items of any kind (and promoted backlog
  items) by default and includes them only when requested.
- A derived priority query marks blocked items distinctly from actionable ones.
- A non-backlog item with a workable status and no admitted blocker appears as actionable
  in `next`; the same item with a terminal status, or gated by an unresolved admitted
  blocker, does not — proving actionability is the status-and-relations synthesis applied
  across kinds.
- A derived explanation names the relation path or lifecycle fact that caused a
  blocked / actionable / priority classification.
- A cycle in an acyclic relation overlay is reported as a diagnostic naming the involved
  node IDs and edge kinds, and the affected output degrades rather than trusting a false
  order.
- Re-running the same query over unchanged input yields the same order and explanations;
  deleting any derived cache and recomputing yields equivalent results.

## 6. Behaviour

Primary flow — derived survey: an operator asks for the corpus with derived priority. The
system scans doctrine entities into the registry graph, applies the doctrine policy, and
returns active items across kinds ordered by, in policy-defined precedence: explicit
authored priority where present; actionability state (the status-and-relations
synthesis); relation-derived consequence or pressure; kind/status policy; and a stable
deterministic fallback. The exact weights are doctrine policy, not graph-core truth.

Primary flow — next work: an operator asks what to consider next and receives a short
candidate list, each entry carrying id and title, item kind, lifecycle state, authored
priority if present, actionability state, a primary derived reason, and any blockers or
blocking consequences. The command is advisory: it transitions nothing, creates no
slice, and mutates no priority.

Primary flow — explain: an operator asks why an item is ordered or masked as it is. The
system renders the structured explanation into a readable account naming local facts and
relation paths — for example explicit rank, terminal-state exclusion, promoted-resolution
exclusion, blocked by another item, blocked by an open question, related to active drift,
blocks a slice or requirement, or deterministic fallback.

Primary flow — blockers: an operator asks what blocks an item, or what an item blocks.
The system resolves typed graph reachability over the relation kinds the policy admits
and reports direct (and, where requested, transitive) blockers, distinguishing authored
outbound edges from derived inbound edges.

Diagnostic flow — cycles: when a relation kind declared acyclic contains a cycle, the
derived surface reports a diagnostic. For affected queries the policy either excludes the
cyclic component from derived priority or degrades to local authored priority plus the
stable fallback. It never silently emits a false topological result.

Cache flow: the system may persist computed graph outputs, explanation data, warnings, or
indexes as a disposable cache. Invalidation strategy is an implementation detail; the
correctness contract is recomputation from authored entity state.

Edge cases and guards: an item with no authored priority is normal — the default for
non-backlog kinds — and orders by derived context plus deterministic fallback; a terminal
item of any kind (and a promoted backlog item) stays addressable but is excluded from
default active output; a derived answer with no admitted relation evidence still carries a
lifecycle-and-fallback explanation rather than an empty one.

## 7. Verification

Verification confirms that derived priority is deterministic, explainable, graph-backed,
and strictly separate from authored truth — without binding the spec to a particular
implementation.

Graph construction is proven by seeding a registry with entities of several kinds and
their relations and confirming the adapter emits the expected nodes and typed edges while
preserving doctrine IDs. Reverse lookup is proven by confirming the inbound references in
derived inspect output are computed from authored outbound edges, not from any authored
inbound field. Actionability is proven as a synthesis of status and relations, across more
than one kind: an item with a terminal status is non-actionable regardless of relations;
an item with a workable status but an unresolved admitted blocker is non-actionable; only
a workable-status, unblocked item is actionable. Ordering is proven by confirming explicit
priority, actionability,
derived consequence, kind/status policy, and deterministic fallback are applied in the
documented precedence for the doctrine v1 policy. Cycle handling is proven by seeding a
cycle in an acyclic overlay and confirming the diagnostic names the involved nodes and
edge kinds and that affected output degrades rather than trusting a false order.
Explanation is proven by confirming every derived classification carries structured
reasons identifying local state, relation paths, masks, and fallback contributors.
Determinism and cache disposability are proven by confirming an unchanged input yields an
identical order and explanation and that deleting any derived cache and recomputing
produces equivalent output. Boundary preservation is proven by confirming the graph-core
test suite contains no doctrine entity vocabulary and no time/scheduling/commitment
semantics, while the doctrine policy tests carry those interpretations explicitly.

Where a check must reference a specific obligation, cite the durable requirement entity
(`REQ-NNN`), never a mobile membership label. Coverage of the functional and quality
requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

(This spec resolves PRD-009 OQ-002 — the product shape of backlog priority — as a
two-layer head-tail model: a minimal durable **authored** layer (a ranked head over an
unranked tail, PRD-009's seam) plus a registry-backed **derived** layer that contextualises
it and is never persisted as truth. That resolution is recorded against PRD-009 OQ-002,
not re-litigated here.)

(This revision broadens the actionable channel from backlog-lifecycle to **all admitted
kinds**: actionability is the status-and-relations synthesis stated in §1/§3, applied
uniformly, with the per-kind workable-versus-terminal status mapping owned by policy. No
kind is barred as a kind. SL-047 — the cross-kind survey/next/explain/blockers CLI —
descends from this broadened intent.)

(OQ from the seed — new PRD vs amendment-plus-TECH — is resolved by this artefact's
existence: derived priority is its own product spec (PRD-011), and the graph crate's
mechanism, module boundary, and v1 policy/channel layout descend into a `/spec-tech`
specification downstream. The seed's §7 Policy v1, §8 Core/Policy/Adapter boundary, and
Appendix B forbidden-core enumeration are that TECH spec's seed material, not product
intent.)

- OQ-001 — What exact authored priority seam should PRD-009 expose: `rank`, `band`, `pin`,
  or a separate ordering file keyed by item id? The layer model is settled; the field
  shape is PRD-009's to fix and blocks the authored ordering verb.
- OQ-002 — Which relation kinds are admitted into the v1 actionability/blocking policy,
  and which are merely contextual? Blocks the policy's blocker set and the cycle contract.
- OQ-003 — Should the graph core be packaged as a doctrine crate, an externally-derived
  crate, or a neutral shared crate? A packaging decision the downstream TECH spec must
  close.
- OQ-004 — Is transitive blocking shown by default, or only on an explicit `--transitive`
  or explain surface? Blocks the default blockers behaviour.
- OQ-005 — When a cycle is detected in an acyclic overlay, should the command fail hard,
  warn and degrade, or choose per relation kind? §6 leans warn-and-degrade; the
  per-relation-kind choice is open.
- OQ-006 — Should derived consequence account for knowledge-record state (PRD-010) in v1,
  or defer governance pressure until PRD-010 is implemented? Blocks the v1 consequence
  inputs.
- OQ-007 — Should an authored priority rank override actionability in derived survey while
  `next` still prefers actionable work? Blocks how the two surfaces diverge on a ranked
  but blocked item.
- OQ-008 — How should the policy version be recorded in cache/projection output so stale
  derived results are recognisable? Blocks cache-staleness detection.
- OQ-009 — Should the derived actionability policy admit a **code-surface (path-glob)
  trigger** as a non-entity actionability input — an item held non-actionable until a
  phase's planned or touched file set matches its trigger globs — or is edit-conditioned
  dormancy a separate capability outside this graph? The graph core must stay path-free
  (§2 constraint: no dependency on file paths), so a path-trigger can only enter as a
  policy-layer mask, resolved by the existing scope-admittance predicates
  (`src/retrieve.rs`, SL-008), never as a graph edge. Distinct from the entity-relation
  triggers this spec already admits and from temporal dormancy (out of scope, §2). Blocks
  whether the architectural-trigger work (IMP-012, with IMP-013/IMP-014 as fixtures) lands
  as a channel of this capability or as a sibling spec.
- OQ-010 — Per kind, which lifecycle statuses count as **workable** (actionable when
  unblocked) versus **terminal** (excluded from default active output)? The synthesis
  model is settled; the mapping is doctrine policy (SPEC-001), but some kinds have no
  obviously work-shaped lifecycle — is a `draft` or `active` spec actionable work; is a
  `proposed` ADR a decision still to make? Blocks the policy's per-kind status-class table
  and the cross-kind `next` result. SL-047 cannot finalise its actionable set until this
  closes in SPEC-001.
