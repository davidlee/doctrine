# IMP-143: merge SL-110 Rev 2 pencil-model UX rework from candidate/110/review-001 into main

## Context

SL-110 ("Web map UX polish") shipped PHASE-01..05 on `main`, but items 4
(concept-map edit UX) and 5 (focus transition) were rejected at audit (RV-098
F-4 blocker, F-5 major). The Rev 2 rework — PHASE-06 (new DSL ops
`rename_node_occurrence` + `relabel_rel_all`), PHASE-07 (per-cell pencil icons,
`[ ] edit all` checkbox, inline editing, delete `renderEditToggle`),
PHASE-08 (non-member focus → Semantic) — was implemented on
`candidate/110/review-001` but **never merged to `main`**.

The slice was closed on `main` with the rejected buttons-everywhere model
(`renderEditToggle` with "Edit this"/"Edit all" buttons). The working
pencil-icon + checkbox UX lives stranded on the candidate branch.

## Merge surface (18 files, all conflict-prone)

All 18 files touched by the Rev 2 commits have also changed on `main` since
the divergence point (`69b34330`):

```
src/concept_map.rs
src/map_server/routes.rs
web/map/index.html
web/map/src/app.ts, app.test.ts
web/map/src/concept-map.ts, concept-map.test.ts, concept-map.css
web/map/src/model.ts, model.test.ts
web/map/src/priority.ts, priority.css
web/map/src/render.ts, render.test.ts, graph.css
web/map/src/sidebar.css
web/map/src/state.ts
web/map/src/types.ts
```

## Approach

1. Fork `main` → `IMP-143/merge-sl110-rev2`
2. Cherry-pick or merge the four Rev 2 commits from `candidate/110/review-001`
3. Resolve TS/Rust conflicts, rebuild dist, verify vitest + cargo test green
4. Manual walkthrough confirming pencil icons, edit-all checkbox, focus transition
5. Merge to main

The Rust backend ops (PHASE-06) are the highest-risk — `concept_map.rs` and
`routes.rs` have seen heavy churn from SL-131 (MCP), SL-132, and SL-134.
