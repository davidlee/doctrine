# CHR-171: SPEC-007 and REQ-378 still say memory find

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`SPEC-007` (memory system tech spec) and `REQ-378` still name the verb by its
pre-`SL-184` name, `memory find`. `spec validate` cannot catch this. Surfaced by
`SL-275` research (design-input delta 8); out of scope there. Fix via a Revision
if the wording is normative. Consider also recording `DEC-347`/`DEC-348`
(search page size, free-text retrieval floor) in `SPEC-007` at the same time,
since the spec is silent on paging and floors.
