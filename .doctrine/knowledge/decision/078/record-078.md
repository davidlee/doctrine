# DEC-078: Prompt projection uses content-addressed fragment receipts

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Every stable fragment emitted by `doctrine design show --format prompt`
carries a canonical fragment name and content digest. A caller may repeat:

```text
--known-fragment <name>@<digest>
```

Doctrine omits the body only when both identity and digest match the current
asset. A changed or unknown digest causes the current fragment to be emitted,
so upgrades and process edits cannot be hidden by a stale receipt.

The receipt is a stateless projection input, not persisted delivery state.
Callers can carry it across turns, compactions, and sessions; losing it merely
causes harmless reinjection. The dynamic TurnEnvelope is always emitted.

The harness-delivered skill definition is not part of per-turn prompt output.
Receipts apply to the invariant stage hymn and the selected coarse process
fragment.
