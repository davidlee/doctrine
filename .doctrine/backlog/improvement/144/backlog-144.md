# IMP-144: Tag read-surface wiring for concept-map/review/REC/revision, then extend taggable set

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-136 ships a generic `doctrine tag` verb. Its taggable set is curated to kinds
whose **read surfaces render tags** — slice, governance (ADR/POL/STD/RFC),
backlog, knowledge, spec, REQ. Concept-map (CM), review (RV), REC, and revision
(REV) were **deliberately excluded**: their `show`/`--json`/`list` paths do not
render tags, so allowing writes would create write-only (silently vanishing)
metadata (Codex MAJOR, SL-136 design §7 D2).

## The work

Per excluded kind: wire the tag read surfaces — `key()` populating
`FilterFields.tags` for `list --tag`, and the `show`/JSON render of the tag axis
— then add the prefix to SL-136's taggable set. With the surfaces wired the
generic verb already handles them (uniform root storage, no per-kind write code).

Low priority — these kinds are rarely classified. Sequence after SL-136 closes.
