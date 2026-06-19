---
seq: 0021
scope: capture
target: discovered — triage aid over proposals 0001–0020
confidence: high
reversible: yes (a prioritization aid; commits to nothing — pure reading order)
---
## What
Twenty proposals are queued (0001–0020). This is a **triage guide** so the 5am
review is fast: process by (confidence × value ÷ effort), with dependencies noted.
Distinct from 0014 (the *thematic* "consumption surfaces" thesis) and from the
eventual index-summary — this one is *what to action first*. Every item below still
defers its decision to you; this only orders the reading.

## Tier A — concrete defects, high confidence, cheap (do first)
- **0012** — SPEC-001/002 missing `parent` edges; priority subtree detached from C4
  root. *Fix: add 1–2 `parent =` lines.* High conf.
- **0013** — SPEC-005 stale: ADR `supersedes` migrated to `[[relation]]` (SL-095) +
  verb landed (SL-062), but prose says typed/inert/pending-IMP-006. *Fix: correct
  one §.* High conf.
- **0001** — ISS-025 / ISS-027 are the same defect captured twice. *Fix: merge/dispose
  one.* High conf.
- **0002** — SPEC-003 cites 3 of 11 container specs; back-fill 6 existing refs. High conf.

## Tier B — timely / time-sensitive (decide now, while the window is open)
- **0018** — SL-121 (active, in `design`) and IMP-075 rewrite the *same* `integrate`
  body; fold or sequence **before SL-121's plan locks**. Decay if deferred.
- **0008** — IMP-069/070/097 are one spec-altitude workstream; consolidate before
  three parallel level-validators get built independently.

## Tier C — hygiene / governance gaps, cheap-to-medium
- **0004** — widen IMP-067 to a corpus-wide `entity::id_path` (~24 inline sites).
- **0005** — SPEC-018 (relation contract) has 0 requirement members; author the spine.
- **0009** — authored-priority slot designed (D10) + REQ-054 minted, unbuilt + untracked.
- **0016** — PRD-005 advertises a shared-remote lease backend SPEC-008 calls unbuilt.
- **0019** — ~52% of requirements have empty inline `acceptance_criteria`; settle the
  field's contract (optional vs expected).
- **0020** — 4 skills (code-review/handover/dreaming/reviewing-memory) absent from the
  boot routing snapshot; generalises IMP-042.
- **0010** — ADR-001 layering ratchet has no burn-down; coverage→requirement wart =
  60% reduction via extract-types-to-leaf.
- **0015** — IMP-056 sharpened: shared formatter touches a tested/persisted register
  (scope decision report-only vs register-wide).

## Tier D — strategic / build (the indispensable-to-teams bets; need design)
- **0003** — transitive impact/blast-radius query (gate on IMP-020 first). Flagship.
- **0006** — bridge concept maps ↔ relation graph (validate + drift-detect).
- **0011** — unified `doctrine doctor` health gate (aggregate 4 scattered checks).
- **0007** — graph interchange export (DOT/GraphML/Cypher), reuse `/api/graph`.
- **0017** — `worktree gc --all` oracle-gated bulk sweep (77 worktrees accreting).

## Lens (read for *why*, not *what*)
- **0014** — thesis: the graph's ROI is gated on consumption surfaces (0003/0006/
  0007/0009/0011), not more internal modelling. Use to decide Tier D ordering.

## Dependencies & cross-refs (don't action out of order)
- **IMP-020** (cordage `reachable` triplication) gates **0003**; do it first.
- **0008** (build IMP-069 level-adjacency validation) would *catch* **0012**'s class —
  pair them; extend the gate to assert *presence* of `parent`, not just rank.
- **0011** (doctor) is the natural home for advisory counters proposed in **0019**
  (empty-AC count) and **0010** (layering baseline size).
- **0018** is the only item with a hard clock (SL-121 in design *now*).

## Recommendation
Clear **Tier A** in one sitting (four small, certain corrections), then make the two
**Tier B** sequencing calls before SL-121 advances. Tier C as backlog grooming. Treat
**Tier D + 0014** as one planning conversation about the next theme — start with
**0011** (cheapest team win) and **0003** (flagship, after IMP-020).

Decision deferred to YOU: the tiering reflects my read of effort×value×confidence —
override any placement; this is a reading order, not a mandate.

## Next doctrine move
```
# fastest path through the queue:
sed -n '1,40p' proposals/0012-*.md proposals/0013-*.md proposals/0001-*.md   # Tier A
sed -n '1,40p' proposals/0018-*.md                                           # Tier B clock
# then groom Tier C and schedule the Tier-D planning conversation.
```
(No verb to run here — this orders the other proposals' verbs.)
