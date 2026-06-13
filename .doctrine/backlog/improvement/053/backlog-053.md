# IMP-053: Record↔record associative relation class (informs/bears-on) for SPEC-019 Slice B

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during SL-059 `/design`. SPEC-019's pinned record relation labels (D6)
cover record→other-corpus (`specs`/`slices`/`requirements`/`drift`/`governed_by`),
record→backlog-item (relate), record→spawned-work (`spawns`), and same-family
`supersedes` (replacement, not association). **No typed record↔record *associative*
edge exists** — e.g. QUE↔ASM ("the assumption I hold about this question"), ASM→DEC
("this belief shaped this decision"). Today these are expressible only as free-text
`evidence` citations (dangle, not edges) or `.md` prose.

Need: mint a new `RelationLabel` (e.g. `bears-on`/`informs`/`relates`) targeting the
four record kinds — an A→B authored edge, outbound-only, reverse derived (ADR-004).
This is a **SPEC-019 amendment** (the label class is spec-pinned in D6) feeding the
Slice B relation seam (FR-005). Decide direction semantics (symmetric vs directed)
and whether one label or several at amendment time.
