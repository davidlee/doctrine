# HYP-001: HYP is unused because no skill routes to it

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Why this record exists

Raised during SL-249's design run while ruling `inq-6` (what facet contracts
`EVD` and `HYP` should carry in governance). The research round had flagged
*"`HYP` has n=0 in this corpus, so its facet contract has no usage precedent
beyond the code struct"* as a limit on the ruling — and a limit is not an
explanation.

It is also its own first instance: `HYP-001` is the corpus's first hypothesis
record, and it exists because someone asked why there were none.

## The measurements

Taken 2026-08-06 against the shipped `.agents/skills/` corpus and
`.doctrine/knowledge/`.

**Corpus counts.** `EVD` 26 · `CPT` 2 · `HYP` 0.

**Mentions of each kind across all shipped skills.** evidence 68 · question 67 ·
decision 66 · assumption 31 · constraint 23 · hypothesis 22 · concept 18.

Read naively that says `HYP` is *reasonably* mentioned, which is what makes the
naive reading wrong. Of 30 `hypothes*` hits across the whole skill corpus, **26
are in `/rigour`**, where the word is used in its ordinary-English
differential-diagnosis sense — *"Enumerate at least two live hypotheses"*,
*"Predict each hypothesis's expected result first"*, *"Observe, update, prune,
repeat"*. None of them refers to the record kind. The only appearance of the
`HYP-` prefix anywhere in the corpus is inside a list explaining how id prefixes
resolve to kinds.

**Mention count does not predict usage.** `CPT` has the most `/knowledge`
mentions of any kind (4) and 2 records. `EVD` has 3 mentions and 26 records.
Whatever drives capture, it is not how often a kind is named.

## The proposed mechanism

Capture follows *routing* — a skill naming a kind as the destination for a
specific kind of work — not vocabulary. The handoffs that exist all enumerate
the four kinds that existed when they were written:

| skill | handoff |
|---|---|
| `/harvest` | "→ DEC / QUE / ASM / CON" |
| `/design` | "an open question → QUE, a locked choice → DEC, an assumption the design carries → ASM" |
| `/preflight` | "capture as ASM" |
| `/consult` | "a resolved tradeoff → DEC" |

`SL-159` added `EVD` and `HYP`; `SL-197` added `CPT`. Neither updated the
handoffs. So the behavioural prompts still describe a four-kind world — the same
four-kind world `PRD-010` and `SPEC-019` still describe, which is `ISS-316`'s
gap showing up one tier further out, in the prompts that actually drive agent
behaviour rather than in the specs that document it.

`EVD` accumulated 26 records anyway, so routing is clearly not the only path to
capture; agents and operators reach for evidence directly during research and
review. That is a real complication and is why the proposition is scoped to
`HYP` rather than asserted as a general law.

## Relation to the slice

`SL-249` fixes the *mechanical* half of the capture problem — no write seam, and
a create path with no slot for content. This is the *behavioural* half:
a slot nobody is told to fill stays empty for a different reason.

That half is `IMP-403` lead 5 (*whether `/knowledge`, `/design` and
`/record-memory` instruct an agent to fill the facet at all*), which `SL-249`
records as an explicit non-goal. This record is the testable form of that lead,
not a claim `SL-249` acts on.
