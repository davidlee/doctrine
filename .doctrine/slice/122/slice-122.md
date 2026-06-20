# RFC kind: first-class discussion artifact

## Context

Doctrine has authored kinds that *decide* (ADR), *specify* (PRD/SPEC/REQ),
*change* (slice, REV), and *record* (REC, RV). It has no home for **deliberation
itself** — the discussion of a thing before (or independent of) any governance
move. Today such discussion is stranded in conversation context, scattered across
design prose, or forced prematurely into an ADR that then carries a governance
position it has not earned.

An RFC is a discussion artifact: analogous to an ADR in being authored, durable,
and citable, but **carrying no governance position of its own**. It is a place to
think in public about *whatever the thing is*, and to link out to anything worth
linking (entities, specs, prose, external refs).

Relationship to the Revision kind (REV, ADR-013): a REV is the *auditable record*
of a structured update to governance artifacts. An RFC is the *precursor
discussion* — the deliberation a REV may later formalise. The two are
complementary, not redundant: an RFC can exist with no REV; a REV can be created
optimistically with no RFC. The exact edge between them is an open question (below).

Adjacent prior art to reconcile against during design:
- ADR-007 (Review as a first-class kind), ADR-013 (Revision as a first-class
  change-axis kind) — the precedent that a new first-class kind is itself
  architecturally significant and warrants its own ADR.
- ADR-004 (relations stored outbound-only; closed RELATION_RULES vocabulary).
- The `DEC` / free-text-ref pattern: `integrity::ensure_ref_resolves` forward-
  validates only refs whose prefix is a numbered kind in `integrity::KINDS`;
  non-entity pointers are carried as unvalidated free-text. "Link to anything"
  intersects this directly.
- Backlog IDE-007 (when a DEC record vs an ADR vs a governance surface) — bears
  on RFC's "no governance position" framing; link, don't duplicate.

## Scope & Objectives

Introduce **RFC** as a first-class authored doctrine kind:

- A `Kind` descriptor + scaffold, materialised through the existing entity engine
  (Kind-is-data pattern, `entity::materialise`) — no new engine abstraction.
- CLI surface: `doctrine rfc new` (+ `show`, and `list`/catalog integration) for
  parity with peer authored kinds.
- Storage under `.doctrine/` honouring the storage rule (structured metadata in
  TOML, prose body in MD). Authored, committed, diffable.
- Registration in `integrity::KINDS` so generic id ops (`validate`, `reseat`,
  ref-resolution) cover it.
- Install/commit wiring: `install/manifest.toml [dirs].create` +
  `.gitignore` negation for the new authored tree (the silent-uncommittable trap).
- A relation story for RFC that satisfies "link to anything worth linking" while
  respecting ADR-004's closed-vocabulary model — whatever the design lands on.
- An ADR capturing "RFC as a first-class discussion kind, governance-neutral"
  (expected to emerge from `/design`, mirroring ADR-007 / ADR-013).

Objective: a fresh agent can `doctrine rfc new "<topic>"`, write deliberation,
link it to relevant entities, and have it survive as authored, citable corpus —
with zero governance weight attached.

## Non-Goals

- Not an ADR/POL/STD: an RFC asserts **no** governance position and is never a
  source of canon. It does not appear in boot's governance sections.
- Not a Revision: it is not the auditable record of a governance update. (Edge
  case relationship deferred to design.)
- Not a backlog item: RFCs are knowledge/deliberation, not tracked work.
- No web/lazyspec projection, no MCP surface, no memory-harvest automation in
  this slice (candidate follow-ups).
- No migration of existing scattered discussion into RFCs.

## Affected Surface (provisional — design confirms)

- `src/entity.rs` — consumer of the new `Kind` const (no engine change expected).
- New verb module `src/rfc.rs` (mirrors `src/adr.rs` shape) + CLI wiring in the
  command layer.
- `src/integrity.rs::KINDS` — add the RFC kind ref.
- `install/manifest.toml`, `.gitignore` — authored-tree wiring.
- `.doctrine/rfc/` (singular, design §3) — the authored tree.
- Relation vocabulary: `RELATION_RULES` — add `RFC` to the `related`/`AnyNumbered`
  rule's sources (RFC's own edges) + a new `originates_from` row (REV→RFC). Design §1.
- `src/status.rs` — RFC count line in `doctrine status` (design §4).
- Templates for the scaffold (rfc-nnn.toml / rfc-nnn.md).

## Risks / Assumptions / Open Questions

Assumptions:
- A1: RFC is a *numbered, first-class* kind (`RFC-NNN`), not loose unnumbered
  markdown — implied by "analogous to ADRs" + "can link". Design to confirm.
- A2: The entity engine needs no new abstraction (Kind-is-data holds for ~13th
  kind). Confirm against the `GovKind` wrapper boundary.

Open questions — ALL RESOLVED in `design.md` (§ refs):
- OQ-1 (naming) → §3: `.doctrine/rfc` singular.
- OQ-2 ("link to anything") → §1: RFC's own edges ride the existing
  `related`/`AnyNumbered` rule — link to any entity, real graph citizen, no engine
  fork, no free-text adoption.
- OQ-3 (RFC↔REV edge) → §1: REV→RFC `originates_from` ("precursor of"), tier-1
  writable, outcome-neutral. ADR-004-clean (no RFC-side reverse storage).
- OQ-4 (lifecycle) → §2: minimal `open → resolved|withdrawn`; outcome-blind, no
  `accepted`; status ⊥ the REV edge.
- OQ-5 (ADR scope) → §5: one ADR — D1 governance-neutral first-class kind, D2
  precursor-to-change-axis.
- OQ-6 (visibility) → §4: omit from boot GovRows (soft, revisitable); surface in
  `doctrine status`; `rfc list` catalog.

Risks:
- R1: Scope creep into projection/MCP/memory surfaces — fenced as non-goals.
- R2: Relation-model collision with ADR-004 — **retired**. Design §1: "link to
  anything" is an additive const edit (label-first table), not an engine fork.

## Verification / Closure Intent

- `doctrine rfc new` mints `RFC-001`, scaffolds TOML+MD, tree is committable.
- `doctrine rfc show RFC-NNN` synthesises both tiers; `validate`/`reseat` cover RFC.
- RFC can be linked to at least one peer entity per the design's relation model;
  the edge renders via `inspect`.
- Existing engine suites stay green unchanged (behaviour-preservation gate).
- The governing ADR is authored and accepted.
- `just gate` clean.

## Follow-Ups

- lazyspec/web projection for RFC (cf. IMP-105).
- MCP surface for RFC if discussion benefits from agent tooling.
- Memory-harvest / dreaming integration for resolved RFCs.
- IDE-007 guidance note (DEC vs ADR vs governance surface vs RFC).
