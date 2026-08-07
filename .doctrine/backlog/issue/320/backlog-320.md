# ISS-320: Re-adopting an edited design.md needs a section map nothing emits

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`adopt_authored` is the sole lawful crossing of an authored-watermark divergence
(`DEC-092` rule 2) — the protocol an agent must follow after hand-editing
`design.md` during a managed design run. It is reachable in principle and
awkward in practice, because **the payload it requires cannot be produced by any
`doctrine` verb.**

## What happens

Hand-edit `design.md`, then `doctrine design materialise`:

```
Error: design.md has been edited outside this run — the watermark says
`ec973c1e…` and Doctrine reads `61dd11f2…`. Ordinary mutation is refused
against prose the snapshot no longer describes; re-adopt the document with an
`adopt_authored` declaration naming its exact current fingerprint.
```

The remedy named is *"naming its exact current fingerprint"*, and the refusal
helpfully prints both fingerprints. Follow it literally and the next refusal is:

```
Error: adopt_authored's marker map is not complete and exact:
9 missing, 0 unknown, 0 mismatched
```

`AdoptAuthored` (`src/design_run/submission.rs`) carries **two** fields: the
whole-document `fingerprint`, and `sections` — *"the complete stable-marker map:
every section Doctrine knows, and no other, each naming the fingerprint the
caller read for it"*. The first refusal does not mention the second field, and
the second refusal reports only counts, so it names neither the missing section
ids nor the digests they need.

## Why it is a real block, not a papercut

There is no verb that emits those digests.

- `doctrine design show` / `resume` print each section's fingerprint **from the
  snapshot** — that is, the *stale* values, which are precisely the ones the
  adoption must not carry. Copying what `show` prints reproduces the divergence
  the adoption exists to cross.
- Nothing else prints the *read* digests.

So the only route is to recompute them from the parser's own rules, off the
source: sections are delimited by `<!-- doctrine:section sec-N -->` marker lines;
a section's body runs from just past its marker line to the byte before the next
marker (or EOF), with the trailing newline stripped and `per_line(…,
unescape_line)` applied; the digest is `sha256` of that body
(`src/commands/design.rs::authored_sections`, `src/design_run/document.rs::parse`).

An agent that reads the source can do this. That is the wrong bar for a
documented recovery path, and it is a guessing exercise of exactly the kind the
project's *use the CLI, don't guess* guardrail forbids.

## Observed

SL-248, design run `dr-019fd432-6e5e-7282-b425-19f07630a627`, 2026-08-08, while
remediating `RV-346` round 6 across seven of nine sections. Recomputation
worked and the adoption landed at revision 76 — verified correct because the
whole-document digest matched Doctrine's own reading and the two untouched
sections (`sec-3`, `sec-4`) reproduced their snapshot fingerprints exactly. That
cross-check is available only by luck: it needs at least one unedited section.

### Second crossing, 2026-08-08 — two additions

Same run, remediating `RV-346` rounds 7–9 (`F-30` then `F-38`), crossed twice
more at revisions 77 and 79.

**A wire detail this issue did not record, and it costs a cycle.** The envelope
is **snake_case** — `run_uid`, `known_revision`, `submission_id`,
`adopt_authored`. Neither `ApplyRequest` nor `SubmissionEnvelope` carries a
`serde(rename_all)`, while several sibling enums in the same module *are*
kebab-case, so the surrounding vocabulary actively misleads. A payload built
with kebab keys is refused with `missing field run_uid` — which names one field
and not the convention, so the natural repair is to fix that key alone and be
refused again. This is the same unactionable-refusal defect as fix 3 below,
reached from a different direction.

**The luck in the cross-check is removable.** The recomputation can be validated
*before* it is relied on: run it against the **unedited** document and require
all nine digests to reproduce the snapshot's. That is a positive control over
the whole algorithm — marker recognition, region bounds, the single trailing
newline strip, `per_line`/`unescape_line`, and the digest — and it needs no
unedited section afterwards, because it is performed while every section is
still unedited. Only then edit, recompute, and check that exactly the sections
you touched moved. Both crossings did this and the moved set matched the edit
set each time (rev 77: `sec-7`; rev 79: `sec-7`, `sec-8`).

Worth stating because the original note reads as though the cross-check is a
happy accident of a partial edit. It is not: it is available on every crossing,
it is stronger than the post-hoc version, and an agent that skips it is trusting
a hand-rolled reimplementation of a parser it never tested.

Note also that `unescape_line` is the identity unless the document contains an
*escaped* marker (`doctrine::section`, two or more colons — `shape` rejects a
one-colon line via `colons < 2`). Real markers are single-colon, so on an
ordinary `design.md` the transformation does nothing — but that is a property of
the document, not of the algorithm, and the control above is what establishes it
rather than assuming it.

## Sketch of a fix

Cheapest first; any one of these closes it.

1. **A verb that emits the payload.** `doctrine design adopt <slice>` — reads
   the document, computes the map, applies the adoption. The declaration form
   stays for programmatic callers. This is the honest fix: adoption is a
   deliberate act, and requiring the human to transcribe nine digests adds
   ceremony without adding deliberateness.
2. **Emit the map, don't apply it.** A `--emit-adoption` on `design show` that
   prints the ready payload for review before it is fed back. Preserves the
   "look before you cross" property the two-step shape is presumably reaching
   for.
3. **Fix the refusals, at minimum.** `AdoptionStale` should name the `sections`
   field, and `AdoptionMarkersInvalid` should list the ids and their read
   digests instead of counting them. This is strictly a mitigation — it turns
   an unguessable payload into a transcribable one — but it is small.

Note 1 and 3 are not alternatives: even with an `adopt` verb, a refusal that
reports `9 missing` and no ids is unactionable for anyone hand-driving the
declaration.

Related: `DEC-092` (authored watermark guards the authored tier), `DEC-072`
(design sections are first-class runtime records), [[mem.signpost.doctrine.review]].
