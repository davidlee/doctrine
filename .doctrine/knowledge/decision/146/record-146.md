# DEC-146: Record content is read per-id at the render layer, not carried by the corpus scan

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question was asked about the wrong struct

`SL-246`'s `OQ-6` names `CatalogEntity` (`src/catalog/hydrate.rs:111`) as the
type that carries identity and body but no facet. That is true, and irrelevant
once [[DEC-145]] put the composed read on `doctrine inspect`: `inspect` runs
over `ScannedEntity` (`src/catalog/scan.rs:105`), produced by `scan_entities`.

The correction makes the scan-carried arm *more* plausible than the scope
implied, not less — `ScannedEntity` already carries a `risk` facet, `tags`, and
an optional `body` behind `ScanMode { include_bodies }`. Adding a `RecordFacet`
beside them is a small, idiomatic change. It is still the wrong one.

## Why per-id wins

**The cost lands where the demand is.** A per-id read happens only for records
that survive the label filter, only when the level is not `skip`. The
scan-carried arm charges every consumer of `scan_entities` — `validate`,
`survey`, the priority graph, `backlog show` — for a field only `inspect` reads.

The user's framing: *better to make a small and bounded set of reads less
efficient than one serving the whole corpus.*

**It preserves the default for free.** `SPEC-013` pins rendered output
byte-exact per verb, so `skip` producing today's bytes is a hard requirement.
Under a lazy read that is structural — nothing runs. Under the scan arm it is
something you have to not break.

**It respects the layering and the precedent.** `RecordFacet` parsing lives in
`knowledge`. `knowledge::relation_edges` (`src/knowledge.rs:1090`, SL-096
PHASE-01) already establishes the shape: a `pub(crate)` kind-module accessor
that reads one record on another module's behalf. This is that accessor's
sibling, not a new mechanism.

**It matches how `inspect_from` already works.** That function scans the corpus
once for the graph, then re-reads per-entity detail from `root` for the queried
entity (`outbound_for`, the interaction-type read). Per-id record reads are the
same split applied to the neighbours.

**It does not reverse SL-222.** PHASE-09 deleted `[estimate]`/`[value]` from the
scan and left `check_facet_residue` (`src/catalog/scan.rs:270`) behind to catch
residue. Widening `ScannedEntity` with a new facet argues against a direction
this project set deliberately.

## What to watch

`read_record` (`src/knowledge.rs:1069`) reads the sibling `.md` unconditionally
and fails when it is absent. The `facets` level does not want the prose. Either
the accessor takes a facet-only path or the level pays for bytes it discards —
settle it in the design; it is not a fork.

The scaling risk is `IMP-398` S5. If the transitive knowledge closure becomes
the ordinary way people read a design rather than a deep dive, N stops being
bounded and the scan arm's amortisation wins. That is the revisit condition.

Shapes [[SL-246]]. Resolves that slice's `inq-2` (scope `OQ-6`). Downstream of
[[DEC-145]].
