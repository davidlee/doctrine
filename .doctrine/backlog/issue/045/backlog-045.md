# ISS-045: Web view inverts dependency arrows for `needs` relations

## Reproduction

IMP-120 `needs: IMP-020` — open the web view and observe the arrow direction
between the two nodes.

- **Expected:** arrow points from IMP-120 → IMP-020 (IMP-120 depends on
  IMP-020).
- **Actual:** arrow is inverted, pointing from IMP-020 → IMP-120.

The underlying graph data is correct (CLI inspection shows the right
direction). Only the web rendering is affected.

## Suspect

The D3/force-graph edge-rendering code likely swaps source/target when
binding edges, or the coordinate flip happens during layout. Since other
edge types render correctly, this may be specific to `needs` edges (or all
typed dependency edges that share a code path distinct from the relation
overlay edges).

## Scope

- Fix the arrow direction in the web view for `needs` (and verify `after`
  edges are also correct).
- Add a visual regression safeguard if one doesn't exist (e.g. a known-node
  fixture where edge direction is asserted in the DOM).

## Investigation notes (2026-06-22)

- The actionability graph (`/api/survey`) encodes `needs` edges in **blocker →
  blocked** direction. For IMP-120 `needs: IMP-020`, the edge is `IMP-020 →
  IMP-120`. This is intentional in the priority/survey system — the arrow shows
  "what blocks what." The web view renders the survey data as-is.
- The perceived "inversion" is a semantic mismatch: dependency arrows
  (depends-on → dependent) point the opposite way from blocker arrows (blocker
  → blocked). The fix may be a label/doc clarification in the web view rather
  than a coordinate swap.
