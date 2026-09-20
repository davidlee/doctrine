# IMP-465: Composed knowledge read — the product pass VH-1 asked for

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`SL-246` shipped the composed knowledge read: `doctrine inspect <ID> --knowledge
skip|facets|full` and `doctrine design show <SLICE>` with the same dial. It
passes its suite, holds its invariants, and its human verification came back
**negative on the product** while explicitly not disputing correctness.

`VH-1` (`PHASE-06`), attested 2026-09-20, recorded verbatim at `RV-372` `F-5`:

> it seems like a scrappy prototype of what I actually want to see. Styling is
> incongruent with the document; the omitted facets defy understanding .. would I
> reach for it again? the CLI flags are cumbersome and TBH I expected knowledge
> would be visible by default, and it needs a decent amount of iteration to feel
> properly useful. The verdict is: **disappointing as a feature, but not obviously
> incorrect**.

This item carries that verdict forward. It is the umbrella; two parts have their
own homes because they are defects rather than taste.

## The four complaints, and where each lives

| complaint | route |
|---|---|
| the omitted facets defy understanding | **`ISS-467`** — a withheld tier is not disclosed (`STD-003`) |
| I expected knowledge visible by default | here + **`IMP-398`** — see below |
| the CLI flags are cumbersome | here |
| styling incongruent with the document | here (`RV-372` `F-8`) |

## The default, which is the load-bearing one

`C1` makes `Skip` the default so that adding the level changes no existing
output, and it is expressed in the type so no call site can drift
(`KnowledgeLevel` derives `Default` with `#[default] Skip`). That is *correct as
built* and is why `I1` holds. The dispute is with `DEC-145`, not the code.

It is also evidence against a decision the design already made: `design.md` § 7.2
`D4` pushed the one-line pointer out of scope as "the separate, cheap answer", and
`DEC-145` calls discoverability "not part of this decision". The attestation says
the composed read *without* that pointer does not land — an agent, or a human,
does not find out the knowledge is there. So `IMP-398`'s pointer line is not an
independent nicety; it is the other half of this feature.

## Styling

The block is `knowledge show`'s standalone-record idiom — a literal `[facet]`
header and indented `key: value` lines — appended to ~3,459 lines of Markdown.
The shared producer is *correct* (`C4`, `STD-001`: one facet enumeration), so a
fix has to change the **spelling** without forking the enumeration. That
constraint is the interesting part of the work.

## Flags

`--knowledge facets` is three words for the thing a reader wants most of the
time. Worth considering against the default question rather than separately: if
the level were visible by default, the flag's job changes from *opt in* to *opt
out or widen*.

## What this item is not

Not a correctness claim. `RV-372` records the gate green (7801 tests, zero
failures), conformance clean (22 conformant, 0 undelivered), and every `VA` row
re-derived with its positive control. The feature works; it is not yet good.

Related: `RV-372` `F-5`, `F-7`, `F-8`; `SL-246`; `IMP-398`; `ISS-467`;
`DEC-145`, `DEC-150`, `DEC-261`; `IMP-403` (unfilled facets, which is doing much
of the measured saving — `RV-372` `F-4`).
