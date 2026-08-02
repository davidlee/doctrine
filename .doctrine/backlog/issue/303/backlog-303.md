# ISS-303: Reopening an inquiry discards its disposition

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

`InquiryLifecycle::Resolved → Open` is a legal transition, and it drops the
node's disposition. `src/design_run/tests.rs:353` pins the behaviour:

```rust
let reopened = resolved.transition(InquiryLifecycle::Open).expect("legal");
assert_eq!(reopened.disposition(), None);
```

DEC-062 exists so resolving a node cannot forget *how* it was resolved:
`create`/`adopt` bind a durable record, `unresolved`/`non-durable` carry a
mandatory note. Reopening discards that whole assertion. The node returns to
`open` holding nothing about the answer it previously had.

The knowledge record survives — DEC-111 remains a corpus entity — but the map's
link to it does not, so nothing connects a reopened question to the answer it is
being asked to reconsider.

## Why it matters more than it looks

Reopening is exactly where prior context is most valuable. An agent revisiting a
question it has already answered should be asked **"does this still hold?"**, not
asked cold. Asked cold it re-derives, and may re-derive differently for no reason
anyone can see — the change then reads as a decision when it is an artefact of
forgetting.

That is DEC-062's own stated failure arriving through the back door. Its
rationale warns that *"'we discussed it' gets laundered into a durable claim"*;
here the laundering runs the other way, and a durable claim is silently demoted
to never-discussed.

## Prior art

hydra (RFC-026 E8.5) keeps a single `prior` answer on the head and re-presents it
on reopen, under *"re-present, don't re-ask"*. One slot, not a stack — superseded
history is git's job. That shape fits here: the change log already carries full
history, so the node needs only enough to frame the next question.

## Shape of a fix

Retain the superseded disposition when a node returns to `open` — one slot,
replaced on each subsequent resolution — and surface it wherever the node is
presented for re-answer. Without the surfacing half it is a field written and
never read, which is ISS-299's failure mode and `parent`'s.

The current behaviour is pinned by a test, so reversing it is a design decision
to be recorded, not a patch to be applied.

## Provenance

Found auditing SL-233's shipped design run against an independent implementation
of the same idea. Argument and instrument: **RFC-026 E8**.
