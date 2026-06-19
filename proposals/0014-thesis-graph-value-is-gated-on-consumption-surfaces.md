---
seq: 0014
scope: capture
target: discovered — cross-cutting thesis over proposals 0003/0006/0007/0009/0011
confidence: med
reversible: yes (a framing/prioritization capture; commits to nothing)
---
## What
A pattern emerged across this session's analysis, worth capturing as a single
thesis because it should shape how the review queue is prioritized:

**Doctrine's graph topology is rich on the inside and thin on the outside.** The
internal model is mature and well-governed — verified this session: corpus FK/orphan
clean (`spec validate`: "corpus clean"), all 47 spec lineage edges resolve,
ADR-001 layering enforced by a whole-crate fitness test, the relation contract
(SPEC-018) precise, client a thin renderer (no server-policy duplication). The
*capture* and *storage* of topology is essentially done.

The recurring gaps are all on the **consumption / outward surface** — what a human
or team or external tool can *do* with the graph:
- **0003** — no transitive impact/blast-radius query; `inspect` is 1-hop only,
  while `blockers --transitive` and `retrieve --expand N` prove the walk exists.
- **0006** — concept maps (the human narrative view) are decoupled from the real
  relation graph; no validation, no drift detection.
- **0007** — `export` ships only the agent-facing lazyspec; no graph interchange
  (DOT/GraphML/Cypher) for the tools teams already run.
- **0009** — the authored-priority slot (the human's "pin this" lever over the
  derived worklist) is designed (SPEC-001 D10) and requirement-backed (REQ-054) but
  unbuilt and untracked.
- **0011** — no unified `doctrine doctor` health gate; integrity checks exist but
  are scattered across four commands a team must know to assemble.

By contrast, most open *backlog* improvements push the other way — deeper internal
modelling (IMP-053 record↔record, IMP-069/070/097 altitude validation, IMP-095/063
supersession vocab, IMP-105 lazyspec projection coverage). Valuable, but they
refine a model that already works; they don't move the needle on
"indispensable to product teams." The marginal return on more internal correctness
is lower than the marginal return on *exposing what's already correct*.

This is the answer to the standing focus question. The graph becomes indispensable
to teams at the moment it (a) answers "what does my change touch?" (0003), (b) keeps
the human's diagram honest (0006), (c) leaves the tool into Gephi/Neo4j/dashboards
(0007), (d) lets a human assert priority over the derived order (0009), and (e)
gives one trustworthy "is the graph sound?" gate for CI (0011). None of these need
new *modelling* — they need new *surfaces* on the model that exists.

## Options
1. **Adopt a "consumption surfaces first" theme** for the next planning cycle —
   prioritize 0003/0006/0007/0009/0011 (the outward surfaces) ahead of the
   internal-modelling backlog. Tradeoff: maximizes movement on the team-indispensable
   axis; defers internal refinements that may be prerequisites for some surfaces
   (e.g. IMP-020 cordage-walk unification gates 0003).
2. **Sequence by dependency, not theme** — let prerequisites pull order (IMP-020 →
   0003; SPEC-018 spine/0005 → richer export). Tradeoff: cleaner build order, but
   risks staying in internal-modelling indefinitely because there's always one more
   prerequisite.
3. **No change** — treat each proposal independently on its own merits. Tradeoff:
   loses the strategic lens; the consumption-surface pattern is the actual product
   signal and dropping it means re-deriving it per-item.

## Recommendation
Option 1 with a dependency caveat: make "consumption surfaces" the explicit next
theme, and within it respect the few real prerequisites (IMP-020 before 0003; the
`/api/graph` serialization is already the reuse seam for 0007, so it has none).
Rationale: the internal model is the asset; its ROI is currently trapped because
little can be *done* with it from outside. The cheapest, highest-leverage moves are
all surfaces, and several reuse existing seams (cordage `reachable`, `/api/graph`,
the D10 comparator) rather than new machinery.

Decision deferred to YOU: whether to run the next cycle as a themed
"surfaces" push (and if so, which two to start — my pick: **0011 doctor** as the
cheapest team win, and **0003 impact** as the flagship, gated on IMP-020), or to
keep prioritizing per-item. This capture commits to nothing; it only names the
pattern so you can decide deliberately rather than by drift.

## Next doctrine move
```
# review the surface cluster together against the internal-modelling cluster:
ls proposals/000{3,6,7,9}-*.md proposals/0011-*.md     # the surfaces
doctrine backlog show IMP-020                            # the one real prerequisite (gates 0003)

# if adopting the theme, capture it as a tracking idea (NOT executed — fence):
doctrine backlog new idea "Theme: graph consumption surfaces (impact query, \
  concept-map bridge, graph export, authored priority, doctor gate) — expose the \
  mature internal model outward; prioritize over further internal modelling" \
  --tag area:relations --tag area:cli
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — a thesis/prioritization capture. The evidence is the five cross-referenced
proposals and this session's clean-corpus findings, not a diff.
