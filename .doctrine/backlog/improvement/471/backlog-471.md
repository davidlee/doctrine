# IMP-471: Govern the inquiry map's dynamic mode at requirement tier

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Nothing at requirement tier says the map is **dynamic**. `PRD-019` `REQ-417` fixes
its shape only — stable identity, one primary parent, sparse dependency references
— and `SPEC-029`'s requirements (`REQ-428`..`REQ-438`, plus the active `REQ-478`)
cover snapshot versioning, revision CAS, idempotent retry, reserve-then-journal,
adoption, the envelope, the watermark, the prompt pack, purity, projection bounds,
no-delete and change-log emission. Not one mentions adding a node, adding or
editing an edge, or graph evolution.

The nearest governing sentence is `DEC-061`'s "conditional discoveries add nodes
when they become concrete rather than pre-authoring an activation language" — a
permission, not a described behaviour. That a map should start sparse, grow as
inquiry proceeds, and be re-edged as dependencies are revealed is asserted only in
tier-5 deliberation (`RFC-030`, `RFC-026` E8.5) and in the mechanics of `DEC-063`.

## Why it matters

Every candidate fix in this area has to guess at the intent, because no requirement
states it. `IMP-469` is the sharpest example: whether early sufficiency acceptance
is meant to hold across discovery, or to be re-earned, is a question about intended
dynamics — and it is currently answered only by accident, through a `reach` column.

## Relationship to QUE-218

Gated on `QUE-218` (*does the inquiry map earn a semantic tier?*): specifying the
dynamic mode presupposes the map is worth specifying. `RFC-030` is open too, and
its thesis may reshape the map before anything is written as canon. Recording the
gap is cheap now; acting on it waits for both.

## Shape

Once `QUE-218` settles: a requirement that a run's map begins sparse and may add
nodes and edges as inquiry proceeds — together with the half that is dark today,
namely what re-derivation an *edited* edge demands. `DEC-063` permits redeclaring a
node to change `parent` or `needs`, but no artefact defines the consequences for
the derived blocked set, frontier eligibility or `needs_in_degree` ranking.

## References

- `PRD-019` `REQ-417`; `SPEC-029` `REQ-428`..`REQ-438`, `REQ-478`
- `DEC-061`, `DEC-063`; `QUE-218`
- `RFC-030`, `RFC-031`; `IMP-469`
