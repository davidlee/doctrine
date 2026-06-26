# ASM-001: Role machinery extends to shapes — ADR-016 roles apply to shapes label

A load-bearing assumption for the M-Pr option in QUE-001. M-Pr proposes a closed
role dimension `{gates, informs}` on the `shapes` RelationLabel, riding SL-149's
role machinery. But roles currently only refine `references` — this would be the
first extension of the role grammar to a second label.

## What's assumed

ADR-016's closed-role contract (per-edge role assertion via `link --role`,
dimension-wide validation via `Role::from_name`, consumer-side projection over the
role subset) is label-agnostic — the machinery was built once and can be reapplied
to `shapes` without grammar changes.

## Risk

If the `shapes` label has constraints (source-kinds, target-kinds, tier placement)
that don't compose cleanly with a role dimension, or if the `link --role` surface
needs per-label adaptation, M-Pr collapses. In that case, M-E (distinct `gates`
axis) becomes the only surviving option.

## Validation

- Check ADR-016 / SL-149 design for label-agnosticism in the role contract
- Attempt a prototype `link ASM-001 shapes --role gates RFC-008` after SL-149 lands
- Verify the consumer projection (gating overlay) composes cleanly over the role subset

## References

- ADR-016 — closed role dimension
- SL-149 — role machinery implementation
- QUE-001 — gated by this assumption (M-Pr path)
- RFC-008 § Mechanism options, M-Pr
