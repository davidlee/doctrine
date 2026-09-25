# IMP-486: Typed MapNode lifecycle and provenance

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

From RV-393 `F-3` (SL-266 pre-close code review). `MapNode.lifecycle` and
`.provenance` in `src/design_run/render/envelope.rs` are `&'static str`, as the
SL-266 design (sec-2) fixed them. The tree renderer (`render/tree.rs`) parses
them back: `State::of` returns `Option`, and `node_lines` / `letter` fall back to
the raw string, which is unreachable because `map_nodes` only emits
closed-vocabulary tokens.

Carry `InquiryLifecycle` / `Provenance` on `MapNode` instead, serialised to the
same tokens, so the envelope's JSON is unchanged. That deletes the `Option`, both
fallback branches and the provenance-label table lookup, and a new lifecycle
becomes a compile error in the renderer rather than a raw-string mark.

Already done in SL-266: `State::word` derives the lifecycle words from
`InquiryLifecycle::as_str`, so they are spelled in one place (STD-001).
