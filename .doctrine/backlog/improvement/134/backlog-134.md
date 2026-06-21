# IMP-134: Extend tagging to all appropriate entity types — model + CLI

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Current state

`doctrine backlog tag` exists for backlog items (ISS/IMP/CHR/RSK/IDE). Tags are
a powerful cross-cutting classification axis — used in IMP-118's scoring
(`[priority.tag_coefficients]`) and for filtering/grouping in list commands.

Several entity types already carry a `tags` field in their TOML schema but have
NO CLI verb to set/clear them:

| Type | TOML has `tags`? | CLI verb?
|------|------------------|----------
| ADR  | ✅ (seeded `[]`) | ❌
| POL  | ✅ (seeded `[]`) | ❌
| STD  | likely same model | ❌
| RFC  | ✅ (seeded `[]`) | ❌
| SPEC | ✅ (spec-001 uses them) | ❌
| SL   | ❌ (model has it in lazyspec, scaffold missing) | ❌
| knowledge | ✅ (model) | ❌

## Scope

### Modelling
- Add `tags = []` to the slice TOML scaffold (align model with lazyspec)
- Confirm ADR/POL/STD/RFC/SPEC/knowledge all have tags in their schema

### CLI
- Add a generic tag verb usable across kinds, OR per-kind tag subcommands
  (`doctrine adr tag`, `doctrine spec tag`)
- Reuse the existing validation/add/remove/display logic from `doctrine backlog tag`
- Pure/impure split: validation pure, disk read/write impure (same seam as backlog)

### Surface
- Tags should be visible in list/show output for the tagged type
- Tag filtering on list commands (`--tag`) where supported

## Connections to graph ordering

Tags are a direct input to IMP-118's priority scoring (`[priority.tag_coefficients]`),
which feeds `survey`/`next` ordering. But the current `survey`/`next` only considers
backlog items + slices in its eligibility model. Extending tags to all entity types
(specs, ADRs, RFCs, etc.) is a prerequisite for extending the actionability graph to
cover them — because tags are how you'd filter, weight, and group non-backlog entities
in the priority order.

Currently the graph ordering (consequence tally, mint order, dep/seq levels) is
backlog-and-slice only. Broader tagging is a modelling precondition for broadening
the priority surface.

## Links

- Surfaced by IMP-118 dependency analysis (tag coefficients need tags on specs, slices, etc.)
- Precedent: `doctrine backlog tag` (src/backlog.rs:apply_tags, apply_tags_quiet)
- IMP-118 — multi-dimensional priority scoring consumes tag coefficients
