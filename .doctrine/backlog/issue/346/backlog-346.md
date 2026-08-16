# ISS-346: design apply silently absorbs unknown payload keys

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`doctrine design apply` deserialises its payload into `ApplyRequest`
(`src/design_run/submission.rs`), which carries no `deny_unknown_fields` at
either level. A payload key the schema does not hold is dropped by serde and the
submission proceeds: the admission passes, the revision bumps, a receipt is
written, and **no event is emitted and no state changes**. The caller is told
`revision N stage <stage>` — indistinguishable from a successful mutation.

The same holds one level down, inside `Declaration`: a misspelled inner field is
absorbed the same way.

## Why it bites

An agent that does not already know the mutation vocabulary cannot learn it from
the tool. `design show`'s `declaration_example` advertises only `declare` (node
creation) and `traversal`; there is no `dispose` in it, and no `--help` text
carries the payload schema. So the discovery loop is *guess a key, get a
success-shaped response, believe the guess was wrong for some other reason*.

Observed on SL-253: two probe payloads (`{"resolve":[{"subject":"inq-1"}]}`, and
the same carrying a deliberately bogus field name) were both absorbed. The
design run sat two revisions ahead of its own decisions with `resolved=0`, and
the agent correctly stopped rather than keep guessing — but only because
`/design`'s degradation rule said so, not because anything refused. Recorded as
observation `019ff42b-2655-7aa3-b042-0a83272a328c`.

## Shape of a fix

Two halves, either useful alone:

1. **Refuse the unknown key.** `ApplyRequest` uses `#[serde(flatten)]` for the
   envelope, and serde's `deny_unknown_fields` is incompatible with `flatten` —
   so this is not a one-attribute change. Either un-flatten the envelope, or
   deserialise to `serde_json::Value` first and diff the key set against a
   single-sourced field list (`WRITER_ACTS` already enumerates the run-level
   writer fields for a related reason, so a second hand-maintained list would be
   the thing to avoid). `Declaration` has no `flatten` and can take the
   attribute directly.

2. **Refuse the no-op submission.** A payload that admits, changes nothing, and
   emits no event should arguably not advance the revision at all — a
   *nothing-declared* refusal would have caught both probes even without (1),
   and is the cheaper half. Careful with `Admission::Resumed`, which is a
   legitimate no-advance path already.

Worth considering alongside: widen `declaration_example` (or add a
`--format schema`) so the disposition and other run-level acts are discoverable
from the envelope rather than from the source. That is arguably the root cause —
(1) and (2) turn a silent wrong turn into a loud one, but only discoverability
stops the guessing.

## Closed as a duplicate of ISS-333 (2026-08-16)

`SL-251`'s reconciliation audit (`RV-362` `F-9`) found this item and `ISS-333`
recording the same defect — `design apply` discarding an unknown **top-level**
key, bumping the revision, writing a receipt, printing no change row and exiting
0 — under two ids, both open, so the slice's discharge statement had two places
to land and no way to choose. `ISS-333` is the survivor: it is the older split
(out of `ISS-318` at `SL-249`'s close), it already carries the serde `flatten` /
`deny_unknown_fields` constraint that makes this structural, and it is the id
`SL-251` is linked to.

The third suggestion above — *widen `declaration_example` or add a
`--format schema` so the shape is discoverable from the envelope rather than the
source* — is the one thing here that shipped. `SL-251` delivered it as
`doctrine design contract [--format json|prompt]` plus the published
`reference/design-payload-contract.md`. The two refusal candidates (1) and (2)
did **not** ship and stay open on `ISS-333`.

One correction worth carrying across, because this item states the opposite of
what the code does: **`Declaration` already denies unknown fields**
(`submission.rs:123`), as do `CheckpointActDeclaration` and
`AgentActDeclaration`. Three of the twelve wire structs refuse; the other nine
discard. So a misspelling *inside* a `declare` entry is refused with serde's own
message, while the same misspelling one level up is swallowed — and that
asymmetry is the whole of `ISS-333`. (`RV-362` `F-10` corrected the same
falsehood in `mem.fact.design-run.apply-payload-vocabulary`.)
