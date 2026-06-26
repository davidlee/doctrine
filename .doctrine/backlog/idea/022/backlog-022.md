# IDE-022: shapes-roles: disambiguate epistemic record-record vs affects record-work on the shapes relation

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Split out of SL-158 (`/design`, 2026-06-26). ADR-017 §Neutral: *"`shapes` roles
remain a separate, smaller piece (semantic disambiguation), decoupled from gating,
ridable on ADR-016."* SL-158 kept the **actionability** half (trinary partition +
`needs`-target gate widening); this is the **semantic** half it shed.

`shapes` is the record's outbound "what I affect" surface (PRD-010 §3), graph-inert.
It targets a wide set (PRD/SPEC/REQ/SL/ISS/IMP/CHR/RSK/IDE/ADR/POL/STD + the 4 record
kinds). Two semantically distinct uses ride the one label:

- **epistemic** record→record (e.g. an ASM shaping a QUE) — intra-family.
- **affects** record→work (e.g. a CON shaping a SL/REQ) — cross-family.

## The open question (do they earn their keep?)

Should `shapes` carry a **role** (a closed role dimension per ADR-016) to
disambiguate epistemic-vs-affects — or is that plane **derivable from the target
kind** (record-target ⇒ epistemic; work/governance-target ⇒ affects), making a
stored role redundant? Decide before adding vocabulary: a derivable distinction
should not be stored (RFC-003 design law: *derivable, not relational*).

## Scope sketch (if it proceeds)

- `src/relation.rs` — `RELATION_RULES` row(s) for `shapes` gain a role dimension
  (only if not derivable).
- Stays graph-inert — this is NOT gating (gating is `needs`, settled in ADR-017).

## Links

- ADR-016 (closed role dimension), ADR-017 (the gating decision that left this out),
  RFC-003 (derivable-not-relational), PRD-010/SPEC-019 (records), SL-158 (origin).
