# IMP-035: Slice relationships schema has no ADR slot: slice to ADR refs are prose-only

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Gap

A slice's `[relationships]` block is `{specs, requirements, supersedes}` — there is
**no slot for an ADR (or any governance-kind) ref**. A slice governed by an ADR can
only cite it in prose; the link is invisible to the relation graph (SL-046) and to
any future `link` writer (SL-048).

Surfaced during the SL-046 ↔ ADR-010 interrogation: SL-046's design wanted to
record ADR-010 as the governing decision but could not author the edge — the
absence *is* the cross-corpus relation gap.

## Scope

Add a structural slice→ADR (more broadly slice→governance) relation so the edge is
authorable, validated, and read by the SL-046 graph — symmetric with SL-048's
spec↔ADR objective. Likely the same `link`-style verb + per-source-kind legal-set
table ADR-010 D2 specifies; the slice source-kind row gains an ADR-target label.

## Owner

**Realised by SL-048** (`slices = ["SL-048"]`) — the structural cross-corpus edge
slice. This is the slice-source analogue of SL-048's existing spec↔ADR objective,
not a separate effort. Governed by **ADR-010** (relation contract + write seam).
