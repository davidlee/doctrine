# CHR-024 RFC — locked design position

> Output of the pre-RFC design conversation (David + Claude, 2026-06-23). This is
> the **agreed spine** the RFC authoring phase inherits — not the RFC itself. Items
> marked LOCKED are settled; UNLOCKED items are deferred sub-decisions; OPEN items
> are the remaining work before/within RFC authoring.
>
> Read after: research-synthesis.md, then this. The synthesis is the model as-is;
> this is where we've decided to take it.

---

## 0. Axis triage (LOCKED)

The seven findings (F-1..F-7) + friction items collapse to four axes at different altitudes:

- **Axis A — backlog kinds are second-class.** `governed_by`, `related`, `reviews`
  exclude BACKLOG sources (F-1/F-2/F-4/F-5). **Easy win, slice out now**, need not
  wait on the RFC. Extend source-sets; add a review/audit outlet for BACKLOG/SL.
- **Axis B — overloaded edge intent.** `specs`/`slices` conflate multiple intents
  (F-3, IMP-149). **This is the RFC's purpose.** Everything below is mostly B.
- **Axis C — coverage / completion.** (F-6, IMP-141.) **Mostly NOT a vocabulary
  problem** — lives in validate rules, `/close` gating, SPEC-002 requirement
  reconciliation. Model should be *expressive enough* to support it; do **not** mint
  relation labels for it. `partially_addresses` is a C predicate, kept OUT of the
  role grammar.
- **Axis D — decomposition.** (F-7.) Real and semantically significant. **Carve out
  as a sibling feature/spec**, but its modelling informs B (see §6). Not folded into
  the B vocabulary cleanup.

## 1. The layering (LOCKED)

Graph-effect is **not** a peer axis of a relation. Two layers, kept separate:

- **Relation contract** (`RELATION_RULES`): structural truth — sources, target-kind,
  tier, and the role grammar (§3). Durable.
- **Consumer policy** (priority overlay / `/close` / `validate`): graph-effect,
  gating, eviction, closure. A *projection* over selected edges. A label's gating
  behaviour is **declared in the consumer**, not in the vocabulary. IMP-047 is the
  principled version of that assignment (a new label silently gets `Reject,Unbounded`
  until the priority overlay assigns policy).

Corollary: actionability (`needs`/`after`/IMP-047 `gates`) belongs to the
dep/seq/actionability layer, not the general semantic relation model.

## 1b. Design law: derivable, not relational (LOCKED)

> **Do not encode in the relation what is derivable from entity state or facets.**
> The edge carries durable *structural intent*; everything else *projects* over it.

Found three times, same shape — each an "apparent missing label" that is really a
projection over other data:

- **Coverage / completion (Axis C)** — not a label → `validate` / `/close` / SPEC-002.
- **Temporal (`slices` planned-vs-done, IMP-149)** — not a label → derive from target
  **lifecycle status** (§5).
- **Altitude (SL→PRD vs SL→SPEC; product/C4 ladders)** — not a role → derive from
  target **facet** (§6b).

Same spirit as §1 (graph-effect is consumer policy, not relation-intrinsic). This law
is the RFC's discipline against vocabulary bloat.

## 2. The deciding principle: label vs role (LOCKED — the RFC's spine)

> **Separate LABEL** when the distinction changes the edge's **structure,
> graph-effect, or inbound semantics.**
> **ROLE facet** when it only refines the **intent** of an otherwise
> structurally-identical edge.

This is the rule the RFC applies across the whole vocabulary.

## 3. Type-safety resolution (LOCKED)

"Common mechanism across entities" and "robust type safety" are **not incompatible** —
the tension was an artifact of keying the target constraint on the *label*. Fix: key
it on the **role**.

- Move target constraint from `(source, label) → TargetSpec` to
  `(source, label, role) → TargetSpec`.
- **Two-level closed grammar:**
  - `(source_kind, label) → legal roles` (e.g. `references` from SL admits
    {implements, reviews, scoped_from, bears_on}; from RV admits {reviews} only).
  - `(source_kind, label, role) → TargetSpec` (the role carries the target gate;
    e.g. `references(implements) → {SPEC,PRD,REQ,ADR}`, `references(reviews) →
    AnyNumbered`).
- **Roles are a closed enum**, not free text.
- Not new machinery: `RELATION_RULES` is already multi-row per label (`related` ×2,
  `supersedes` ×3). This adds one column. Lockstep enum-order test + overlay
  allocation survive.

`specs`'s real defect was a **noun verbing** — the missing verb *is* the role. Once
the role supplies the verb, the target set widens to actual canon ({SPEC,PRD,REQ,ADR})
and becomes role-determined.

## 4. Applied vocabulary decisions (LOCKED unless noted)

- **work→canon reference family collapses into `references` + role.**
- **Role candidates:** `implements`, `reviews`, `scoped_from`, `bears_on`.
  (Explicitly NOT `partially_addresses` — Axis C. Avoid `touches` — too vague; use
  `bears_on`.)
- **Structural primitives stay distinct labels:** `part_of`, `supersedes`,
  `exclusive_with`.
- **`governed_by` — stays a label.** Authority is a distinct structural thing with
  its own inbound ("governs") and consequence weight; not just a reference role.
- **`related` — collapses into `references(role=related)`.** The degenerate
  no-committed-intent reference. Shrinks the bag.

## 5. Temporal vs lexical overload (LOCKED)

Two "ambiguities" that look identical in the chore have **opposite fixes**:

- **Lexical overload** (`specs`): genuinely multiple intents → **role facet**.
- **Temporal projection** (`slices` planned-vs-done, IMP-149): **one stable edge read
  against the target's lifecycle status.** Planned = edge + slice open; done = edge +
  slice closed. Minting `planned_by`/`completed_by` is a category error (forces
  rewriting the edge as status moves). **No new label** — derive the reading from
  target status. Dissolves the structural half of IMP-149.

## 6. Decomposition modelling note (informs B; feature carved to D)

- Single `part_of` primitive, **child→parent** (outbound-from-child, ADR-004-clean);
  inbound derived (`contains` / `parts`).
- **Cross-altitude, one primitive applied thrice:** PRD product tree; SPEC C4 tree
  (`parent` becomes a specialization of `part_of`, possibly carrying a C4-level
  facet); SL/epic work hierarchy.
- **Structural-first.** Close-gating / actionability is consumer policy (§1), not the
  definition of the relation.
- Keep **strictly separate** from reference-intent. "part of this IDE" is
  `references(scoped_from)→IDE`, NOT `part_of` — the work is *derived from part of*
  the idea, not structurally a part of it.

## 6b. Altitude lattice (functionally missing; deferred to D) (LOCKED stance)

`.doctrine/concept-map/001/concept-map-001.toml` already specifies a full altitude
lattice that the relation model lacks:

- **Product ladder:** Domain ⊃ Capability ⊃ Feature ⊃ User Story (`belongs to`).
- **C4 ladder:** System Context ⊃ Container ⊃ Component ⊃ Code (`decomposes`).
- Declared **symmetric** (Product Altitude ⟷ C4 Level). Feature/Capability/Domain are
  all *`is a` PRD*; the four C4 specs are all *`is a` Technical Spec*.

Stances (LOCKED):
- **Altitude is a target-side facet, not a role** (§1b). PRD carries `product_altitude`
  ∈ {Domain,Capability,Feature,UserStory}; SPEC carries `c4_level` ∈
  {SystemContext,Container,Component,Code}. The work→artefact role stays
  altitude-agnostic; "implements a User Story vs contributes-to a Domain" reads off the
  target facet. So the SL→PRD/SPEC residue resolves to **one `implements`, no new role.**
- The `belongs to` / `decomposes` chains are the **seed spec for the D `part_of`
  carve-out** — decomposition is already drafted here, not speculative.
- **`is_kind_of` reconciliation:** the "is a" taxonomy (Feature is-a PRD) is load-bearing
  but lives at the **type level, in the concept map** — exactly where a type ontology
  belongs. The §10 rejection of `is_kind_of` stands at the **numbered-entity-instance**
  level. Both hold.

Deferred: building the altitude facets + lattice is **not** this RFC; note and revisit
post-draft.

## 7. `exclusive_with` (candidate, LOCKED as in-scope-to-consider)

Symmetric "these are mutually-exclusive alternatives — pick one." Novel; nothing
expresses it today (not `supersedes`, which is after-the-fact). Eviction consequence
is **future consumer policy** (§1), not baked into the relation definition.

## 8. Free property to take: role-derived inbound (UNLOCKED — leaning yes)

Because the role names the verb, the inbound reciprocal can be role-derived:
`inspect SPEC-018` → "implemented by SL-x · reviewed by CHR-024 · scoped SL-y" instead
of flat "specs ×3". Strictly more legible; the natural reciprocals `inbound_name`
fumbles today. **Leaning role-derived; stays unlocked** pending the rest.

## 9. Accepted costs (LOCKED)

1. Roles are a closed enum (each new intent is a code change — no worse than a new
   label today). ✅ accepted.
2. Migration: existing `specs`/`slices` edges need a role backfilled; `migrate` stamps
   a default (`implements`, or explicit `unspecified` to force triage). ✅ accepted.
3. Surfaces carry the role: `CatalogEdgeLabel`, `inspect`, `relation list`, web graph
   edge rendering gain a role dimension. ✅ accepted.
4. Inbound role-derived vs label-flat → §8, UNLOCKED.

## 10. Rejected (KR-completeness temptations) (LOCKED)

- `is_kind_of` — subsumption is type-level; doctrine's kinds are code-closed; between
  entities it has no crisp meaning distinct from `part_of`/`related`.
- `is_instance_of` — needs a class/template/pattern *numbered* entity; none exists
  (memory patterns are free-form, not numbered).
- `composes_with` — too vague; collapses into `interactions`/`related`/`part_of`.

## 11. P1 evidence — existing-edge classification (DONE)

Exhaustive classification of the B-relevant populations (`relation list`, authoritative).
Full detail in `p1-classification.md`. Headline:

- **Grammar is complete for entity→entity.** {implements, reviews, scoped_from,
  bears_on, related} covers **100%** of the 113 specs+related+drift edges. No missing
  role. (`reviews` has *zero* current instances — absent-edge case.)
- **Current labels mismap intent at scale** (the grammar *corrects*, not just renames):
  - `specs`: SL→ = implements (~44, ok); **IMP→/RSK→ mean bears_on/scoped_from** (12
    mismapped — a risk cannot *implement*).
  - `related`: peer reading is the **minority** — RFC→bag = bears_on (~26), SL→backlog
    = **scoped_from** (~13), GOV→GOV = pure related (4), SL→SL = peer/seq (4).
  - `slices`: **100% BACKLOG→SL, temporal** (§5) — confirmed, no role.
  - `drift` (5): escape hatch — bears_on-memory (2), bears_on-file (1),
    decomposition/"carved out from" (1, = F-7 in the wild), feeds-into/seq (1).
- **`exclusive_with`: zero instances** — confirmed speculative; RFC marks "modelled,
  not demanded," does not ship.

## 12. Deferred queue — note-and-revisit AFTER RFC draft (OPEN)

- **Non-entity-target edge.** The one thing `references`+role **cannot** absorb: targets
  outside the numbered space (memory, file path, glob, vec-of-files). `drift` is abused
  for exactly this. Shape: `(label+role)[src: entity] → [target: non-entity | file |
  glob | vec]`. Backlog: **IMP-012** (architectural triggers — structural triggering
  condition on file change), concept-member→entity linkage (cf. **IDE-015** bridge
  concept-map↔graph). RFC notes; does not solve.
- **Altitude lattice** (§6b) — product/C4 facets + `part_of` ladder. Seed:
  concept-map-001. Deferred to D.
- **Prose-hunt flock (P2).** Fan-out agents to find *absent* relations expressed as
  prose (governance-in-prose F-1, review-in-prose F-5, decomposition-in-prose F-7).
  Deferred — note-and-revisit.

## 13. Ready to draft (OPEN)

Open calls for the RFC body, now narrow:
- The two boundary rulings are **set** (§4): `governed_by` stays a label; `related`
  collapses into `references(role=related)`.
- The **non-entity-target ruling** (§12) — does the model keep a free-text escape
  hatch, or force the IMP-012/concept-member path? RFC states the question, defers the
  build.
- **§4 cut** — how far the `references` collapse extends — is set by §4; P1 confirms the
  grammar covers the real population. No blocker remains to authoring.

---

*Recorded 2026-06-23 as CHR-024 pre-RFC locked position. Next: real-case enumeration
(§11), then RFC authoring.*
