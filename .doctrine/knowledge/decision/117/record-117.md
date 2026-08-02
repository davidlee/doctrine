# DEC-117: New component spec descending from PRD-012, parent deliberately unset

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The spec anchor map capability gets **one new tech spec**:

- `c4_level = component` — the normal stopping altitude; container would
  overstate a verb plus a config table, and code level is exceptional.
- `descends_from = PRD-012` — the product intent genuinely is PRD-012's.
- **`parent` unset**, with the absence authored as an open question on the spec
  itself.

One spec, not two: the report and the adapter contract ship together.

## Why no parent

The candidate was SPEC-006 (Spec composition machinery, container), which would
have made the new spec a sibling of SPEC-017. The census supports that placement
structurally — SPEC-027 is the precedent, a projection-and-report component
parented under SPEC-004, the container owning the data it projects, while
descending from a product spec in a different tree (PRD-016).

It was rejected as a **compromise that would be cemented**. SPEC-006's
responsibilities are uniformly about assembling, parsing and reassembling specs;
the anchor map composes nothing — it reads specs and joins them against an
external inventory. Calling it a component of composition machinery is a stretch,
and a parent edge is authored truth the registry harvests into the decomposition
tree every later agent reasons from. A wrong edge is worse than a missing one
precisely because it is invisible: absence shows as absence, while a plausible
parent reads as a finding.

The corpus is 27% dark by loc. A containment tree over a partial map should be
allowed to be partial.

**Absence is lawful, not a workaround.** Verified three ways:

- PRD-012's principle is "a spec has **at most one** parent" — zero satisfies it.
- `spec validate` reports the corpus clean and its findings vocabulary carries
  only `second_parent` (multi-valued); there is no orphan or missing-parent check.
- The corpus already ships specs with absent spine fields — SPEC-004, SPEC-013
  and SPEC-016 each carry no `descends_from`.

**The absence is authored, not blank.** An empty field reads as "nobody filled it
in"; a recorded open question reads as "we looked, and containment is not yet
determinable". Only the second is true, so only the second should be legible.

The alternative of minting a new container for corpus-governance reporting was
declined: a container holding a single component is a boundary nothing supports.
If a second governance-report component appears, that is the promotion trigger.

## One spec rather than two

The adapter contract does not get its own spec. The argument first given for this
— that its only consumer is the report — was **wrong**, and the census itself is
what refutes it: PRD-012 `OQ-2`'s future code-structure importer would read the
same unit inventory.

The argument that survives is different. `OQ-2` is unresolved about what the
importer even reads, so splitting now would carve a boundary around a consumer
whose shape nobody knows. Do not pre-split; record the trigger — if `OQ-2`
resolves and the importer wants the same inventory, that is when the contract
earns separate identity.

## Provenance

Settled at the `inq-1` fork of design run `dr-019fc13a` (SL-243), after the
`/spec-coverage-assessment` pass the fork was deferred to. The coverage map is
runtime-tier at `.doctrine/state/sl-243-spec-coverage-map-anchor-map.md` and is
disposable; this record and the slice notes are its durable residue.

The census also found the capability **fully dark** — BM25 over the obvious
keywords returns only slices, backlog items and this run's own decisions. No
PRD, no SPEC, not even a prose claim. So there was no existing spec to amend.

## Related

- [[ISS-301]] — the general form of this: parent edges elsewhere in the corpus
  that may be inferred rather than determined, SPEC-027 the known candidate.
