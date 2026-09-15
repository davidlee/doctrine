# IMP-448: Canonical id format re-spelt at the reservation midpoint

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`kinds/resolve.rs:41` documents `canonical_id` as *"the canonical-id **format**
authority"* (`SL-204 PHASE-04`, the mirror of `parse_ref`'s parse authority). But
`entity.rs:557` builds the same string by hand —

```rust
let name = format!("{id:03}");
…
let canonical = format!("{prefix}-{name}");
```

— rather than calling it. That site is where every minted canonical id actually
comes from: the `on_reserved(id, &canonical)` midpoint, and `entity.rs:558` is the
only invocation of `on_reserved` in the tree.

Two spellings of one format is what `STD-001` is about, and the fact that the
*authoritative* one is not on the mint path makes it worse than cosmetic: a change
to `canonical_id` would not reach the ids the engine mints.

## How it surfaced

Writing `SL-259` `PHASE-06`'s `widest_canonical_id` const proof
(`src/commands/design.rs`), which bounds every mintable canonical id against
`DESIGN_ID_BYTES` so `run::apply`'s pass 2 has no value-dependent refusal. The
proof holds — both spellings produce the same form, single-sourced from the
prefix table — but it can only claim the *form*, not that there is one formatter.
That limit is stated in the proof's own doc comment and should be deleted with
this item.

## Shape of the fix

`entity.rs` calls `kinds::canonical_id(prefix, id)` for `canonical`, keeping
`name` for the directory. Watch for the layering direction before assuming it is
a one-liner — `entity` is beneath `kinds` in some readings, which is plausibly
why the duplication exists.
