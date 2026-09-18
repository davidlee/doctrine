# Entity reads carry their knowledge records

## Context

The managed design workflow (`PRD-019`) moved the substance of slice design into
knowledge records. A design no longer argues and concludes in one document — it
argues in `design.md` and *rules* in `DEC` records. Nothing renders the two
together, so the ruling and the argument for it can only be read apart.

`SL-244` is the specimen. Its `design.md` is **3,456 lines** and cites **20
distinct `DEC` records by id**, quoting fragments of them inline (`DEC-121`
appears 8+ times as quoted phrases). Fifteen knowledge records point at the slice
— twelve via `shapes` (ten `DEC`, plus `EVD-012` and `QUE-206`) and three `DEC`
via `references(concerns)`. `doctrine slice show SL-244` renders **none** of them:
it emits outbound relations only, as does every other kind's `show`.

The derivation already exists. `doctrine relation list --target SL-244` returns
the inbound edges correctly. It lives in a third command family, reachable from
neither the entity nor the knowledge record, so nothing leads an agent to it from
where the question is actually asked. A substantial part of this change is
routing to an existing capability rather than building a new one.

Originates from `IMP-398` (knowledge record discoverability), whose remaining
candidates stay on the card.

## Scope & Objectives

**The composed read.** An entity rendered together with the *content* of the
knowledge records that point at it — not a list of ids, and not a graph. The
subject is any entity kind that carries knowledge relationships (`SL`, `SPEC`,
`PRD`, `ADR`, `RFC`), with the slice/design case as the motivating and acceptance
specimen.

**Relation-keyed, one hop.** The record set is derived from inbound relation
edges (`ADR-004`: storage is outbound-only, reciprocity is derived). Prose
citations are explicitly *not* consulted — see Non-Goals.

**A verbosity dial.** Three levels, defaulting to today's behaviour:

| level | renders |
|---|---|
| skip | no knowledge (the current, default read) |
| facets | the deciding fields only — what rules and what changes whether the ruling still stands, never the argument that got there (`DEC-150` fixes the set per kind: a `DEC`'s `context` + `choice` + `rationale`, a `QUE`'s `question` + `why_matters` + `answer`) |
| full | complete record bodies |

The middle level is the one that earns the feature. Measured on the specimen it
costs **~30% of `full`** (31.8 KB vs 107 KB across the fifteen records) — a real
saving, but not the order of magnitude first assumed; see `research/research.md`
§ *The specimen, re-measured*.

**Not the pointer line.** `DEC-145` leaves discoverability unsolved — an agent
asks this question at `<kind> show` — and names a one-line pointer there as "the
separate, cheap answer; it is not part of this decision." Drafting confirmed it
is separate and found it is not cheap: it touches six `show` renderers, moves
every one of their byte-exact goldens, and unlike this slice's levels it is
unconditional, so it changes default output everywhere. Returned to `IMP-398`
with the finding.

**A reader for the design document, on the verb that already names it.** Added
at review (`DEC-261`, superseding `DEC-260`). No verb renders a design document
today: `slice show` excludes it by contract, `slice design` is a deprecated
scaffold, and `design show` renders the design *run*'s turn envelope. This slice
reclaims `doctrine design show <SLICE>` for the document and moves the envelope
to `--format prompt` — a value that already exists and is already its default
rendering's name, so nothing is renamed and no caller learns a new verb. The
deprecated `slice design <ID>` leaf retires with it.

This is a **scope addition** and is named as one: it is a verb rehome that
`IMP-393` had carried as separate work, and it changes the default output of a
live `SPEC-013` golden surface — the one the managed design run is itself driven
through. It was taken because the alternative siting (`slice design show`) was
three levels against `SPEC-013`'s stated two, and because the migration measured
at 14 references rather than the multi-seam cost it had been assumed to be.

**Objectives**

1. An entity read can carry its inbound knowledge records' content.
2. The verbosity dial is explicit and defaults to no change in existing output.
3. The inbound derivation is factored so a later transitive closure (`IMP-398`
   S5 — the recursive knowledge view) extends it rather than replaces it.
4. Empty facets are legible as gaps, not as absence of content.
5. `doctrine design show` means what `show` means everywhere else in the CLI,
   and the turn envelope stays reachable under a name it already has.

## Non-Goals

- **No prose-citation parsing on the read path.** `SL-244`'s `design.md` cites
  ~10 `DEC` records it holds no edge to (`DEC-063` `shapes` `SL-233`; likewise
  `DEC-065/066/067`, `-073/074`, `-086/088`, `-101/102`) — inherited governing
  context, cited but unlinked. Closing that gap is a *validate-and-warn* concern
  on the authoring path, deferred to its own backlog item. This slice reads
  edges; it does not scan prose.
- **Not the recursive knowledge closure.** Transitive walking through
  knowledge→knowledge edges, halting at non-knowledge nodes, is `IMP-398` S5.
  This slice keeps the seam open for it (objective 3) and stops at one hop.
- **Not the remaining `IMP-398` candidates** — `knowledge list --kind`,
  `--related-to`, the `KN` kind alias, repeatable `graph --label`, graph focus
  anchoring. All stay on the card.
- **Not a corpus facet-hygiene pass.** Empty facets are rendered honestly, not
  backfilled.

## Affected surface

Narrowed by the inquiry (`DEC-145`, `DEC-146`, `DEC-147`), then widened at review
by `DEC-261`. This is the orientation map; `design.md` § 5.6 carries the
authoritative touch-set, and the two are not duplicates of each other.

**The composed read** — the capability itself, reached from `doctrine inspect`.

- `src/relation_graph.rs` — `InspectView` (`:572`), `inspect_from` (`:636`),
  `render_from` (`:760`): the inbound derivation, and where selection splits from
  rendering (`DEC-147`).
- `src/knowledge.rs` — the per-id record accessor (sibling of `relation_edges`),
  the level's block renderer, and the tier / empty-policy inputs that
  `format_facet` **and** `facet_json` both take (`DEC-149`, `DEC-150`).
- `src/commands/inspect.rs` — the verbosity level on `doctrine inspect`.

**The design read** — a scope addition taken at review (`DEC-261`, superseding
`DEC-260`).

- `src/commands/design.rs` — `show` renders the design document plus the
  knowledge block; `--format` gains `document` and defaults to it, and `prompt`
  keeps the turn envelope unchanged.
- `src/slice.rs` — the deprecated `SliceCommand::Design` leaf and
  `scaffold_design_doc` retire; a `design_document` reader replaces them.
- `src/commands/guard.rs`, `src/commands/cli.rs` — the rows keyed on the retiring
  `SliceCommand::Design` variant delete with the variant; `design show` stays
  read-classed.
- `install/routing-process.md` and ~4 emitted strings — the migration itself:
  every place naming `design show` as the turn read re-points at `--format
  prompt`. Pricing this at 14 references is what made `DEC-261` decidable.

**Test surface.** Three existing suites go red here *on purpose* — the retiring
leaf's tests, one help assertion carrying `SL-233` `EX-5`, and the bare
`design show` call sites. `design.md` § 5.6 names them; a red suite it does not
name is the byte-identity alarm, not a golden to update.

**Adjacent, and not a phase of this slice.** Five committed memories document
`design show` as the envelope read. Re-attesting them is `CHR-073`, a
`/reviewing-memory` follow-up — the corpus is not code and re-attestation is its
own verb.

**Dropped.** `src/catalog/**` — `DEC-146` leaves the corpus scan untouched.
`src/commands/relation.rs` — `relation list --target` is prior art for the
derivation, not a touch site. `src/relation.rs` — `DEC-148` filters on the
source's kind and writes no label allow-list, so the relation vocabulary is read,
not changed. `src/kinds/mod.rs` — `is_record` (`:128`) is consumed as-is.
`src/spec.rs`, `src/adr.rs`, `src/rfc.rs` — the one-line pointer returned to
`IMP-398` (see *Not the pointer line* above), so no `show` renderer is touched.

## Governing context

- `ADR-004` — relations stored outbound-only; reciprocity is derived. Fixes the
  derivation as a reverse scan, not a stored back-edge.
- `SPEC-018` — cross-corpus relation contract; the label vocabulary and roles.
- `SPEC-019` — knowledge-record entity surface; record kinds and typed facets.
- `SPEC-013` — CLI surface; flag grammar and listing conformance.
- `PRD-019` — managed design workflow; why design content lives in records.
- `PRD-010` — epistemic and governance records.

## Risks, assumptions, open questions

**R1 — the facet tier is routinely empty, and the renderer hides it.** Measured
in the research round (2026-08-05):

- Corpus fill: decisions **24%** populated (35 of 142), questions **10%** (4 of
  38), assumptions 37%, evidence 58%, constraints 60%.
- Population is **all-or-nothing**: every decision carrying one textual field
  carries all of them (35/35/35 on `context`/`choice`/`rationale`) — there are no
  partially-filled records. So *which* fields the level selects barely matters;
  *whether the record has a facet* is the whole question. This is why `OQ-4`
  outranks `OQ-2`.
- On the acceptance specimen, **4 of the 15** inbound records render **nothing**
  at the `facets` level — `QUE-206`, `DEC-140`, `DEC-141`, `DEC-142` — and all
  four carry substantial prose (`QUE-206` is 6.7 KB).

The mechanism is the renderer itself: `format_facet`
(`src/knowledge.rs:1302-1364`) emits the `\n[facet]\n` header **only when at
least one axis is populated**, and `show_opt_line` (`:1286-1291`) drops absent
fields silently — its doc comment says so outright. An empty facet therefore
renders as *nothing at all*: not a blank block, not a header. The render
**reproduces** the half-invisible failure this risk names
(`mem.pattern.doctrine.amend-knowledge-both-tiers`; observed on `DEC-099`,
`ASM-007`, `QUE-201`) rather than merely failing to fill it. Note the JSON path
disagrees — `facet_json` (`:1432-1462`) emits every field with `Option` → `null`
— so `OQ-4` reconciles two existing behaviours rather than inventing one.

Objective 4 is the mitigation. A naive middle level inherits the concealment and
tells the reader *less* than opening the record would.

**R2 — inbound noise swamps the signal.** `SL-244` carries 28 inbound edges, of
which **11 are `references(originates_from)` from backlog items** — harvest
exhaust, not design content. An uncurated inbound render makes the common read
worse. Mitigated by `DEC-148` — selection filters on the source's kind, which
excludes all 11 because a backlog item is not a knowledge record.

**R3 — token cost.** `full` on `SL-244` is 3,456 lines plus sixteen records.
Bounded by construction (`skip` is the default), but the dial's levels must be
worth their price on every axis the project weighs.

**A1** — the inbound edge set is a sufficient proxy for "the knowledge that
shapes this entity", accepting the ~10 cited-but-unlinked records as a known,
separately-tracked miss.

All six were dispositioned in the design run's inquiry and are closed. They are
kept here as the record of what was open; the ruling is the `DEC`, not this list.

**OQ-1 — surface.** A flag on the existing `<kind> show` / design read, or a
distinct kind-agnostic verb (`knowledge digest <ref>`)? Determines whether this
is one seam or one per kind. → **`DEC-145`**: neither; it rides `doctrine
inspect`, the kind-agnostic inbound view that already exists.

**OQ-2 — facet selection per record kind.** Which fields constitute the
`facets` level for each of the seven record kinds. → **`DEC-150`**.

**OQ-3 — label curation.** *As framed, superseded.* The question assumed the
fix was curating which inbound labels qualify. → **`DEC-148`**: it is not a
label question. Selection filters on the SOURCE's kind (`kinds::is_record`), and
no label allow-list is written. The noise this OQ named — `originates_from`
backlog rows, and `reviews` — comes from sources that are not knowledge records
at all, so a label list would have excluded it only by coincidence. `DEC-148`
also explicitly rejects dropping `concerned by`, which this OQ's framing invited.

**OQ-4 — empty-facet rendering.** Visible gap marker versus silent blank
versus fallback to prose. → **`DEC-149`**: marked, two distinct markers
(unfilled record vs. a kind with no facet by design), no prose fallback.

**OQ-5 — closure seam shape.** How far to factor the inbound derivation now so
S5's typed traversal extends it (objective 3) without speculative generality.
→ **`DEC-147`**: split selection from rendering; the record's caption is
selection-supplied text, not a `RelationLabel`. Nothing else is generalised.

**OQ-6 — the composition seam** (added post-research). *Premise void.* It scoped
the choice as `CatalogEntity` growing a `RecordFacet` versus a per-id read.
→ **`DEC-146`**: `CatalogEntity` is not on `inspect`'s path, so that arm is moot;
the live comparison was against `ScannedEntity` growing one, and the ruling is a
per-id read at the render layer with the corpus scan untouched.

**Post-research reordering.** `OQ-4` is primary — and `DEC-149` is what made it
so, since ruling that the renderer is built for a healthy corpus invalidated the
fill-rate argument `OQ-2`'s research answer rested on. Field selection was **not**
low-stakes after all: see `DEC-150`.

## Design inputs carried from research

Full artefact: `research/research.md` (runtime tier — **gitignored and
disposable**; re-run with `doctrine slice research 246` if absent). The findings
that must survive it are inlined here and in `ISS-316`.

- **Ride the `Detail` precedent, do not invent a dial.** `design show --full`
  widens a `Detail::{Normal, Full}` enum
  (`src/design_run/render/envelope.rs:86-117`) whose comment states the
  principle: the caps are what make *"normal is a subsequence of full" the same
  code path rather than two implementations that could disagree*. The three
  levels should be one renderer under different bounds. Nothing in the tree today
  is a three-level detail dial — this would be the first.
- **`OQ-2`'s research answer is partly superseded — see `DEC-150`.** It proposed
  `DEC` `context`+`choice`+`rationale`; `QUE` `question`+`why_matters`; `CON`
  `statement`+`source`+`applies_to`; `ASM` `claim`+`confidence`+`basis`, excluding
  `alternatives`, `consequences`, `decided_*`, `answer`/`answered_*`, `validated_*`
  and `waiver_*`. Two different arguments were doing that work, and only one
  survived: `alternatives`/`consequences` are excluded as bulk (they are most of
  the cost — `DEC-080`'s facet renders at ~4.5 KB against a 274-byte body), but the
  short status fields were excluded on **current fill rate** (`answer`: 0 of 38),
  and `DEC-149`'s healthy-corpus ruling voids that argument. `DEC-150` restores
  `answer` to `QUE`, `waiver_reason` to `CON` and `invalidated_by` to `ASM` — each
  is short and each changes whether the record still binds — and rules the three
  ungoverned kinds honestly: `EVD` and `HYP` take all their fields, `CPT` has none
  and renders `DEC-149`'s by-design marker.
- **`EVD`/`HYP`/`CPT` have no governance to defer to** — `ISS-316`. `SPEC-019`
  specifies four record kinds, not seven. Any field list for those three is
  **invention, and must be labelled as such** rather than presented as derived.
  They are also the thinnest in evidence (`EVD` n=12, `CPT` n=1, `HYP` **n=0** —
  never used). `CPT` has no facet fields at all by design
  (`knowledge.rs:570-573`).
- **The inbound label set is closed and pre-named** (`src/relation.rs`):
  `shaped_by` (`:528`), `concerned by` (`:426`), `spawned_by` (`:542`),
  `supported_by`/`disputed_by` (`:699`/`:712`), `superseded by`. **The inference
  drawn from this is superseded by `DEC-148`**: the closed six is a *consequence*
  of filtering on source kind — those are by definition the labels a record can
  point outward with — not a list to select from. And the specimen does not argue
  for dropping `concerned by`: `DEC-144` arrives under it and is substantive, so
  curating it out to dodge three empty facets would fix the wrong thing. That
  defect is `DEC-149`'s.
- **`Shapes` targets include the record kinds themselves** (`:530-531`), so
  knowledge→knowledge edges are legal. Objective 3's seam for `IMP-398`'s
  recursive view is real, not hypothetical.
- **`SPEC-013` pins rendered output byte-exact** via per-verb black-box goldens
  (`SPEC-013:13`, `:123`) — which is what makes `skip`-as-default load-bearing
  rather than merely polite.
- **No duplication risk.** Nothing else claims a record-content render;
  `relation list --target` derives the inbound set correctly but emits ids only.

## Verification / closure intent

`SL-244` is the acceptance specimen: reading its design with the dial at
`facets` must surface the twelve `shapes` decisions' rulings without the 11
`originates_from` backlog rows, and must do so at a cost a working agent would
actually pay. Closure additionally requires the existing kind-`show` suites green
unchanged (the behaviour-preservation gate on shared machinery), and the default
read byte-identical to today's.

`DEC-151` splits how that is proven, because the specimen is a live slice whose
inbound record set keeps changing and a golden over it would rot:

- **By test.** Synthetic-corpus goldens pin the mechanics at all three levels —
  a filled `DEC`, an unfilled one, a `CPT`, a record reachable under two inbound
  labels (the `EVD-012` dedup case), and a backlog item plus a review as inbound
  noise.
- **By agent, at audit.** The specimen claim itself. The attestation must name
  the record ids surfaced, the ids excluded, and the rendered byte count at each
  level — enough to re-derive, since `SL-244`'s authored corpus is committed. An
  attestation reading "looked fine" does not discharge it.
- **Declined on inspection, not unavailability:** a live-corpus invariant test
  asserting `facets` ⊆ `full` and `skip` byte-identical to today's. Half of it
  needs a stored baseline of today's output, which *is* the live golden it was
  meant to avoid; the other half holds by construction under `DEC-150`'s single
  field-order table. See `DEC-151`.

## Summary

## Follow-Ups

- **`IMP-393`** — *Reader-facing design render for review.* This slice reclaims
  `doctrine design show` for the design document and partially discharges that
  item (`fulfils`, degree `partial`). What stays open there: the slice's other
  three documents (plan, notes, and the scope `slice show` renders), and the
  fuller reader-facing render — run metadata, attestation standing, open
  inquiries, the navigable neighbourhood — none of which this slice delivers.
  `IMP-457` was minted during drafting for the same defect and is **closed as a
  duplicate** of `IMP-393` (`RV-370` `F-6`).
- **`SPEC-013`'s two-level clause.** The spec states *"The surface is a two-level
  clap subcommand tree"*, and six groups under numbered entity kinds are already
  three levels — `slice selector`, `spec req`, `spec interactions`, `revision
  change`, `memory sync`, `knowledge edit <kind>`. `SL-246` conforms and so does
  not need this reconciled, but the clause is descriptively false of its own
  governed surface and someone should carry a `REV` for it. Surfaced by `RV-370`
  `F-1`; not this slice's to fix.
- **`CHR-073`** — *Re-attest the five memories naming `design show` as the
  envelope read.* Surfaced by `RV-370` `F-16`, which found them missing from a
  migration count the design called measured. Not a phase — re-attesting a
  memory is its own verb and the corpus is not code — but **not deferrable
  either**: a stale memory is injected into every agent's context by `memory
  retrieve` and the surface hook, so between the reclaim shipping and these
  edits every agent is told to run a command that now means something else.
  Sequence it into close rather than letting it drift. Whether it warrants a
  hard `needs` gate on this slice is a judgement left to close.
- **`IDE-054`** — *Audit the CLI for format/content axis coupling.* `RV-370`
  `F-9` found `--format` (a content selection) and `--json` (an encoding)
  colliding on `design show` with no stated precedence. This slice resolved its
  own instance the cheap way and declined the principled split on cost;
  `IDE-054` asks whether the coupling is widespread enough to be worth a
  standard.
