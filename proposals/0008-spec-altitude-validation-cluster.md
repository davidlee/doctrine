---
seq: 0008
scope: backlog
target: IMP-069, IMP-070, IMP-097 (cluster)
confidence: high
reversible: yes (proposal only; no consolidation/transition performed — yours to run)
---
## What
Three open `improvement` items, all body-less (no prose tier), describe one
coherent workstream: **altitude / C4-level validation over the spec descent
graph.** None references the others; created within days; no sequencing edge
between them.

- **IMP-069** (`area:spec, correctness`, created 06-14) — "Level-adjacency validate
  tightening: parent **exactly one** product/C4 rank above child." → a rule on the
  `parent` edge.
- **IMP-070** (`area:spec`, created 06-14) — "`descends_from` to capability-level
  PRD constraint — couple tech descent target to product level axis." → a rule on
  the `descends_from` edge.
- **IMP-097** (created 06-18) — "Altitude assessment framework for requirement
  placement (product vs tech, C4-level rules)." → the same level model applied to
  *requirement* placement.

These are the **same axis** (C4 / product-vs-tech altitude) applied to three edges
of the lineage graph (`parent`, `descends_from`, requirement membership). And the
validation they call for **does not exist yet**: `descends_from`/`parent` are real
edges (`src/relation.rs:111,302`; `src/lazyspec.rs:125`), carried as scalar
lineage, but a grep for level-adjacency / rank-above / altitude *checking* logic in
`spec.rs`/`lazyspec.rs`/`relation.rs` finds only the data and the relation mapping —
no rank rule. So all three are greenfield validators over the same graph.

Implemented as three separate items, they invite **parallel implementation**: three
independent "what C4 level is this entity / is the parent one rank up" checks, each
re-deriving the level lattice, likely diverging (069 says "exactly one rank"; 070
says "capability-level"; 097 says "product vs tech" — these must agree on one level
model or they contradict). That is exactly the anti-pattern the project forbids
("no parallel implementation — ride existing seams").

## Options
1. **Consolidate into one slice** (close 069/070/097 as folded-in, or keep them as
   the slice's acceptance bullets). One C4-level lattice + one rank-adjacency
   predicate, applied to `parent`, `descends_from`, and requirement placement.
   Tradeoff: settles the level model once, kills the divergence/parallel-impl risk;
   cost is a larger single scope and the bookkeeping of folding three items.
2. **Keep three items, add explicit sequencing + a shared-primitive note.** Leave
   them separate but record that 069 first defines the level lattice + adjacency
   predicate, and 070/097 consume it (`after` edges). Tradeoff: smaller increments,
   preserves the existing ids; relies on whoever picks them up honouring the shared
   primitive rather than re-rolling it.
3. **Leave as-is.** Tradeoff: zero effort, but three body-less same-axis items will
   likely be grabbed independently and grow three level-checkers.

## Recommendation
Option 1 — consolidate into one slice ("spec/requirement altitude validation"),
because the three items share a single load-bearing decision (**the C4-level
lattice and what "one rank above" means across product vs tech**) that cannot be
made three times consistently. The lineage edges already exist; what's missing is
one authoritative level model + predicate, and that is a single design question.
If you prefer to preserve increment granularity, Option 2 is acceptable *provided*
069 is explicitly the primitive-definer the others depend on.

Decisions deferred to YOU:
- (a) **consolidate (1) vs sequence (2)** — one slice, or three items with `after`
  edges and a shared-primitive contract.
- (b) **does requirement-placement (IMP-097) belong with the spec-edge two**, or is
  requirement altitude a separable concern (different entity, possibly different
  lattice)? It shares the level model but acts on a different edge.
- (c) **the level model itself** — is "parent exactly one C4 rank above child" the
  invariant, or are skip-level edges legal in some cases (e.g. a component spec
  descending directly from a context PRD)? This is the design crux 069 raises.

## Next doctrine move
```
# read all three together to confirm the shared axis (read-only):
doctrine backlog show IMP-069
doctrine backlog show IMP-070
doctrine backlog show IMP-097

# EITHER consolidate (option 1) — scope one slice, fold the three in:
/route                 # → /slice  (then reference IMP-069/070/097 as the scope)
# ... and on close, transition the folded items (NOT executed here — fence):
#   doctrine backlog <resolve/close verb> IMP-070 IMP-097  (per `backlog --help`)
# OR sequence (option 2) — add after-edges so 070/097 follow 069:
#   doctrine backlog <relate/after verb> IMP-070 --after IMP-069   (verb per --help)
```
(Verbs described, NOT executed — fence forbids backlog transition / relation writes.)

## Illustration (optional)
None — a triage/consolidation proposal. The substance is the cluster recognition
and the single-level-model decision, not a diff.
