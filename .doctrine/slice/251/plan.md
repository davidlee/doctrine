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
set as *start, show, apply, resume*. That line is **already** stale by
`materialise`, and `contract` is the sixth verb, not the fifth — so the prose
update `/reconcile` owes is a two-verb correction, not one. (`sec-6` says "a
fifth variant beside the existing four"; `DesignCommand` has carried five since
`materialise` landed — `commands/design.rs:118-130`, classified at
`guard.rs:431-435`. The plan repeated the miscount and `PHASE-06/EX-1` now
carries the correction.) The design settled this as a prose update at reconcile
rather than an amendment, so it is not a phase here — but it is the first thing
`/reconcile` owes.
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

**PHASE-02 is the oversized one, and it is not split.** Twelve exhaustive
literals plus the whole `PAYLOAD` table is roughly twice any other phase. The
seams that would divide it all leave a worse intermediate: splitting the table
from its literals ships rows with no compile barrier under them, splitting the
literals by defining module gives `attestation.rs` a phase of about forty lines,
and splitting structs from enums leaves a root that references half a closure.
`DEC-227` accepted twelve literals as the dominant mechanical cost when it chose
the full closure over top-level-only; the size is the decision's, and hiding it
behind a phase boundary would not make it smaller.

**Where PHASE-02 and PHASE-03 will feel like rework and are not.** (`PHASE-03`'s
`EX-9` used to say this and only this, which made it a note in a criterion slot
that nothing could fail; it now carries the checkable half — amended fixtures
keep their no-`..` form — and the rationale lives here.) The coverage
equality is the thing that tells us which declaration sites no fixture reaches,
and it cannot say so until it runs. So PHASE-03 is expected to go back and put
elements in containers PHASE-02 left empty. PHASE-03/EX-9 says this in the plan
so a reader does not score it as PHASE-02 having been done badly — under the
design's rev-72 rework, *fully populated* keeps exactly one obligation (the
no-`..` literal), and container non-emptiness stopped being a fixture rule at
all when it became a pin.

## Plan review, 2026-08-15 — what changed and why

An adversarial pass over plan-against-design found nine gaps. All are now in
`plan.toml`; this is the account of them, since a criterion carries its
obligation but not its history.

**Three obligations were mis-stated — a criterion that, met literally, ships the
wrong thing.**

- `PHASE-02/VT-1` said "for each of the **twelve** closure structs … equals its
  `KeyContract` rows". `sec-8` pin 1 has **eleven** call sites and says
  `SubmissionEnvelope` "cannot take that call site" — no `TypeContract`, so
  `PAYLOAD` would be three keys against thirteen rows. Its real pin, the
  disjoint union `{envelope keys} ⊎ {ten act keys} == {thirteen rows}`, was in no
  phase at all; it is now `PHASE-02/VT-3`. `EX-2` had the distinction right, so
  the plan contradicted itself one criterion apart.
- `PHASE-07/VT-2` asserted the worked example "parses as a valid
  `ApplyRequest`" — the exact claim `DEC-228` was corrected off, for two reasons
  both live in the source (`envelope.rs:75-81`): `"known_revision":<n>` is not a
  JSON value and the final `concat!` arm is prose. It now pins the JSON arm
  alone, after substitution, and `EX-9` carries the `JSON_ARM` / `concat!`
  restructuring the pin needs and the plan had not allocated.
- The two extern-region rows — `CreateRecord.kind` as `Token(Extern)` and
  `.facet` as `Map { key: Extern{…, selector: "kind"} }` — had no exit criterion
  anywhere, while `PHASE-04/EN-2` cited `PHASE-02/EX-1` as though it carried
  them. It does not. Now `PHASE-02/EX-9`, with `EN-2` re-pointed.

**Three obligations were dropped or downgraded.** `sec-8` pin 7's *refusal*
surface survived only as a manual `VA` (now also `PHASE-07/VT-3`); its
*classification* pin pointed at `guard.rs`, where the arm is a compile barrier,
rather than `main.rs`'s `cls` / `observation_write_class_split` where `sec-7`
puts it (now `PHASE-06/VT-2`). Pin 9's selector-sibling assertion — which
`sec-8` says "no other pin reads" — was absent, and is folded into
`PHASE-05/VT-1`. And `sec-5`'s three *semantic* rendering rules (payload placed
by tagging, `BARE STRING` marking, struck-out untagged token) had no criterion
behind `VA-1`'s shape comparison, though they are the two failures `sec-1`
opens on; now `PHASE-05/EX-8`, with the fixed-string parentheticals as `EX-9`.

**Two over-claims would have failed on contact.** `PHASE-01`'s "all fourteen
enums" and "each `VARIANTS` non-empty" are false for the untagged
`WireFacetValue`, which pin 4's own extraction table skips; and pin 4's samples
live in `payload_contract.rs`'s test module per `sec-8`, which a sibling
`tests.rs` cannot reach — so `PHASE-01/VT-1` moved, and `PHASE-03/VT-3` split,
leaving the struct probe in `tests.rs` and the variant probe (`VT-4`) beside its
samples.

**Retired ids, spent and not reusable** (`PHASE-NN` and criterion ids are
immutable — a later append takes the next number, never these):

- `PHASE-01/VT-2` — a `VT` over a compile barrier `sec-8` lists under *four
  things need no test*, with keywords the macro's own definition satisfies. It
  could not fail. `VA-1` carries that barrier properly, both ways.
- `PHASE-05/EX-6` — restated a choice `sec-5` explicitly leaves to the
  implementer and calls not load-bearing.

## PHASE-01 execution, 2026-08-15 — one criterion appended

`PHASE-01/EX-9`, appended at execution. `sec-4` concludes "All six are `const
fn`, so the token array stays a const" and `EX-5` turns that into an obligation
to consume all six authorities. The `const fn` half is true — all six are, at
the exact lines `sec-4` cites — but the conclusion does not follow for two of
them. `Provenance` and `ReviewDisposition` have **no fieldless variant between
them**: `ShapingQuestion { record: String }`, `ImportedProse { section:
DesignId, … }`, `Conducted { review: ReviewRef }`, `Waived { reason: String }`.
Calling `label()`/`arm()` in a const initialiser therefore needs a temporary the
const evaluator must drop (E0493, confirmed by a compiled repro), and two of
those four cannot be built from the leaf at all — `DesignId`'s `raw` and
`ReviewRef`'s tuple field are private with non-`const` constructors.

The repair keeps the single source and moves where it is enforced. Those two
enums take the naming arm, and `VT-1`'s per-variant walk — which the
exhaustiveness barrier makes total over variants, so it cannot silently lose a
case — asserts each named token against the authority's own return for that
variant. That is a *detector* where the other four get a barrier, which is
exactly the substitution `sec-4` warns against at line 992; it is taken here
because the barrier is unavailable rather than merely inconvenient, and `EX-9`
records the difference instead of hiding it. `/reconcile` owes `sec-4` the
correction, alongside the three it already owes.

**What this owes the design.** Three of the nine originate upstream rather than
in the plan, and `/reconcile` should carry them back: `sec-6`'s *fifth variant*
is the sixth; `sec-4`'s "pin 4 covers all fourteen uniformly" reads against
`sec-8` pin 4's own `Untagged → skipped` row; and `sec-8` pin 1's eleven/twelve
split is stated correctly but is easy to read as twelve call sites — which is
how the plan came to say it.
