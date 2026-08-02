# IMP-388: Structured rejected alternatives

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Rejected alternatives are recorded everywhere in this corpus and structured
nowhere. They live in ADR prose, in DEC record bodies, in slice notes, in
research artefacts — as sentences, in whatever shape the author chose, under
whatever heading seemed right.

That is consistent with doctrine's storage rule (structured data in TOML, prose
in MD), so this is a deliberate inheritance rather than an SL-233 oversight. It
still has a cost, and the cost is now measured.

## What it cost, concretely

CHR-049's moderator had to adjudicate whether `explore.triage` had been
discharged prematurely. Its stated condition names five items, one of which is
*shaping decisions*. SL-243's notes carried four under headings of their own name
and covered the fifth under `### The three carried questions, and what the
evidence recommends` — which does decide things, and does name a rejected
alternative for one of its three items, by pointing at `research.md`.

Whether that satisfies the condition is a judgement call about prose, and the
moderator recorded it as the one marginal call at that edge. **With rejected
alternatives as data the question is a query, not a judgement.** The measurement
sheet's own first pass misquoted the heading in the favourable direction, which
is precisely the fragility of assessing structure by reading prose.

## Prior art

hydra (RFC-026 E8.5) hangs `rejected: [{option, why_not}]` off the answer —
`option` and `why_not` both required. Small, cheap, and it makes "what did we
consider and discard?" answerable without reading anything.

## Shape

The natural home is `CreateRecord` / the disposition payload, so a decision minted
through a design run carries its rejected set as data and the prose body
elaborates rather than carries. Two things to settle:

- **Does it belong on the knowledge record too**, or only on the checkpoint that
  minted it? A DEC authored by hand outside a design run has the same need.
- **Migration is not required.** Existing prose stays prose; this is additive, and
  a partially-structured corpus is still more queryable than none.

Note the tension with the storage rule worth naming rather than eliding: rejected
alternatives are *rationale*, which the rule puts in prose. The counter is that
`{option, why_not}` is a record with fields, not narrative — the same argument
that puts `needs` and `provenance` in TOML.

## Provenance

Argument and instrument: **RFC-026 E8**, and the corrected triage call in
`.doctrine/backlog/chore/049/exercise/moderator-sheet.md` § 6a.
