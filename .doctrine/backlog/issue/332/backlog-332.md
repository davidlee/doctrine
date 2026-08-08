# ISS-332: SPEC-019 closed-enum list omits provenance

Found by the `VA-1` agent read of `REV-050`'s amendment (SL-249 PHASE-07 T8).
Prose defect in `SPEC-019` only — **no code defect**, and no `EX-6` scope breach.

## What is wrong

`REV-050` added the `evidence` facet to `SPEC-019`, and with it a fourth closed
enum, `provenance` (`inspection` / `experiment` / `reproduction` / `citation`).
The facet bullet at `spec-019.md` §*Typed facets and the evidence structure*
names it correctly. Two later sentences that enumerate the closed enums were not
refreshed and still list three:

- `spec-019.md` ~l.128 — "The closed enums (`confidence`, `basis`, constraint
  `source`) ride the same `\"\" -> None` optional seam …"
- `spec-019.md` ~l.353 — §*Risks*, **Facet-enum drift**: "The closed facet enums
  (`confidence`, `basis`, constraint `source`) each need a known-set guard
  mirroring their variant set …"

Both are now incomplete: `provenance` is a closed enum, it rides that same
`"" -> None` seam (`optional_enum(&raw.provenance, "provenance")?`,
`src/knowledge.rs:818`), and it is declared `FieldShape::Closed(Provenance::KNOWN)`
at `src/knowledge.rs:1000`.

## Why it is prose-only

The shipped code is correct and already guarded. `provenance_known_set_matches_variants`
(`src/knowledge.rs:3502`) is exactly the drift canary the §*Risks* bullet asks for,
and it covers `Provenance`. So the second site understates the mitigation that
exists rather than describing a missing one. Nothing needs implementing; two
sentences need a fourth item.

## Why the PHASE-07 canary cannot catch it

`tests/governance_kind_coverage.rs` measures **record-kind** coverage: the paired
long-name↔prefix form for each of `kinds::RECORD`, and the `\bfour\b` arithmetic
identity per tier. A stale *facet-enum* enumeration carries no kind name, no
prefix, and no numeral — it is invisible to every rule the canary runs. This is
the same `R-inventory` shape the phase has now hit five times: a hand-maintained
list that a widening amendment did not sweep. The canary narrows the class it
covers; it does not close it.

## Suggested fix

Add `provenance` to both enumerations. One-line edits, no structural change, no
`.toml` tier change. Cheapest carried on the next `SPEC-019` touch rather than as
its own revision — the spec is not wrong about behaviour, only under-inclusive in
two lists.

## Related legibility note (not part of this issue)

Three further `SPEC-019` sections are knowingly four-kind and say nothing about
`ISS-316` owning the remainder: §*Per-kind lifecycle vocabularies*, the §6 allowed
transition matrix, and `settle`'s four resolving states. `EX-6` deliberately
excludes `EVD` / `HYP` / `CPT` lifecycle vocabulary and supersession rules from
this amendment, and that boundary held — but a reader arriving cold cannot tell
those sections are *scoped* rather than *stale*. A one-line pointer to `ISS-316`
in each would settle it. Recorded here so the observation is not lost; it is not
a defect in `REV-050`.
