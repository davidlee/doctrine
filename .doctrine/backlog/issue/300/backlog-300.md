# ISS-300: Deferring a node records no reason

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The asymmetry

A node leaves `open` by one of two routes, and only one of them keeps its
rationale.

**Resolution cannot forget.** It goes through `disposition`, and DEC-062's four
forms are exhaustive over what the outcome was: `create` and `adopt` bind a
durable record, `unresolved` and `non-durable` carry a **mandatory `note`**. The
type comment says why the note-bearing pair exists at all — *"without them a
resolved node that produced no record is unrepresentable, and 'we discussed it'
gets laundered into a durable claim."*

**Deferral and pruning carry nothing.** Both ride `Declaration.lifecycle`, a bare
`Option<InquiryLifecycle>` over `Open | Resolved | Deferred | Pruned`
(`src/design_run/submission.rs`). There is no adjacent reason field and no
admission check. A node can leave the frontier permanently, and the map records
only that it left.

So the machinery that makes resolution honest is absent from the two moves that
are *more* lossy: a resolved node at least points at a record, while a deferred
one points at nothing and a pruned one is gone.

## Observed

SL-243's design run (`dr-019fc13a`) deferred `inq-1`–`inq-3` at revision 8 —
the governance-placement cluster, routed to a `/spec-coverage-assessment` pass.
That routing is real and load-bearing, and it exists **only** in
`.doctrine/slice/243/notes.md` § Routing.

It surfaced because the run took a context break at revision 9. The continuation
prompt written for the fresh agent had to carry the rationale by hand, and said
so in as many words:

> `inq-1`–`inq-3` are deferred to a `/spec-coverage-assessment` pass, not parked.
> A lifecycle move has no reason field, so `.doctrine/slice/243/notes.md`
> § Routing is the only record of where they go.

That is the failure in one line: **prose has to shadow the map**, so the map
cannot be the record. A resuming agent reading run state alone sees three nodes
in `deferred` and cannot distinguish *routed to a named downstream pass* from
*parked* from *abandoned*.

## Why this is the same shape as ISS-299

ISS-299 is that the map never reaches the user. This is that the map does not
hold enough to be worth reaching them with. Both leave the durable spine thinner
than the prose beside it, and both push the real content back into the
conversation the run exists to outlive.

## The fix has two precedents in this same file

Neither needs inventing — the codebase already does this twice, and both times
for the same reason:

| site | rule |
|---|---|
| `DischargeDeclaration.reason` | *required* when the claim is `Skipped`, **checked at admission rather than by serde**, so the failure is a typed `Refusal` naming the step |
| `StageDeclaration.reason` | a *backward* stage move carries its reason (DEC-067); the field is `Option` only because a forward move has none |

Both encode: the lossy direction states why, the cheap direction does not have
to. `deferred` and `pruned` are the lossy direction of a lifecycle move and are
the third instance of the pattern.

Suggested shape, following `DischargeDeclaration` exactly: a `reason` beside
`lifecycle`, required at admission for `Deferred` and `Pruned`, refused with a
typed `Refusal` naming the node when absent. `Open` (reopening) and `Resolved`
(already covered by `disposition`) need nothing.

Then render it: a deferred node's reason belongs wherever deferrals surface, or
the field is written and never read — the failure mode ISS-299 already documents
for `parent`.

## Provenance

Found while moderating CHR-049's measurement exercise against SL-243, from the
continuation prompt's own account of what the run could not hold. The claim was
verified against `src/design_run/submission.rs` and
`src/design_run/inquiry.rs` rather than taken from the prompt.
