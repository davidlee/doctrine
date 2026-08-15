# DEC-225: Contract address pushed into refusal and envelope

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The repo has already argued this, and it argued against fetch-only

`refusal.rs:483` explains why a gate refusal carries remedy text rather than
merely naming the unmet condition. The remedy is a total function of the
condition, *"which is what lets a caller refused at the top edge on a bottom
edge's condition act on it **without fetching anything**."*

That is this slice's problem, stated in the repo's own words, and the answer it
reached was push, not pull. A payload contract that is only fetchable would be a
step back from a standard the neighbouring axis already meets. The question here
is therefore not *whether* to push, but *what*.

## The gap is at the parse site

`design.rs:1504` is the whole of it:

```rust
let request: ApplyRequest =
    serde_json::from_str(payload).context("parse the apply payload as JSON")?;
```

No remedy seam at all. Every payload that fails to deserialise surfaces as raw
serde text under one context line, pointing nowhere.

Both failures observed during this slice's own design run land exactly there:

- an unknown key on `Declaration`, refused by `deny_unknown_fields`
  (`submission.rs:123`);
- `AgentAct`'s external tagging, where a payload written by analogy with its
  internally-tagged sibling `DelegationAct` produced `unknown variant 'act'`.

It is the exact point of failure and currently the least helpful surface on the
path. Wrapping it is one `map_err`.

## Push the address, pull the body

The address is small enough to ride every refusal and every turn. The body is
not, and `DEC-064`'s envelope byte budget is the reason.

The second insertion point is the turn envelope's no-drop region, beside
`DECLARATION_EXAMPLE` (`render/envelope.rs:75`). This reaches a caller *before*
it fails rather than after, on every turn, and no-drop membership means it cannot
be elided by budget pressure.

Both costs are measured rather than estimated. The example is under 320 bytes
against `ENVELOPE_DECLARATION_EXAMPLE_BYTES = 1024`, inside
`ENVELOPE_NORMAL_BUDGET_BYTES = 24576`. A pointer line is roughly 60 bytes. The
contract body never rides the turn, so assumption `A1` — no v1 envelope wire
change — holds.

## The candidate declined, and what would revive it

`DEC-224` deferred the fourth rendering candidate to this question: inject the
full contract into `resume`'s projection, riding `contract_section`
(`commands/design.rs:2343`) and its `--known-contracts` suppression.

Declined. It spends the budget `DEC-064` exists to protect in order to save a
caller one invocation, when a pointer does the same work.

Not foreclosed, and the seam is named so nobody has to rediscover it. If
measurement later shows cold-re-entry agents routinely fail before fetching, this
is the escalation. Note the mechanism would need adapting rather than reusing:
`contract_section` is keyed off stage-advance edges via
`Condition::contract_asset_key`, so a payload contract cannot ride it literally.
The reusable part is the *pattern* — a suppressible block negotiated by a
`--known-*` flag, which is `DEC-124`'s no-digest ruling reaching the wire.

## Downstream

The parse path gains its first remedy seam, so payload deserialisation refusals
join gate refusals in being actionable without a round trip.

`inq-8` becomes concrete rather than hypothetical: the pointer lands beside
`DECLARATION_EXAMPLE`, in the same no-drop region, and that example's known
defect — it omits `cursor` — is now adjacent to new content. `DEC-228` settles it.
