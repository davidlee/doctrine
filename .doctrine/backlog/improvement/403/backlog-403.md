# IMP-403: Knowledge facets are systematically unfilled — investigate the capture surface

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The measurement

Re-measured over `.doctrine/knowledge/*/NNN/record-*.toml` during `SL-246`'s
pre-design research round (2026-08-05):

| kind | n | facet populated |
|---|---|---|
| decision | 142 | 35 (24%) |
| question | 38 | 4 (10%) |
| assumption | 8 | 3 (37%) |
| evidence | 12 | 7 (58%) |
| constraint | 5 | 3 (60%) |

Population is **binary and lock-step**: every decision carrying one textual field
carries all of them (35/35/35 on `context`/`choice`/`rationale`). There are no
partially-filled records. So this is not authors getting bored halfway — it is a
population that either engaged the structured tier or never touched it.

`answer`/`answered_by`/`answered_on` on `QUE` is **0 of 38**.

## Why this is not a data-hygiene chore

The split between filled and unfilled is a property of the *creation context*,
not of the author's diligence. Finding the causal pattern means looking at what
was different about the 35 — which skill was running, which verb minted the
record, whether a human was in the loop — and at the interaction surfaces that
do or don't ask for the fields. That investigation is the work here.

## Leads

**1. There is no writer for the structured tier.** `doctrine knowledge` offers
`new | list | show | inspect | status | paths`. There is **no `edit`**. Filling a
facet means hand-editing TOML. Compare the kinds that do have one:

| kind | verb | scope |
|---|---|---|
| `memory` | `edit` | full field read→mutate→write transaction |
| `backlog` | `edit` | status + resolution |
| `spec` | `edit` | descent/parent scalars |
| `knowledge` | — | none |

The memory corpus fills its fields. The knowledge corpus does not. That
correlation is worth taking seriously as a first hypothesis.

**2. Creation seeds every field empty, at exactly the moment the content is
known.** `knowledge new` scaffolds `[facet]` with all fields present and blank
([[mem.pattern.doctrine.amend-knowledge-both-tiers]]). So does the managed
design run: `CreateRecord` (`src/design_run/submission.rs`) accepts `kind`,
`title`, `slug` and `acceptance` — and no facet — even though the agent
proposing the disposition has the ruling in hand as it writes the checkpoint.
Observed first-hand across `DEC-145`…`DEC-148` in `SL-246`'s design run: four
records minted with the decision fully argued in conversation, four empty facets
hand-patched afterwards. This may be the single highest-leverage fix — capture
at the moment of decision rather than as a follow-up chore.

**3. Nothing validates it.** A `decision` can hold `status = "accepted"` with an
empty `choice` and the corpus reports clean. `doctrine validate` has no signal
here. An advisory warning (not an error) would at least make the gap visible.

**4. The render conceals it.** `format_facet` (`src/knowledge.rs:1302-1364`)
emits the `[facet]` header only when at least one axis is populated, and
`show_opt_line` (`:1286-1291`) drops absent fields silently. An unfilled record
renders as *nothing at all* — not a blank block, not a header. Meanwhile
`facet_json` (`:1432-1462`) emits every field as `null`. The two paths disagree,
and the human one is the concealing one. (`SL-246` `inq-5` decides how the
*composed read* handles this; it does not fix `knowledge show`.)

**5. Skill guidance.** Whether `/knowledge`, `/design` and `/record-memory`
actually instruct an agent to fill the facet, and whether the templates'
seeded-empty shape reads as "optional" — unexamined.

## Adjacent: should knowledge records be TOML-only?

Raised during `SL-246` design (2026-08-05). Unlike most entity kinds, a knowledge
record's facet and body are authored in the same breath. If they remain backed by
separate files, that implies a CLI good enough to hide the fact.

**Provisional answer: no, don't merge — but the second clause is the right
diagnosis.** The split is not causing the empty facets; it is making them
invisible. Collapsing the files fixes the invisibility and leaves the cause
untouched.

Three reasons to resist the merge:

1. **The incentive does not change.** Population is all-or-nothing — 35/35/35 on
   decisions, no partially-filled records anywhere. Authors are not running out
   of steam mid-facet; they never enter the structured tier at all. In one file
   they would fill `body` and skip `choice`/`rationale`/`consequences` for the
   same reason they do now: nothing asks, no verb writes, nothing complains. The
   result is 24% fill in one file instead of two.
2. **The kinds do not want the same shape.** `ConceptFacet {}` has no fields by
   design (`src/knowledge.rs:570-573`) — *"every concept rides its attributed
   prose body"*. `CPT` is prose-primary, `DEC` is facet-primary, `EVD` is nearly
   all facet. One storage decision across seven kinds has to be wrong for some.
3. **Prose in TOML is a real cost.** `QUE-206`'s body is 6.7 KB. Multi-line TOML
   strings diff badly, lose Markdown editor support, and break on an embedded
   `"""`. Knowledge would also become the one kind that reads differently from
   every other — `show` synthesizes two tiers for all of them, `paths` prints
   both, and the publication machinery assumes both.

**What to build instead**, in leverage order — this is the concrete content of
"a better CLI that hides the split":

1. **Capture the facet at mint time** (lead 2). The highest-leverage fix by a
   distance: the ruling exists, fully argued, in the same payload that mints the
   record.
2. **`doctrine knowledge edit`** (lead 1), modelled on `memory edit` — a single
   read→mutate→write transaction over fields. Note which corpus is well
   populated.
3. **Advisory validation** (lead 3).

**The honest counter, unresolved.** If `knowledge edit` ends up needing to write
prose *and* fields in one transaction — likely, since the two genuinely are
authored together — then it performs the merge at the interface layer while
still paying to keep two files on disk. That is a defensible trade (files stay
diffable, Markdown stays Markdown) but it *is* a trade, and it deserves deciding
rather than defaulting. Build the interface first; the storage question is much
easier to answer once nobody is hand-editing either file.

## Not in scope

`SL-246` reads records; it does not fill them. Its non-goal *"not a corpus
facet-hygiene pass — empty facets are rendered honestly, not backfilled"* stands.
This item is the follow-on.

Related: `SL-246`, `ISS-316` (`SPEC-019` governs four record kinds, the corpus
has seven), `IMP-398` (knowledge record discoverability).
