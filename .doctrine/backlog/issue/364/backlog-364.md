# ISS-364: RecordKind::ALL is hand-maintained, not enum-derived

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`RecordKind` is declared at `knowledge.rs:60-68`. `RecordKind::ALL` separately
enumerates its seven variants as a hand-written `[RecordKind; 7]` at
`knowledge.rs:159-169`, a hundred lines away.

Adding an eighth variant is a compile error at `facet_fields` — whose match over
`RecordKind` is exhaustive — so the new kind is forced to acquire a facet row.
**Nothing forces it into `ALL`.** Every consumer that iterates `ALL` rather than
matching on the enum then silently omits the kind.

The incumbent test cannot catch it either.
`record_kind_from_prefix_round_trips_each_kind` (`knowledge.rs:3334-3341`)
iterates `ALL` and asserts its members have seven distinct prefixes — an
assertion that stays green over a stale `ALL` by construction.

## Why it matters

`ALL` is described in its own doc comment as "the single source for the
cross-kind `list` read (each tree in turn) and the prefix round-trip", so a
dropped kind is a kind that silently stops being listed. `doctor`'s
inert-facet-key check iterates it too (`doctor_checks.rs:167`).

## How it surfaced

`RV-357` `F-12` — an external adversarial review of `SL-251`'s design, which had
claimed the iteration was covered by `facet_fields`'s compile barrier. It is not.
`SL-251` states the barrier at its real strength instead and routes around the
gap: `sec-8` pin 5 takes its kind-set oracle from an exhaustive match over
`RecordKind` rather than from `ALL`. That protects the payload contract and
leaves the underlying list unfixed, which is this item.

## Shape of the repair

Derive the iteration from the enum rather than restating it, so a new variant
cannot be omitted. The cheapest instrument already exists in this repo as a
pattern: a dead exhaustive match that makes the omission a build failure
(`condition_vocabulary!` in `gate.rs`, and the `payload_variants!` `SL-251`
designs on the same principle). A `strum`-style derive is the other option and
carries a dependency question.

Whatever the mechanism, the acceptance is the same: adding a variant to
`RecordKind` and building must fail until the iteration authority includes it.

## Related

- `SL-251` — surfaced it; routes around it rather than repairing it.
- `STD-001` — the single-source principle `ALL` is meant to serve and does not.
