# IMP-387: Record which decision mooted a node

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Answering one question routinely makes a sibling question meaningless. Deciding
the report reads the corpus in-process (DEC-111) moots every question about how
it would consume the JSON contract. The map has no way to say that.

The available moves are `pruned` — a bare lifecycle token with no reason at all
([[ISS-300]]) — or `resolved` with a disposition that misrepresents what
happened, since the node was never actually answered on its own terms.

So the causal link between one decision and the questions it retires is not
recorded anywhere, and a later reader cannot distinguish *this question was
settled by a decision elsewhere* from *this question was dropped*.

## Distinct from ISS-300

ISS-300 is that a lifecycle move carries no **reason**. This is that it carries
no **edge**. Free text saying "mooted by the in-process decision" would satisfy
ISS-300 and still leave nothing queryable, nothing that follows a rename, and
nothing that lets `DEC-111` show what it retired.

Doctrine already treats this class of link as first-class everywhere else — a
decision that supersedes another gets a relation, not a sentence.

## Prior art

hydra (RFC-026 E8.5) calls it **cauterisation**, and the modelling choice is the
interesting part: it is *not a state*. The head is marked `answered` with
`answer.cauterised_by = <slug>` and `answer.text = "cauterised"`, and
deliberately does **not** cascade to its children. Its stated principle:
*"cauterisation is answer, not deletion — killed branches remain recorded with
rationale for future reference."*

That reading is worth taking seriously here. It says a mooted question *has* been
answered — the answer is just "by that decision over there" — which fits DEC-062's
insistence that leaving `open` always carries a semantic disposition. A fifth
`Dispose` form (`mooted-by`) may be a better fit than a new lifecycle state.

## Shape

A `Dispose::MootedBy { node, reason }` variant, or equivalent, plus the reciprocal
read: the decision record should be able to show what it retired. Suppression of
cascade for this form specifically, per hydra's note — a question killed by a
sibling's answer does not invalidate its own children the way a re-answer does.

## Provenance

Argument and instrument: **RFC-026 E8**. Observed in SL-243's run, where
`inq-1`–`inq-3` were deferred to a downstream pass rather than mooted, and the
routing rationale survives only in `notes.md` prose.
