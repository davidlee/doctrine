# IMP-380: explore.scope step text is thinner than its gate condition

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The residual of `RV-341` F-1, which was **disposed as design-honoured** — this
is the one improvement that survived that adjudication, and it is a nit.

## Background, so this is not re-litigated

`RV-325` F-16 (round 10) made the classification test an **authoring gate**:
every 2a step states its own truthful completion condition *in its text*.
`RV-325` F-17 (round 11) applied it, found the round-10 "and checkable in run
state" clause contradicted a round-3 owner ruling, and **split the limbs** —
classification asks only for a stated truthful condition; state-visibility moved
to *collection*. The gate was then applied over all nine 2a obligations
(`.doctrine/slice/233/sketches/thin-adapter.md`, "The gate, applied"): all nine
state a condition, all nine survive as 2a, five of nine are state-visible.

Two texts were changed to buy visibility; the rest were left alone deliberately.
PHASE-08 landed exactly that — its entire diff to
`install/design-prompts/exploring.toml` (`78d00a074`) is the one prescribed line:

```
- "Triage the design surface: open questions, risks, …"
+ "Record the design surface triage in the slice notes: open questions, risks, …"
```

So the shipped asset conforms to what the review settled. `RV-341` F-1's blanket
claim — that four of the five steps state acts rather than conditions — is the
**round-10** reading, and those same four steps are literally what round 11's
reductio used to overturn it.

## What actually remains

One row is genuinely thinner than its articulated condition:

| | text |
|---|---|
| shipped `explore.scope` | *"Read the slice scope, the specs and ADRs it descends from, and prior art."* |
| gate table row 1 | *"the slice's governing specs and ADRs can be named, and prior art is **located or its absence established**"* |

"Read X" names an act; the condition names an outcome, and *"or its absence
established"* is the discriminating limb — you can read and still not be able to
name. That is the shape F-16 rule 1 targets.

By contrast `explore.canon` ships *"Run /canon **so the ADRs, policies and
standards governing this surface are in view**"* — the purpose clause is the
table's row-3 condition almost verbatim, so it needs nothing.

## Why it is only a nit

Row 1 is marked **not state-visible**, so the PHASE-09 adherence kit does not
collect it — it names it and reports the fraction (five of nine). No collected
obligation is undelivered, so nothing measures against text the agent never saw.
This is a quality improvement to shipped guidance, not a conformance gap.

## The fix

Enrich `explore.scope`'s text toward its stated condition. Note the cost, which
is real but small: a discharge binds the digest of the whole step definition, so
editing the text makes existing discharges stale and they are re-made
deliberately.

## Method note worth keeping

F-1's measurement was `grep -c "Complete when"` — 0 for `exploring.toml`, 2/2,
1/1, 3/3 for the three runbooks PHASE-08 authored. That measures an **idiom**
the new runbooks adopted and this file never used, not the semantic property the
gate asks for. The finding flagged its own method ("evidence, not proof"); the
caveat was right and the conclusion drawn past it was not.
