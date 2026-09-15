# DEC-252: The contract walk supersedes serde's message for the three attributed types

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

`SL-259` `PHASE-03`'s unknown-key walk runs **before** deserialisation, so it
answers the unknown-key case for *every* wire type — including
`Declaration`, `CheckpointActDeclaration` and `AgentActDeclaration`, the three
that carry `#[serde(deny_unknown_fields)]` and used to answer for themselves.
A caller no longer sees serde's text for that case.

This was not sought. It is what running the check earlier necessarily does, and
the alternative — exempting the three attributed types so serde still speaks —
was rejected: it would preserve two message shapes for one fault, chosen by
whether a type happens to carry an attribute, which is not a distinction a
caller can see or act on.

## What it supersedes

`SL-251` (`done`) `PHASE-07` `VT-3`, which pinned that an unknown key on
`Declaration` comes back carrying **serde's own text**, and its `VA-1`
companion. That slice's test was rewritten in place to pin the successor
invariant. **If `SL-251`'s `VT-3` is re-run it will read red**, deliberately —
the behaviour it verified has been replaced, and holding its `unknown field`
literal alive to keep a closed slice's grep green would have been false
evidence.

## Why `DEC-225` is intact

`DEC-225` (`SL-251` `sec-6`) is the *remedy rides the point of failure* rule:
serde's own message verbatim, **no paraphrase, no classifier**, then the
contract's address on an indented continuation. Its target is a wrapper that
catches a serde error and re-words it, losing detail.

This is not that. Nothing wraps or re-words a serde error; an earlier check
refuses first, and it carries strictly more than serde did:

| | serde's `deny_unknown_fields` | the walk |
|---|---|---|
| the offending key | ✓ | ✓ |
| what was expected | ✓ | ✓ (the type's full admitted list) |
| **where in the payload** | ✗ | ✓ (dotted path, e.g. `declare[0].cursror`) |
| **which type admitted it** | implicit | ✓ named |
| **coverage** | 3 of 12 wire types | every type in the closure |
| the contract's address | ✓ | ✓ (same indented continuation) |

`sec-8` pin 1 is what makes the substitution safe rather than plausible: it
holds each struct contract's key set equal to a fully populated value's serde
output, so *the expectations the walk prints are the ones serde would have
printed*.

## The ruling: a reconciliation line, not a `REV`

Settled by `SL-259`'s audit (`RV-366` `F-6`), which this record's predecessor
paragraph handed the question to. Three reasons, in order of force.

1. **A `REV` would resolve to no legal write.** The revision kind is the
   governance-dependency vehicle (`ADR-013`) and routes changes to governance
   and spec entities. `SL-251`'s `PHASE-07` `VT-3` is a *plan criterion*, and
   `PHASE-NN` / `EN-` / `EX-` / `VT-` ids are immutable-append — there is
   nothing a `REV` could lawfully rewrite.
2. **No governance artefact changed.** `DEC-225` is intact (see above), and the
   table above shows the walk carries strictly more than serde did, with
   `sec-8` pin 1 holding the printed expectations equal to serde's.
3. **The defect the `REV` argument pointed at is a *linking* failure, not a
   revision failure.** "An auditor re-running `SL-251`'s gate has no in-band way
   to learn why" was true, and it was caused by this record being `proposed`,
   facet-empty and unrelated to anything — not by the absence of a revision. It
   is fixed at a hundredth of the cost.

So, at `SL-259`'s reconcile: this record moved to `accepted`, its facet was
populated from the prose above, and it now carries `references --role concerns`
edges to **both** `SL-251` and `SL-259`. An auditor re-running `SL-251`'s gate
reaches the supersession in one hop.
