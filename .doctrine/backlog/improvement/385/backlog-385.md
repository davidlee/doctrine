# IMP-385: Diagram rendering for the spec anchor report

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A graph rendering for `doctrine spec anchors` — DOT, matching the existing
`catalog/dot.rs` emitter rather than adding d2. SL-243 ships `json` and
`markdown` only (DEC-115).

## Why it waits

The whole-corpus view is not readable: 48 specs against 84 dark files is a
hairball in any renderer, which is SL-243's own reason for deferring the
focus-scoped altitude view. A diagram becomes worth rendering when there is
something page-sized to render — so this is naturally the same piece of work as
that view, or arrives just after it.

## Cost when taken

Small by construction. The report core is presentation-neutral (DEC-112, and
the SPEC-027 shape SL-243 mirrors), so this is a format enum variant plus a
renderer. No rework of the core.

DOT over d2: `catalog/dot.rs` already emits deterministically — sorted on
(source, display label, target) with the original list index as final tiebreak —
and declares itself the single source for `dot_escape`. A d2 emitter would share
none of that, since d2's escaping rules are its own. AGENTS.md's stated diagram
preference is mermaid, so d2 is not a house default either.

## Related

- DEC-115 — the SL-243 decision deferring this, and the scope trim it caused.
- IDE-046 — rendering DOT to an image and emitting it inline in the terminal;
  the reason DOT is the more interesting target of the two.
- SPEC-027 — the graph projection whose emitter this follows.
