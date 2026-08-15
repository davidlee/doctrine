# Implementation Plan SL-251: Acts carry their own payload contract

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Seven phases. The design already sequenced most of this for us — `sec-7`'s
*Coupling, for whoever sequences this* names three dependency chains and
declines to turn them into phases, which is the boundary this document is on
the other side of.

The three chains, restated as the shape the phases take:

- **The table is upstream of everything.** The three renderers, the published
  document and every rung of the pin ladder read it. So the table is not a
  phase that can be deferred or parallelised against; it is PHASE-01 and
  PHASE-02, and nothing else starts until they are green.
- **The document is downstream of `render_document` and the manifest row
  together.** `asset_source.rs` refuses an `install/` asset that is neither
  published nor flagged, so the asset and its manifest entry are one landing,
  not two. PHASE-06 carries both.
- **The pointer shares only the const with the rest.** It is independent of the
  table, the renderers and the document in both directions, which makes its
  position a free choice — and PHASE-07 spends that freedom on going last.

## Sequencing & Rationale

**Why the compile barriers land first.** The whole design is built on the
observation that a description free to drift is worth less than no description,
and that the cheapest place to catch drift is the compiler. PHASE-01 buys three
compile barriers before a single row of contract exists: the `payload_variants!`
exhaustive match (a variant added to an enum and not to the list is a build
failure), `stringify!` on the invoked type (a rename is a build failure at the
match rather than a stale literal), and the closed `ExternRegion` with its
exhaustive resolution (a region nobody supplies is unspellable rather than
untested). Every later phase is written against barriers that already exist.

**Why the table and its literals are one phase and the value pins are another.**
PHASE-02 and PHASE-03 could be read as one piece of work — describe the closure,
then check the description. They are separated because they fail differently and
because their evidence is different in kind. PHASE-02's pin is an *inventory*
claim: every key is described. That is what the twelve exhaustive no-`..`
literals buy, and `SL-249`'s `RV-349` `F-4` already recorded the limit — `I9`
proves inventory, not mapping. PHASE-03 is the mapping half, and it is the rung
that took the design four review rounds to state as an instrument rather than as
a list of cases. Landing them together would let the harder one hide behind the
easier one's green.

**Why the extern region is its own phase.** It is about twenty-five lines, which
argues for folding it into the renderers. Two things argue louder against.
It is the only part of the contract the compiler cannot fully hold — `sec-3`
made the *supply* a compile barrier and the *mapping* a compile barrier, but
`RecordKind::ALL` is hand-maintained, so the kind set is carried by a test and
`R1`'s residue lives exactly here. And it is the one seam that crosses tiers:
the leaf declares a region it may not import and the command tier fills it. A
phase boundary around a tier crossing is worth its overhead.

**Why the renderers precede the verb.** PHASE-05 is three pure functions in the
leaf with nothing attached; PHASE-06 attaches them. Splitting there keeps the
CLI surface, the clap plumbing and the publication manifest out of the phase
where the rendering logic is being got right, and it means the renderers are
already pinned by the time the golden test starts depending on one of them.

**Why the pointer goes last, given that it could go anywhere.** It names an
invocation — `doctrine design contract --format prompt` — and until PHASE-06
lands, that invocation does not exist. A pointer to a verb the binary does not
have is worse than no pointer: it converts an agent's *I do not know where to
look* into *I looked where I was told and it was not there*. The independence
`sec-7` records is real, and it is what makes going last free rather than
costly.

## Notes

**On the research advisory.** `doctrine slice research 251` reports the baseline
drifted, listing `slice-251.md` as changed and `design.md` as added. That is the
design consuming the research rather than the research going stale: `sec-3`
traced the closure *against the source rather than recalled*, and `sec-6`'s
clap-capability claim, `sec-5`'s byte arithmetic and `sec-4`'s six token
authorities were each checked against the tree during drafting — which is where
`A3` was retired for asserting a tool limit without checking it. Planning
re-resolved the design's concrete references against the current tree directly
(paths, symbols, and `layering.toml`'s line 31) rather than restamping a
baseline that predates the artefact that superseded it.

**On the selectors.** The scope's coarse `src/command/design*` selector was a
typo for `src/commands/` and never matched; it is removed. `slice selector
doctor` still reports the precise `design-target` rows as redundant against the
broad `src/design_run/**` `scope-relevant` row, and that overlap is deliberate:
the broad row records that this slice *reads* across `design_run` — six token
authorities in five modules — while the narrow rows record the files it
*writes*. The two remaining `unmatched` rows are the files the slice creates.

**What the plan deliberately does not carry.** `SPEC-029` names the design verb
set as *start, show, apply, resume*; a fifth verb makes that line stale. The
design settled this as a prose update at reconcile rather than an amendment, so
it is not a phase here — but it is the first thing `/reconcile` owes.
`sec-7` also records two corrections this slice owes elsewhere: the memory
`mem.fact.design-run.apply-payload-vocabulary` is wrong about `Declaration`'s
`deny_unknown_fields`, and `ISS-346` and `ISS-333` record the same
silent-discard defect and should be merged before closure so the option-3
discharge lands on one id. Both are harvest and close work, not implementation.

**What implementation may still overturn.** `DEC-221` is held loosely at the
user's direction, and `DEC-229` has already used that latitude once — narrowing
the enum half from *generate the type* to *bar a non-exhaustive match*. A phase
that finds better on evidence should record what changed and why rather than
substituting quietly at the keyboard. The reading hazard while that stands:
`DEC-221`'s title still reads as though enum vocabularies are generated; read
the scope note at its head, not the title.

**The one hazard no test covers.** `R4` — the twelve exhaustive literals are one
`..` away from useless, and a test cannot see the difference. PHASE-02/VA-1 is
the review point that stands in for a pin, and it is the criterion most worth
not waving through.
