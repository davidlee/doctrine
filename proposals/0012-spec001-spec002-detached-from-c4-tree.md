---
seq: 0012
scope: spec
target: SPEC-001, SPEC-002
confidence: high
reversible: yes (proposal only; no authored-tier edit — fence holds)
---
## What
Two tech specs are **detached from the C4 containment tree**: they carry
`c4_level` and `descends_from` but have **no `parent` edge** (only the commented-out
template line `# parent = "SPEC-002"` survives in their toml).

- `SPEC-001` (Graph-Derived Priority Engine) — `c4_level = "container"`,
  `descends_from = "PRD-011"`, **no `parent`** (`.doctrine/spec/tech/001/spec-001.toml`).
  Every other container parents to the context root: SPEC-004/006/007/008/009/010/
  011/012/013 all have `parent = "SPEC-003"`. SPEC-001 should too.
- `SPEC-002` (Requirement Reconciliation Engine) — `c4_level = "component"`,
  `descends_from = "PRD-013"`, **no `parent`** (`.doctrine/spec/tech/002/spec-002.toml`).
  Every other component parents to a container (→ SPEC-004 / SPEC-006 / SPEC-012).
  SPEC-002 should parent to whichever container owns it.

(SPEC-003 itself is the context root — legitimately parentless. These two are the
only non-root specs missing the edge; they are also the lowest-numbered = earliest
authored, the same staleness pattern proposal 0002 found in SPEC-003's prose.)

Consequences — a real topology hole, not a cosmetic gap:
- **Disconnected subtree.** SPEC-018 (Cross-corpus relation contract) has
  `parent = "SPEC-001"`; SPEC-001 has no parent. So the whole priority/graph subtree
  (SPEC-001 ← SPEC-018) dead-ends and is **unreachable top-down from the SPEC-003
  context root** via `parent` edges. Likewise SPEC-002 (and anything under it).
- **Validation blind spot.** IMP-069 (level-adjacency validation, see proposal
  0008's cluster) can only check edges that exist — a *missing* parent passes
  silently. These two are live examples of exactly why that validation is needed:
  the gate that would catch them isn't built, and the edges it would check are
  absent.
- The graph asymmetry reads as intentional ("these specs have no container") when it
  is just unbackfilled — the inverse failure mode of an explicit decision.

## Options
1. **Backfill both parent edges.** `SPEC-001.parent = SPEC-003` (unambiguous —
   matches all sibling containers); `SPEC-002.parent = <owning container>` (needs a
   one-time judgement, see deferred-(b)). Tradeoff: smallest corrective edit,
   restores top-down reachability; the only open question is SPEC-002's container.
2. **Backfill SPEC-001 only now; resolve SPEC-002 separately.** SPEC-001→SPEC-003 is
   certain; SPEC-002's container is a modelling call. Tradeoff: ships the certain
   half immediately, doesn't block on the judgement; leaves one spec detached
   meanwhile.
3. **Leave as-is.** Tradeoff: zero effort; but two core specs (priority,
   reconciliation — both central to the product story) stay unreachable from the
   context root, and the hole will keep masking IMP-069's value.

## Recommendation
Option 1 if you can name SPEC-002's owning container now; otherwise Option 2
(ship SPEC-001→SPEC-003 immediately, queue SPEC-002). Rationale: SPEC-001→SPEC-003
is a zero-judgement correction that matches nine sibling containers and reconnects
the entire priority/graph subtree (incl. SPEC-018) to the root — high value, no
risk. SPEC-002 needs one decision (which container) and shouldn't hold up the
certain fix. Pair this with proposal 0008 (build IMP-069 level-adjacency validation)
so the gate exists to prevent recurrence — and extend that gate to flag
**missing** parents on non-root, non-context specs, not just wrong-rank ones.

Decisions deferred to YOU:
- (a) confirm `SPEC-001.parent = SPEC-003` (container → context, matching siblings).
- (b) **SPEC-002's owning container** — Requirement Reconciliation Engine sits under
  which container? (SPEC-001 priority/graph? the entity engine SPEC-004? a
  reconciliation container if one is intended?) This is the only modelling call.
- (c) should IMP-069's validation also assert *presence* of a parent on every
  non-context spec (catching this class), not only rank-adjacency?

## Next doctrine move
```
# confirm the missing edges (read-only):
doctrine spec show SPEC-001        # note: no parent; siblings all → SPEC-003
doctrine spec show SPEC-002        # note: no parent
grep -L 'parent =' .doctrine/spec/tech/*/spec-*.toml   # specs missing the key

# corrective edit is authored-tier — route it (NOT executed; fence forbids):
/route             # → small slice or boot.md-Governance "small backlog item"
                   #   quick-design edit adding the two `parent =` keys.
```
(Verbs described, NOT executed — fence forbids editing authored spec state.)

## Illustration (optional) — ILLUSTRATIVE, not applied
Hand-authored (no worker), the certain half (SPEC-001):
```diff
--- a/.doctrine/spec/tech/001/spec-001.toml
+++ b/.doctrine/spec/tech/001/spec-001.toml
 c4_level = "container"
 descends_from = "PRD-011"
+parent = "SPEC-003"
```
