# Governing commitment vertical experiment

## Context

Doctrine has good representations of *things* — requirements, decisions,
phases, selectors, evidence, review findings — but no representation of **a
normative commitment travelling through the development process**. That
object is currently smeared across design prose, plan prose, criteria,
selectors, implementation notes, review responses, and evidence. The seed
analysis (`gordian-seed.md` in this slice folder — preserved from the
gitignored `gordian.local.md`) decomposes the missing object into:

- **Claim** — "I assert X should/must/is true"; has provenance and basis; may
  be proposed, contested, superseded, rejected.
- **Commitment** — a claim admitted as governing intent; what implementation
  is *not allowed to accidentally change*.
- **Obligation** — an execution-scoped responsibility to make/keep
  commitments true. **Already exists: the `EX` criterion.** The missing
  object is *what an `EX` owes*, not a new obligation kind.

Adjacent deliberations bound the shape:

- RFC-027's obligation study (`H9`) **rejected** a plan-level obligation
  graph — seven of eight candidate fields restated existing owners. This
  slice must not resurrect it.
- RFC-029 owns the *proving* seam — the `VT`→`EX` binding, evidence
  staleness, failure cause. This slice sits immediately **upstream**: the
  design-intent→`EX` seam, where the runtime model is still bare `{id,text}`.
- RFC-020 (ledgered facet claims) is the existence proof for the mechanics
  to steal: provenance, derived authority, append-only supersession, conflict
  rather than silent winning, evidence tiers, derived operative view — steal
  the **ledger semantics**, don't generalise `comparison::claims`.
- RFC-026 `G15`: intent and implementation approach are conflated — execution
  has no explicit boundary between "adapt freely" and "changing this changes
  the design". The adaptation envelope answers it.

## Scope & Objectives

One deliberately narrow vertical, end to end — no generic schema first:

1. **Extraction.** During managed design, the agent may extract candidate
   commitments from accepted prose/decisions. Extraction carries provenance;
   it is never self-authorising.
2. **Admission.** An explicit trusted step produces the governing set.
   Authority derives from *who admitted the record and through which
   operation*, never from a worker-writable field (RFC-020 lesson).
3. **Discharge links.** Each `EX` criterion links to the commitment(s) it
   discharges.
4. **Semantic operations.** An implementer gets exactly two: **challenge
   commitment** and **propose refinement** — both append-only,
   evidence-bearing. Crossing a commitment boundary emits a challenge, not a
   silent change.
5. **Semantic diff.** At reconciliation, render the diff between the
   dispatched and final governing sets — implementation diff vs contract
   diff, separately.

Semantic payload stays deliberately weak (stable identity, statement,
canonical references where known, provenance, basis). Strong typing goes on
the governance: proposer, admitter, governing/superseded lineage, discharging
obligations, contest state, bearing evidence. Lifecycle axes (epistemic /
lineage / execution / proof) stay orthogonal — no fused status enum.

## Non-Goals

- **No generic predicate system** — no subject/predicate/modality/strength
  schema, no attempt to structure every design sentence.
- **No new obligation kind** — `EX` is the obligation (RFC-027 `H9` stands).
- **No new actionability engine** — the semantic graph (what must remain
  true) and the actionability graph (what work can happen when) stay
  separate relations; this slice touches only the former.
- **No universal change record.**
- **The `VT`→`EX` proof binding is RFC-029's cluster**, not this slice's —
  this slice stops at the commitment→`EX` seam and must compose with that
  work, not restate it (boundary: RFC-029 owns *proving*, this owns *what is
  owed*).
- No worker-writable `authority` field, anywhere.

## Affected surface (coarse)

Managed design run (extraction + admission), the plan/phase criterion model
(discharge links), reconciliation (semantic diff), a probable new
commitment module, and the CLI verbs to drive it. Fenced by `scope-relevant`
selectors; the exact touch-set is design's job.

## Risks, assumptions, open questions

- **R1 — ontology creep.** The failure mode the seed warns about (and
  RFC-027 Stage 0 demonstrated): typed payloads must be earned by a consumer,
  not designed up front.
- **R2 — parallel implementation.** Ledger mechanics exist in
  `comparison::claims`; design must decide reuse vs pattern-transplant
  explicitly, not build a third ledger by accident.
- **A1** — the managed design run is the extraction surface (its
  `InquiryNode` already separates provenance/lifecycle/disposition/needs —
  the healthy pattern to follow).
- **OQ-1** — is the adaptation envelope (reserved / bounded / delegated /
  observational, per RFC-026 `G15`) in this vertical's minimal payload, or a
  follow-up once challenge/refine mechanics exist?
- **OQ-2** — does the "contract revision" framing ride the existing Revision
  kind (ADR-013) or is the dispatched governing set its own projection?
- **OQ-3** — altitude, *adjudicated at scoping*: RFCs carry no binding
  authority; canon is encoded as ADR / POL / Spec. Expected vehicle: author
  governance (exact form TBD in planning) as a **Revision** and apply it
  during reconciliation. Planning owns the call on which instrument(s).
- **OQ-4** — sequencing against RFC-029: can the discharge link land before
  the `VT`→`EX` binding, or do they share a criterion-row surface that forces
  an order?

## Verification / closure intent

The experiment is judged by whether it catches the expensive failures the
seed enumerates, on real slices:

- design intent that never reaches a plan;
- plan obligations with no governing reason;
- implementation silently changing intent;
- reviewers rediscovering why a constraint exists;
- corrections smeared across several prose copies;
- needless human escalations for adaptations inside the envelope.

Closure requires the vertical demonstrated on at least one real slice's
lifecycle (extraction → admission → discharge links → a challenge or
refinement → reconciliation semantic diff), with the measurement honestly
reported even if negative — RFC-027's Stage-0 "no" is a valid outcome shape.

## Summary

## Follow-Ups
