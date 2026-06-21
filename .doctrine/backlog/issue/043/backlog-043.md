# ISS-043: knowledge populated_record_round_trips_into_shared_meta fails on trunk — tags dropped in round-trip

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`cargo test --bin doctrine populated_record_round_trips_into_shared_meta` FAILS
on trunk (reproduced at `8e2c9e12`, edge==main at the time):

```
test knowledge::tests::populated_record_round_trips_into_shared_meta ... FAILED
assertion `left == right` failed
  left:  Meta { id: 7, slug: "token-expiry", ..., tags: ["auth", "security"] }
  right: Meta { id: 7, slug: "token-expiry", ..., tags: [] }
```

The round-trip into shared `Meta` **drops tags** (expected `["auth","security"]`,
got `[]`).

## Scope / provenance

- **Pre-existing, unrelated to SL-133 PHASE-02.** Surfaced while baselining the
  dispatch funnel verify for the risk-leaf extraction — it is red at base `B` and
  red after the phase delta, identically, so behaviour-preservation held.
- The knowledge round-trip's tag persistence is the suspect (write or read path
  in `src/knowledge.rs`). Likely related area to [[IDE-009]] (knowledge
  read-path validation).

## Next

Decide owner: is this a real regression in knowledge tag persistence, or a stale
test expectation? Reproduce, then fix the offending side (serialize/deserialize
of `tags` into shared `Meta`) or correct the assertion.
