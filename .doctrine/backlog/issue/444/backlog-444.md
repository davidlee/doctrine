# ISS-444: Canonical id form asserts exactly three digits; the namespace ends at 999

Every numbered kind addresses 999 entities. `SPEC-013` fixes the canonical id form
as *prefix + zero-padded-to-three* and is the single id-form authority, so the
bound is one decision in one place — and so is the fix.

**How close it is.** Per-kind high-water marks today:

| kind | max | kind | max |
| --- | --- | --- | --- |
| `ISS` | 444 | `SL` | 257 |
| `IMP` | 442 | `DEC` | 242 |
| `RV` | 362 | `RSK` | 232 |

Backlog issues are 44% of the way and grow fastest — this corpus mints them
several per session. Not urgent; not hypothetical either, and the sweep is the
expensive part rather than the decision.

## The good news, established first

Rust's `{id:03}` is a **minimum** width, not a fixed one. `format!("{:03}", 1000)`
is `"1000"`. So nothing truncates, collides, or corrupts at the boundary: the
writer emits `SL-1000`, mints `.doctrine/slice/1000/`, and the corpus simply goes
**mixed-width**. There is no flag day and no data loss waiting at 999.

The cost is entirely in the readers that assert *exactly* three.

## What actually breaks

**Verified — `src/concept_map.rs:738`:**

```rust
let Ok(ref_re) = Regex::new(r"^[A-Z]{2,5}-\d{3}$") else {
```

Anchored to exactly three. `SL-1000` stops matching, so the `EntityRefLike`
diagnostic — which exists to catch a concept-map node label that is really an
entity ref — silently stops firing for every id past the boundary. A check that
quietly narrows its own coverage is `STD-003` territory.

**Prior art that the hazard is real — `src/relation_graph.rs:1983`:** a test
already plants `SL-998`, `SL-999`, `SL-1000`, `SL-1001` out of order, with the
comment *"Lexical sort would give `["SL-1000","SL-1001","SL-0998","SL-0999"]`"*.
Someone hit width-boundary ordering there and fixed it. **One** site is defended
and tested at the boundary; nothing establishes that the others are.

**Documentation asserts three** in at least five places, one of which rides every
agent's context: `install/glossary.md:43`, `.doctrine/glossary.md:36`,
`install/routing-process.md:64` (and therefore the boot snapshot), `SPEC-013`'s
responsibilities, and `spec-013.md:88`.

## Checked and NOT broken

Recorded so the sweep does not re-derive them:

- `src/listing.rs:57` `parse_ref` — strips the prefix and `parse::<u32>()`s the
  rest. Width-agnostic already, and the reason `SL-1000` would round-trip.
- `src/doctor_checks.rs:294` — `[A-Z]{2,}-[0-9]+`, unbounded.
- `src/meta.rs:201`, `src/integrity.rs:82` — sort `u32`, not strings. Ids reach
  `u32` early nearly everywhere, which is why lexical ordering is a narrower
  hazard than it first looks.

## The framing: widen the readers, never re-pad the corpus

The obvious move — re-pad everything to four digits — is the wrong one, and it is
worth writing down before someone reaches for it.

Ids are **immutable** (the boot guardrail; `STD-002`). Re-padding `SL-023` to
`SL-0023` changes identity: every prose citation in every slice, ADR, spec,
memory and commit message goes stale at once, along with every git-anchored
memory and every directory basename. It buys a tidy `ls` and spends the corpus's
whole referential integrity.

So the target is **not** a migration to four digits. It is:

1. keep `canonical_id` padding to three as a **minimum**, so every existing id
   stays byte-identical forever;
2. find and fix everything that asserts exactly three — the sweep;
3. accept a mixed-width corpus at the boundary as the correct outcome, and say so
   in the documentation that currently promises three.

## Shape of the work

The sweep is the item. Grep alone is not sufficient — `\d{3}`, `{:03}` and
`[0-9]{3}` find the literal cases, but an off-by-width assumption also hides in
fixed-width column formatting, golden test fixtures, sort comparators over
`String` ids, and anywhere a display path slices a ref by byte offset. Worth a
deliberate audit rather than a regex pass, and worth a boundary test in the shape
of `relation_graph.rs`'s — mint past 999 in a fixture corpus and assert the
readers still see it.
