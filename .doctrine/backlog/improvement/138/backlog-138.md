# IMP-138: Relation-transitive walk for inspect

## Source

IMP-133 UX review, second pass (RF-3).
See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem

`blockers --transitive` walks dep edges transitively. There is no
equivalent for relation edges. You can't ask:

- "Show me everything shaped by ASM-001, transitively"
- "Show all specs that descend from PRD-001, transitively"
- "Show the full governed_by closure for SL-047"

The cordage graph already builds with relation overlays — the walk
machinery exists. This is a flag on `inspect` (analogous to `blockers
--transitive`), reusing the same `reachable` walk.

## Proposed shape

```bash
doctrine inspect SL-047 --transitive --label governed_by
# Show SL-047 and everything it (transitively) governs

doctrine inspect PRD-001 --transitive --label descends_from
# Show the full descends_from tree rooted at PRD-001
```

MVP: `--transitive` on inspect, scoped to one label (or all labels if
omitted). Depth limit to prevent explosion on dense graphs.
