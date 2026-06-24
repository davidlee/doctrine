# ISS-050: backlog show and knowledge show don't render .md prose body — metadata-only inspect masquerading as show

## Context

All twelve `doctrine <entity> show` verbs are the canonical way to reassemble and
display an entity's full record — structured metadata AND prose body. Ten of
them do; two don't.

`backlog show IMP-139` outputs the identity header, status, tags, facet, and
relationships — but never touches `backlog-139.md` (25 lines of prose on disk,
silently invisible). Same for `knowledge show`.

These two `show` verbs act like `inspect` — metadata only. The body sits
unrendered in the output.

## Affected

- `backlog show` — `BacklogItem` struct has no `body` field, `format_show` never
  reads the `.md`.
- `knowledge show` — same pattern: `KnowledgeRecord` has no `body` field,
  `format_show` stops at metadata+facet+evidence+relations.

## Discovery

2026-06-24: systematic audit of all twelve `show` verbs against their sibling
`.md` existence. All others (adr, policy, standard, rfc, revision, rec, slice,
spec, review, memory, concept-map) read and render body.

## Expected

Both verbs should read and render the prose `.md` body inline, below metadata
and above/after relationships, matching the house style of the other ten.
