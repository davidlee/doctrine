# IMP-414: `knowledge show` conceals an unfilled facet; `facet_json` does not

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`IMP-403` lead 4, minted at `SL-249`'s close. `SL-249` closed leads 1 and 2 and
named 3–5 as follow-ups rather than dropping them.

## What

The two render paths over the same facet disagree, and the human-facing one is
the concealing one.

- `format_facet` (`src/knowledge.rs`) emits the `[facet]` header only when at
  least one axis is populated, and `show_opt_line` drops absent fields
  silently. An unfilled record renders as **nothing at all** — not a blank
  block, not a header.
- `facet_json` emits every field, as `null`.

So the surface a human reads hides exactly the gap `IMP-403` measured, while
the machine surface reports it faithfully. Line numbers deliberately omitted —
`SL-249` moved this code; find the two functions by name.

## Why it is not just cosmetic

Concealment is causal, not incidental. An author who runs `knowledge show` on a
hollow record sees a clean-looking entity and has no signal that seven fields
are missing. `IMP-403`'s adjacent analysis reached the same conclusion from the
other direction: the two-file split is not *causing* the empty facets, it is
making them **invisible**.

## Constraint

`DEC-149` already rules on how an unfilled facet is marked, and `SL-246` owns
the *composed* read. This item is `knowledge show`'s own render, not that
composition — check both before changing the shape, or the two will diverge
again.

## Related

`IMP-403` (parent), `IMP-413` and `IMP-415` (leads 3 and 5), `SL-246`,
`DEC-149`.
