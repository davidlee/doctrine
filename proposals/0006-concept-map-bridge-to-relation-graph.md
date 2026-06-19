---
seq: 0006
scope: codebase
target: src/concept_map.rs, SL-074 (v1 boundary), relation_graph
confidence: med
reversible: yes (proposal only; no code or design change — analysis read-only)
---
## What
Doctrine ships **two relationship-graph systems that never touch**:

1. `relation_graph` — the *real* topology, derived from authored `[[relation]]`
   edges across the whole corpus (governed_by, specs, supersedes …). Machine-truth,
   validated, the substrate behind `inspect`/`blockers`/priority.
2. `concept_map` (the `CM` kind, SL-074) — a *hand-drawn* DSL diagram. Nodes and
   edges are **free text**; `parse_dsl` (`src/concept_map.rs:330`) builds nodes/
   edges purely from the authored DSL block with no reference to any real entity.

The seam between them already exists and is deliberately inert. `parse_dsl`'s
`check` emits an `EntityRefLike` diagnostic (`src/concept_map.rs:237,881`) when a
node label matches `[A-Z]{2,5}-\d{3}` — i.e. it *detects* that a hand-drawn node
names a real entity — but does nothing with it: there is **zero** reference to
`relation_graph`, `require_minted`, or entity existence anywhere in the module
(`grep -nE 'relation_graph|require_minted|EntityKey' src/concept_map.rs` → empty).
The diagnostic is "purely informational" by design (SL-074 `design.md:297,710`,
"never an error").

Crucially, SL-074 framed this as a **v1 scoping decision, not a permanent
boundary**: `design.md:708` — "Relation labels free-text? **Yes, free-text in
v1**." The bridge was anticipated and left open. As-is, the consequence is the
classic rotting-diagram problem: a user draws `SL-046 → governs → SPEC-018` by
hand; the real `[[relation]]` edge later changes; the concept map silently lies,
and nothing in doctrine notices — even though doctrine *holds the true edge*.

## Options
1. **Leave v1 as-is (free-text, informational).** Tradeoff: zero cost, keeps
   concept maps as a pure narrative/whiteboard kind; accepts that they drift from
   and are unverifiable against real topology — a hand diagram, nothing more.
2. **Validate-only bridge (v2a).** When a node is `EntityRefLike`, resolve it
   against the corpus (reuse `relation_graph::require_minted`); upgrade the
   diagnostic to flag *non-existent* entity-looking refs (`SL-999`), keep existing
   refs informational. Tradeoff: small, rides an existing helper, kills the
   "diagram cites a dead id" failure; does not yet detect edge drift.
3. **Enrich + drift-detect bridge (v2b).** On top of (2): for nodes that are real
   entities, diff the hand-drawn edges against the real `[[relation]]` edges from
   `relation_graph` and report divergence ("you drew `governs`; the corpus has no
   such edge" / "corpus has `specs SPEC-018` you didn't draw"). Tradeoff: the
   topology-capitalizing payoff — a concept map becomes a *curated overlay on live
   truth, kept honest by the graph* — but it couples concept_map → relation_graph
   (check the ADR-001 layering: concept_map would import the engine; relation_graph
   must not import back — likely fine, same direction as `inspect`).

## Recommendation
Option 2 now (cheap, strictly additive, closes a real defect — diagrams citing
dead ids), explicitly as the first step of the SL-074 v2 the design left open; and
file Option 3 as the strategic follow-up. Rationale: (2) is low-risk and reuses
`require_minted` (no new existence path — same discipline as 0003/0004), while (3)
is the actual prize but is a design question (overlay semantics, render of drift,
the layering coupling) that deserves its own slice. Sequencing 2→3 lets the bridge
prove out on validation before taking on drift semantics.

Why this matters to the focus: concept maps are the one place a *human* narrates
architecture, and the relation graph is the one place the *machine* knows it.
Bridging them is the single feature that makes "our architecture diagram, kept
honest automatically" true — exactly the indispensable-to-teams capability.

Decisions deferred to YOU:
- (a) **is the v1 free-text boundary intentionally permanent, or v2-open?** (design
  says v1 — confirm you still want to cross it).
- (b) **2 vs 3 as the next step** — validate-only, or go straight to drift-detect.
- (c) **coupling posture** — does concept_map (an authored leaf-ish kind) get to
  import `relation_graph` (engine)? `inspect` already composes both at the command
  layer; a concept_map→engine import may be cleaner than a command-layer join. An
  ADR-001 layering call.

## Next doctrine move
```
# read the v1 boundary + the inert seam (read-only):
doctrine slice show SL-074
sed -n '690,712p' .doctrine/slice/074/design.md     # the v1 free-text decision

# capture the v2 bridge as work (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "Concept-map ↔ relation-graph bridge: validate \
  EntityRefLike nodes against the corpus (v2a), then drift-detect hand-drawn edges \
  vs real [[relation]] edges (v2b) — SL-074 v1 left free-text open" \
  --tag area:relations --tag area:concept-map
# if pursued as design:
/route                                              # → /slice
```
(Verbs described, NOT executed.)

## Illustration (optional)
None applied. The v2a shape is a one-line resolve at the `EntityRefLike` site
(`require_minted` over the scanned projection) — but the load-bearing decision is
(c) the layering posture, which a speculative diff would prejudge. Kept as prose.
