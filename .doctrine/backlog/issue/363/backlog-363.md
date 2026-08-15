# ISS-363: rfc show omits authored references relations

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Two `references --role concerns` rows authored on `RFC-026` with `doctrine link`
are stored correctly in `rfc-026.toml` and render under `doctrine inspect
RFC-026`. `doctrine rfc show RFC-026` prints only `related: RFC-027`.

## Why it matters

The guardrails are explicit: **read entities via `doctrine <kind> show <ID>`,
never raw files**, because `show` synthesizes the TOML and MD tiers. An agent
following that rule sees an incomplete relation set on an RFC and would
reasonably conclude the edges were never authored — and then author them again.

The scaffold reinforces the wrong model. `rfc-NNN.toml` carries the comment:

> RFCs participate in the tier-1 `related` relation (AnyNumbered — links to any
> entity). Author with `doctrine link RFC-NNN related <target>`.

which names only `related`, though `link` accepts, validates and stores
`references` on this kind without complaint.

## Fix

One of two, and the choice is a real decision rather than a coin flip:

1. **`show` renders every authored relation row**, as `inspect` does. Preferred —
   a read surface that silently drops authored state is the defect, and any kind
   could grow the same gap.
2. **The kind refuses the labels it will not display.** Narrower, but it makes
   `link`'s acceptance a lie about what the entity supports.

Either way the scaffold comment needs correcting to match.

Worth checking the other kinds' `show` implementations for the same gap while the
question is in hand — this is likely a per-kind renderer omission rather than an
RFC-specific one.

## Evidence

- obs `01a00493-903d-71b0-b0ae-9b9a209eeab2`
- `RFC-026` at `2026-08-15` — `inspect` shows two `references(concerns)` rows,
  `rfc show` shows none.

## References

- `ADR-004` — relations stored outbound-only; reciprocity is derived
- `ADR-010` — relation modelling: unify the contract and write seam
- `mem.concept.doctrine.reading-entities` — the read-via-`show` rule this breaks
