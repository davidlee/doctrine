# IMP-149: Ambiguous `slices` relation kind on backlog items

## Problem

The `slices` relation kind on backlog items is semantically ambiguous. It can
mean either:

- "the slice that **resolves** this backlog item" (the item is scoped into
  that slice, and closing the slice resolves the item), or
- "the slice that **spawned** this item" (the item was created as a follow-up
  during slice design/audit/reconcile).

This predates several of the more semantically clear edge types now available
(`governed_by`, `scoped_into`, `spawned_by`, etc.). The graph renders it as a
single unqualified edge label, which forces the reader to infer direction from
context.

## Desired outcome

A design decision on whether to:

- Split `slices` into two distinct relation labels with clear semantics, or
- Keep `slices` but add a direction qualifier (e.g. `slices:resolves` /
  `slices:spawned`), or
- Deprecate `slices` entirely in favour of the newer edge vocabulary.

## Scope

- Inventory all extant `slices` edges and classify their actual semantics.
- Propose a design (likely via a dedicated slice or as part of a broader
  relation-model cleanup).
- Migration plan if the edge kind is split or renamed.

## References

- ADR-004 (relations outbound-only — reverse edges are derived, never stored)
- SL-048 ("the cut" — `[[relation]]` rows)

## Investigation notes (2026-06-22)

- **`needs` works for backlog-item → slice edges.** IMP-120 `needs: SL-138` was
  added via direct TOML edit; the actionability graph correctly renders
  `SL-138 → IMP-120 kind=needs` (blocker → blocked direction).
- **`doctrine backlog needs` CLI rejects SL-prefixed targets** with "unknown
  backlog prefix `SL`". The CLI validation only allows ISS/IMP/CHR/RSK/IDE
  prereqs, even though the data model and graph pipeline accept cross-kind
  `needs` edges. This is either a CLI bug (needs validation too narrow) or a
  deliberate constraint that should be documented.
- **Auto-cleanup drops `needs` edges when target is absent**, then logs as an
  override (e.g. `IMP-120 → SL-138 dropped (dangling: SL-138 absent)`). The edge
  must be manually re-added if the target reappears.
