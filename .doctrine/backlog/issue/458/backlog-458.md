# ISS-458: Web map DOT emitter labels nodes id-only, diverging from the CLI

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

IMP-454 changed the Rust DOT emitter (`src/catalog/dot.rs`) so each node reads
as a bolded handle over its wrapped title. The web map has its own emitter —
`web/map/src/dot.ts::graphToDot`, of which the Rust file is a port — and still
labels nodes with the bare canonical id.

So the CLI and the browser view now disagree about what a node says. The port
relationship is documented in the Rust module header, which makes the drift
easy to miss: nothing enforces it.

Either bring `dot.ts` across (title wrapping, monospace, HTML-like labels with
the bolded handle), or record that the two surfaces intentionally differ.
