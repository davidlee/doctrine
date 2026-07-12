# IMP-288: Tension callout: anchor-determined zero-judgement wording

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by the SL-218 close-out audit (RV-272 F-8; codex VA-1 #5).

A tension callout graded `Determined` whose constraining basis is an anchor
(point constraint) rather than judgement rows carries `RaterCounts (0,0)`, so
`tension_counts_clause` (`src/priority/render.rs:456`) emits:

> tension: … ranks above … on value_dim (determined — no constraining
> judgements); …

Technically accurate — anchors determine the region, they are simply not
judgement rows (D6: "anchors are point constraints") — but "determined — no
constraining judgements" reads self-undermining.

Rare: requires an anchor-determined pair with zero judgement rows *and* a
surviving value/delivery-order inversion; not exercised by current fixtures,
so no golden pins it.

Fix sketch: when `Determined` counts are `(0,0)`, render an anchor-aware clause
(e.g. `determined — anchor-constrained`) instead of the empty-judgements phrase.
Small render change + a unit golden. No behaviour/semantics change.

