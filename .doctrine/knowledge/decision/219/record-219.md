# DEC-219: Act contract carries wire shape and engine semantics, not narrative

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Which register the contract joins

Two tables already describe parts of the `design apply` payload, and they sit at
different registers.

`ApplyRequest::WRITER_ACTS` (`submission.rs:989`) pairs each of nine top-level
acts with the predicate that detects its presence. It is inventory plus
behaviour, and nothing else. `Declaration::WIRE_KEYS` (`submission.rs:563`) goes
further: each key carries the *subject kind that honours it*, which is the fact
that made `ISS-318`'s refusal nameable rather than a shrug. On the neighbouring
axis, `SL-244` split its contracts across two tiers — structure in a const table
(`DEC-123`), narrative in sealed prose assets (`DEC-122`).

So a payload contract row could plausibly be any of three things: shape only,
shape plus engine semantics, or shape plus a per-act narrative asset on
`DEC-122`'s pattern. The question is not stylistic. Each answer implies a
different amount of machinery.

## Shape alone does not cover the measured cost

RFC-026 `E8.7` counted 33 source reads against 52 `doctrine design` calls over
`CHR-049`'s run, and 15 of the 33 were payload-shape lookups. Reading those 15
back, they are not questions of the form *does a `traversal` key exist*. They are
questions of the form *what may `posture` be*. Knowing that `traversal` is
accepted and takes a `posture` field says nothing about `posture` admitting only
`breadth` and `depth`, and it is the second question that sends someone to
`submission.rs`.

This slice's own design run produced two more instances of the same class, both
costing a round trip:

- `AgentAct` (`attestation.rs:671`) is **externally** tagged, so the wire form is
  `{"act":{"blocking-set-declared":{...}}}`. Its sibling `DelegationAct`
  (`submission.rs:877`) is **internally** tagged on `tag = "act"`, and
  `CheckpointActDeclaration.act` (`submission.rs:825`) is a bare `ActKind` enum.
  Three sibling act types, three taggings, discoverable only from source.
- A key on `Declaration` was refused by `deny_unknown_fields`
  (`submission.rs:123`) with no indication of what the accepted set was.

Neither would have been prevented by narrative. Both are prevented by a row that
states the enum's token vocabulary and how the type is tagged.

## The narrative option is deferred, not rejected

Shape plus semantics plus a sealed narrative asset per act — the full `DEC-122`
treatment — buys a second asset family, and with it the sealing question and
`IMP-372`'s override seam, for value no evidence yet demands. Every measured
failure is a shape or vocabulary question.

The deferral is cheap to reverse, and `DEC-123` records why: the const table is
the one place in this design where adding structure later is genuinely free —
compile-time data, never serialised, no wire version, no snapshot migration, no
override seam. An asset address can join a const row the day a client project
demonstrates it needs one. Building it now would be the untested speculation
`DEC-123` declined on its own axis.

## What this costs downstream

The contract must enumerate enum token vocabularies and tagging style, not merely
field types. That is a heavier obligation for the drift pin than a top-level
serde key set, because a variant set is not reachable from one serialised value —
which is the constraint `DEC-221` inherits and answers.

A client project with no source gets shape and semantics but no narrative. If
that proves insufficient, `DEC-122`'s asset pattern is the recorded remedy.
