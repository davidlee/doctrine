# ISS-237: web map DOT emitter uses invalid shape box,rounded

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`web/map/src/dot.ts` NODE_STYLES uses `shape: 'box,rounded'` for SL/PRD/SPEC
and graphToDot writes it directly as the DOT `shape` attribute. Graphviz
reports `warning: using box for unknown shape box,rounded` and the rounded
styling is silently not applied. Correct form: `shape="box"` +
`style="filled,rounded"`. Surfaced by RV-298 F-10 (SL-226 design review);
the Rust CLI emitter models shape/style separately from the start (SL-226
D10) — fixing the web table restores intended rounding and kills the dot
stderr warning in the svg render path.
