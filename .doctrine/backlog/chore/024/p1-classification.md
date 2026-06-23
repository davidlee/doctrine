# CHR-024 P1 — existing-edge classification (evidence)

> Exhaustive classification of the Axis-B-relevant relation populations against the
> locked role grammar {implements, reviews, scoped_from, bears_on, related}. Source:
> `doctrine relation list --label X` (authoritative, all `resolved`/`free_text`).
> Census totals: members 321, reviews 145, governed_by 89, slices 77, owning_slice 70,
> specs 60, requirements 52, related 48, revises 23, parent 21, descends_from 18,
> drift 5, supersedes 5, interactions 4, consumes 1. B-relevant = specs/related/drift
> (113) + slices (77, temporal).

## `specs` (60) — by source kind

| source→target | true role | n | note |
|---|---|---|---|
| SL → PRD/SPEC | **implements** | ~44 | clean; the delivery vehicle |
| IMP → PRD/SPEC | **bears_on / scoped_from** | 8 | mismapped — an improvement doesn't *implement*; its spawned slice does (IMP-012/013/014→PRD-011, IMP-016→PRD-012/013, IMP-019/115→SPEC-001, IMP-093→SPEC-019) |
| RSK → SPEC | **bears_on** | 4 | mismapped — a risk *cannot* implement (RSK-001..004→SPEC-001) |

## `slices` (77) — temporal, no role

100% BACKLOG→SL (IMP 49, ISS 13, RSK 9, CHR 4, IDE 2). One stable edge; planned-vs-done
is the **target slice's lifecycle status**, not a label (§5). Confirmed.

## `related` (48) — peer reading is the minority

| source→target | true role | n |
|---|---|---|
| GOV → GOV | pure **related** | 4 (ADR-002→001, ADR-004→002, ADR-010→004, ADR-014→013) |
| RFC → bag | **bears_on** (concerns) | ~26 (RFC-001 ×8, RFC-002 ×18) |
| SL → backlog (IMP/IDE/ISS) | **scoped_from / addresses** | ~13 |
| SL → SL | peer/companion (maybe `part_of`/seq) | 4 (SL-107→101, SL-112→111, SL-143↔144) |
| SL → RFC | **scoped_from** | 1 (SL-142→RFC-002) |

## `drift` (5) — the escape hatch (all free-text)

| edge | what was meant |
|---|---|
| CHR-021 → IMP-148 "feeds into this audit" | bears_on / sequence |
| CHR-021 → mem.pattern.distribution.shipped-memory-authoring | **bears_on a MEMORY** (non-entity target) |
| CHR-023 → SL-143 "carved out from" | **decomposition** (F-7 in the wild) |
| IMP-148 → mem.concept.doctrine.memory-model | **bears_on a MEMORY** (non-entity target) |
| IMP-150 → install/review-ledger.md | **bears_on a FILE** (non-entity target) |

## Verdicts

1. **Grammar complete (entity→entity):** 100% of 113 edges classify into the 5 roles.
   No missing role. `reviews` = 0 instances (absent-edge case → P2).
2. **Labels mismap at scale:** ~25 edges carry a different intent than their label
   asserts (12 `specs`, ~13+ `related`). The role grammar *corrects*, not just renames.
3. **`exclusive_with`: 0 instances** — speculative; model it, don't ship it.
4. **Boundary the role grammar can't absorb:** non-entity targets (memory ×2, file ×1
   in `drift`). Needs the `(label+role)→[file|glob|vec]` edge — deferred (§12, IMP-012).
5. **SL→PRD/SPEC altitude residue** resolved: altitude is a target facet, not a role
   (§6b) — one `implements`.

*Recorded 2026-06-23 as CHR-024 P1. Position spine: design-position.md.*
