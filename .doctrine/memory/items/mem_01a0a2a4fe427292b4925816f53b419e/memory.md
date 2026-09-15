# Change-log rows degrade; everything else still refuses

`SL-259` `DEC-249` drew the line by **what a token is for**, not by which type
holds it:

- **A token that records what happened** — a change-log row and its terms —
  degrades visibly. The row is retained with its raw table and disclosed.
- **A token that constitutes current state** — `Stage`, `IdKind`, `ActKind`,
  and the delegation group's stored `Declaration`s — still **refuses**. A
  snapshot whose stage cannot be read is not a degraded row; it is an unusable
  run, and tolerating it turns a loud failure into a silent wrong-state one.

Do not widen the tolerance past change-log rows. `IMP-446` carries the separate
(and different) repair for stored declarations.

## The shape, in `change_log.rs`

```rust
enum StoredRow { Read(ChangeRow), Unreadable(RawRow) }
struct RawRow { revision: u64, index: u32, raw: toml::Value, why: Unreadable }
enum Unreadable { Event, PayloadKey, ValueKind, TermTooLong }
```

- `ChangeLog.rows` is `Vec<StoredRow>`; `since()` returns `Vec<&StoredRow>`.
- `record()` still takes `Vec<ChangeRow>` and wraps into `Read` **inside**: a row
  this binary just built is readable by construction, so only deserialisation
  can produce the opaque arm. Both production call sites are unchanged
  (`run.rs`'s `apply`, and `commands/design.rs`'s materialisation revision,
  which records an *empty* row vector — that is why the floor is recorded and
  not inferred).
- `revision` and `index` stay **required** on `RawRow`. The tolerance is for
  vocabulary, never for a row that cannot be placed; a row missing either still
  refuses the parse.

## Two consequences that bite

1. **`DesignSnapshot` and `Applied` are `PartialEq` but not `Eq`.** `toml::Value`
   carries `Float(f64)` and implements no `Eq`, so the marker cannot ride the
   containment chain. Nothing needed it — every comparison is `assert_eq!`.
   Don't "restore" it.
2. **Read the log through the right accessor.** Production has exactly one
   consumer of the contents (`render/envelope.rs`) and it must see **both** arms
   — a production reader that skipped opaque rows is the silent skip `STD-003`
   forbids. `ChangeLog::read_rows()` is `#[cfg(test)]` on purpose. See
   [[mem.pattern.testing.readable-arm-accessor-narrows-every-sweep]].

## Classification is an ordered try, and the order is the contract

Deserialisation attempts the whole `ChangeRow` first, then classifies the cause
where it is known — event token, then payload key, then value kind, then the
admission bound. A failure **no** vocabulary explains propagates the original
serde error, which is how "tolerance is for vocabulary, never for shape" stays a
property of the structure rather than a second guard. The order is pinned by
`each_unreadable_cause_is_classified_where_it_is_known` (`snapshot.rs`).

Pin any compat claim at `snapshot::parse` over a **literal legacy fragment**,
never a unit round-trip over the inner type — see
[[mem.fact.design-run.snapshot-outlives-the-binary]].
