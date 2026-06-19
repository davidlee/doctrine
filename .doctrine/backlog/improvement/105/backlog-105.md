# IMP-105: Extend lazyspec projection to new entity kinds (POL/STD/RV/REC/REV/CM/knowledge)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Origin

Split out of SL-026 (lazyspec read-only projection) during its 2026-06-19 design
re-validation. SL-026 was scoped 2026-06-08; in the ~800 commits since, doctrine
grew several entity kinds the original projection node set never covered. SL-026
deliberately locks a **minimal v1** node set — `{slice, spec, adr, backlog, plan}`
— and defers the rest here (design decision (a), 2026-06-19).

## Scope

Extend `doctrine export lazyspec` to project the entity kinds that landed after
SL-026 was scoped:

- **governance** — policy (`POL`), standard (`STD`)
- **review** (`RV`), **reconciliation** (`REC`), **revision** (`REV`)
- **concept-map** (`CM`)
- **knowledge records** — assumption (`ASM`), decision (`DEC`), question (`QUE`),
  constraint (`CON`)

For each: a lazyspec `TypeDef` (prefix/dir/icon/plural), a status→wire-string map
arm, and edge handling for the axes these kinds source. The unified relation layer
(`src/relation.rs`, SL-048) already enumerates the relevant `RelationLabel`
variants and their source/target kinds — `Shapes`, `Spawns`, `GovernedBy`,
`Reviews`, `OwningSlice`, `DecisionRef`, `Revises`, `Contextualizes`,
`Consumes` — so this is additive type+status+edge wiring, not new read machinery.

## Why deferred, not done

In SL-026's v1, edges *to* these kinds (e.g. slice `governed_by` → `POL`/`STD`)
already project as outbound relations but their targets fall outside the emitted
corpus, so lazyspec drops them silently (`validate_ignore` suppresses
`BrokenLinkRule` — SL-026 design §5.5). Emitting the kinds turns those dangling
targets into live graph nodes. Until then the projection is lossy-by-design, which
is the stated v1 posture.

## Dependency

Rides SL-026's wire format + `project` function. Pick up once SL-026 lands.
See SL-026 design §5.3 (node/edge mapping) and the follow-up note in
`slice-026.md`.
