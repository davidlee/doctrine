# Doctrine ADR authoring

Architecture Decision Records (ADRs) capture consequential design decisions
that have project-wide scope and durability beyond a single slice.

## When to author an ADR

An ADR is warranted when a decision:
- Affects multiple slices or subsystems.
- Carries architectural weight (data model, protocol, governance rule).
- Will constrain future work and needs a durable record.
- Has tradeoffs worth documenting independently of any one slice's design.

For slice-scoped decisions, use `design.md` within the slice. For standing
rules, use policies or standards.

## Lifecycle

- `proposed` — drafted, awaiting review.
- `accepted` — in force. Referenced by slices and governance.
- `superseded` — replaced by a newer ADR via `doctrine supersede <ADR-NEW>
  <ADR-OLD>`. The old ADR carries a `superseded_by` pointer; the new one carries
  `supersedes`.

## CLI

See `doctrine adr --help` for authoring commands and `doctrine supersede
--help` for the supersede lifecycle.
Key verbs: new (title is positional), list, show, status.

ADRs are project-global — they live under `.doctrine/adr/nnn/` as
`adr-nnn.{toml,md}` pairs. Status lives in the TOML, rationale in the MD.

See [[concept.doctrine.entity-engine]] for the entity model,
[[signpost.doctrine.rfc]] for RFC governance,
[[signpost.doctrine.file-map]] for the directory layout,
and [[signpost.doctrine.policies-standards]] for governance standing rules.
