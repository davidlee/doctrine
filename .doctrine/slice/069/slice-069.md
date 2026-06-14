# Shipped memory corpus as a cohesive client onboarding anchor

## Context

Doctrine's memory splits into two trees with different fates (confirmed during
SL-068 planning):

- **Global orientation masters** — `memory/` at repo root, authored with
  `record --global` (SL-018, ADR-002 repo-empty/unanchored/evergreen class).
  These SHIP to every doctrine client via `doctrine memory sync`. Today: 14
  masters (overview, file-map, skill-map, cli-command-map, lifecycle-start
  signposts; storage-model / entity-engine / memory-model / routing-gate
  concepts; core-loop / conventions / tdd patterns; cli-source-of-truth /
  storage-tiers facts).
- **Project-local items** — `.doctrine/memory/items/`, committed to THIS repo
  only (~373 records). Doctrine's own build knowledge. The installer creates an
  empty `items/` tree in a client; this repo's items never ship.

The shipped corpus is a client's ambient orientation: in any doctrine-using repo
it is what `retrieve-memory` draws on and what anchors a cold agent's
understanding of the condoned workflow. But it has never been audited as a
*cohesive whole*. It grew master-by-master; SL-068 PHASE-06 will add more
(candidate-workflow orientation). Without a curation pass the corpus risks
drift: gaps (a condoned workflow with no orientation master), redundancy
(masters overlapping each other or the boot snapshot), staleness (a master
describing a since-changed surface), and misclassification (orientation stranded
in project-local items, or build-local noise shipped as a master).

`mem.pattern.distribution.shipped-not-reachable` already records the failure
mode: a shipped surface is invisible unless boot or a skill points at it. The
shipped memory corpus is exactly such a surface — and its cohesion with the boot
snapshot, the skills, and `using-doctrine.md` has not been verified end to end
from a client's vantage.

## Scope & Objectives

1. **Inventory & classify.** Enumerate the global masters (`memory/`) and assess
   each: still accurate? still orientation (vs build-detail that drifted in)?
   Cross-check project-local `items/` for records that are actually client-facing
   orientation misfiled as project-local.
2. **Test corpus cohesion for onboarding.** Treat the shipped masters as a
   client's onboarding set and ask: from these alone (plus boot + skills), can a
   cold agent in a client repo orient to the storage model, the routing gate, the
   canonical change loop, and the major surfaces? Identify gaps and redundancy.
3. **Test corpus cohesion for workflow.** Ensure every condoned workflow a client
   would run (route → slice → design → plan → execute → audit → close; memory
   record/retrieve; dispatch candidates after SL-068) has a discoverable
   orientation anchor in the shipped corpus, not only in skills.
4. **Reconcile against the boot snapshot.** The boot snapshot already projects a
   Memory index and governance. Define the division of labour: what the shipped
   corpus anchors vs what boot/skills/`using-doctrine.md` carry — eliminate
   drift and duplication, keep one source per fact.
5. **Curate.** Add missing orientation masters, retire/merge redundant or stale
   ones, reclassify misfiled records (promote stranded orientation to global,
   demote shipped build-noise to project-local), so the shipped corpus is a
   deliberate, cohesive anchor — not an accretion.
6. **Guard against future drift.** Leave a lightweight, durable check or
   convention so the corpus stays cohesive as masters are added (e.g. an audit
   verb, a `memory sync` sanity surface, or a documented authoring rule).

## Non-Goals

- Rewriting the memory engine, ranking, or `record/find/retrieve/sync` mechanics
  (SL-005/007/008/018 shipped those). This is corpus curation, not engine work.
- Migrating or mass-editing project-local `items/` beyond reclassifying the few
  that are misfiled orientation.
- The boot snapshot's generation mechanics (SL-011). This slice reconciles the
  corpus *against* the snapshot's role; it does not re-architect `doctrine boot`.
- Per-client custom memory authoring guidance beyond what doctrine ships by
  default.
- Folding in SL-068's PHASE-06 memory work — that lands with SL-068; this slice
  audits the corpus's cohesion as a whole, after.

## Summary

Audit the two memory trees, confirm which records ship, and curate the shipped
global corpus into a cohesive onboarding + workflow anchor for client projects:
fill gaps, retire redundancy, reclassify misfiled records, reconcile the
division of labour with the boot snapshot and skills, and leave a drift guard so
cohesion survives future additions.

## Follow-Ups

- Depends on SL-068 PHASE-06 landing its candidate-workflow orientation into the
  global corpus first (soft sequence) — this audit then includes it in the
  cohesion pass.
- If curation surfaces a need for a `memory` audit/lint verb, that may spin out
  as its own tooling slice.
