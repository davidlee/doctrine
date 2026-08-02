# DEC-112: Anchor report emits a raw JSON struct, outside the list spine

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`doctrine spec anchors` (SL-243) emits its `--json` as a plain serialisation of
the report struct, with no `json_envelope` wrapper, and does **not** flatten
`CommonListArgs`. Format selection rides a dedicated enum, mirroring
`GraphFormat`. Its filters are its own —
depth, language, focus — and none of the entity-list flags (`--status`, `--tag`,
`--all`, `--columns`) appear on it.

## Why this is lawful under SPEC-013, not an exception to it

SPEC-013 fixes the envelope and the flatten for **numbered-entity `list`
verbs**. `D2` scopes `src/listing.rs` to the invariant list axes; `D3` makes the
flatten mandatory so a kind's `list` cannot shadow a shared flag. `spec anchors`
is neither a `list` verb nor an enumeration of numbered entities — it is a join
across specs, requirements and adapter-supplied units — so neither decision
reaches it.

The spec contemplates this boundary already. Its Concerns record SPEC-028's
observation commands as riding the grammar and rendering conventions while their
identity and query axes "do not enter the numbered-entity `CommonListArgs` spine
or conformance matrix". `graph` is the closer precedent still: it serialises
`CatalogGraph` raw, with its own `GraphFormat` enum rather than
`listing::Format`.

The positive argument against the envelope is about the data, not the
precedent. `{kind, rows}` needs a flat row type. This report carries a
containment tree of units, subtree rollups, and a PRD → SPEC → REQ → unit chain
with membership as an edge. Flattening that into rows either discards the tree
the design rests on or hides it inside a cell — the envelope would be
decoration over a shape it does not describe.

## What was rejected, and what was deferred

- **Rejected — full envelope plus the `CommonListArgs` flatten.** Uniformity
  bought by misrepresenting the payload, and it drags in four flags with no
  meaning here.
- **Deferred, not rejected — a self-identifying head** (a top-level
  discriminator naming the report, without the row model). This is the better
  end state: it honours what the envelope exists for while declining what it
  cannot satisfy. It is out of scope here because doing it on one verb would
  leave `graph` as the sole non-conforming surface, and doing it on both is a
  breaking change to an output other consumers read. Carried as **IMP-383**,
  which pairs the code change with the SPEC-013 amendment that would state the
  convention for non-list verbs.

## What this obliges

SPEC-013 `REQ-204` bills every verb a black-box golden pinning its rendered
table and JSON byte-exact. That obligation is unaffected by which shape is
chosen and must reach the plan. Nothing about this report may be added to
`src/listing.rs`, which `D2` keeps a pure leaf importing neither clap nor
`entity`.

## Provenance

Settled at the `inq-5` fork of design run `dr-019fc13a` (SL-243). SPEC-013 was
missed by the research round's governance thread and surfaced by the canon
pass — the binding authority on this question arrived late, which is why the
alternatives are recorded here rather than assumed.

## Amended by DEC-115

This record originally named the format enum's variants as
`json | markdown | dot`, which prejudged the `inq-9` fork. DEC-115 settles that
fork: the enum ships `json | markdown`, and gains a diagram variant with
IMP-385. Nothing else here changes — the shape of the JSON was never at issue in
that question.

## Related

- [[DEC-111]] — the sibling read-path decision from the same run; that one is
  about how the corpus is read, this one about how the result is emitted.
- [[DEC-115]] — narrows the format enum this record described.
