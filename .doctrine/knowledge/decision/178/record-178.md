# DEC-178: One settle verb, reach derived from facet names

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Why the verb exists

`set_record_status` says it plainly — *"No resolution coupling: `status` and
`updated`, nothing else."* That is a deliberate property of a status verb, not a
defect in one. It is also why moving a question to `answered` today writes no
answer, and why `SL-249`'s `OQ-2` was right to reject an `edit --answer` flag:
`edit` is uncoupled field mutation, and bolting a coupled transition onto it
would give one verb two contracts.

`settle` is the coupled transition. `status` stays the uncoupled one.

## The correspondence

| kind | state | `[facet]` evidence |
|---|---|---|
| `QUE` | `answered` | `answer`, `answered_by`, `answered_on` |
| `ASM` | `validated` | `validated_by`, `validated_on` |
| `ASM` | `invalidated` | `invalidated_by`, `invalidated_on` |
| `CON` | `waived` | `waiver_reason`, `waived_by`, `waived_on` |
| `DEC` | `accepted` | — no `accepted_by` |
| `EVD` `HYP` | `confirmed`, `refuted`, … | — no by/on pair ([[DEC-174]]) |
| `CPT` | `active`, `retired` | — no facet ([[DEC-172]]) |

The rule is one line: a state is settleable when its kind's facet carries
`<state>_by` and `<state>_on`. Nothing is enumerated, so nothing drifts when a
kind gains or loses a status token — and that is not a novelty here.
`accepted_status` already derives its answer the same way, and says why: *"if
another kind gains the token, this follows without an edit."*

## The exclusion that corroborates it

`DEC` falls out because `accepted` has no `accepted_by`; its actor pair is
`decided_by`/`decided_on`, named for the proposal rather than the acceptance.

That could have been an accident of naming. It is not, and the check is
`DEC-088`: a decision reaches `accepted` through a content-bound user
attestation on a design-run checkpoint, applied by `apply_record_effects`. A
hand `settle` writing `accepted` would be a second route to that state with none
of the attestation the first route exists to carry. The derivation and the
governance reach the same answer from unrelated directions, which is the best
available evidence that the derivation is tracking something real.

`decided_by`/`decided_on` remain writable — through objective 1's decision facet
subverb, where they belong, as ordinary fields rather than as the evidence of a
transition.

## Shape at the call site

```
doctrine knowledge settle QUE-198 answered  --by david --answer "…"
doctrine knowledge settle CON-012 waived    --by david --reason "…"
doctrine knowledge settle ASM-004 validated --by david
```

The id resolves the kind ([[DEC-173]]'s point — per-kind dispatch relocates the
kind-mismatch check rather than removing it), the state is validated against
that kind's vocabulary by the check `set_record_status` already performs, and
the date comes from `clock::today()`. A state with no `<state>_by` is refused
with a message naming `knowledge status` as the verb that moves it.
