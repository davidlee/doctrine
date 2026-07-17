# ISS-229: Shipped memory mem.signpost.doctrine.knowledge stale: six kinds, wrong default-status vocab

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed (SL-214 audit, RV-280; first noted in SL-214 notes.md)

Shipped memory `mem.signpost.doctrine.knowledge` still describes six knowledge
kinds and the wrong default-status vocabulary (source seeds ASM=held,
DEC=proposed; SL-159 made it seven kinds ASM/DEC/QUE/CON/EVD/HYP/CPT). Shipped
corpus fix: edit `memory/`, `cargo build` (re-embed), `doctrine memory sync`,
`doctrine install`.
