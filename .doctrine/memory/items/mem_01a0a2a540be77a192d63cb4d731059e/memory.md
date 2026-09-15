# Widening a collection to a sum type? The convenience accessor is a silent skip

When a stored collection gains a tolerant arm —

```rust
-  rows: Vec<ChangeRow>
+  rows: Vec<StoredRow>          // Read(ChangeRow) | Unreadable(RawRow)
```

— every test reading `.rows` stops compiling, and the cheap repair is one
accessor that hands back only the good arm:

```rust
#[cfg(test)]
fn read_rows(&self) -> impl Iterator<Item = &ChangeRow> {
    self.rows.iter().filter_map(StoredRow::read)
}
```

That is the right repair for a test **asserting about a row it wrote**. It is
the wrong repair, invisibly, for a test that **sweeps the collection for the
absence of something**:

```rust
let strays: Vec<&str> = log.read_rows()          // was: log.rows.iter()
    .map(|row| row.event)
    .filter(|event| !ChangeEvent::EMITTABLE.contains(event))
    .collect();
assert!(strays.is_empty());
```

The sweep now proves "no *readable* row is a stray". If anything ever lands in
the other arm, the assertion does not fail — it goes **vacuous**. A green test
that has quietly stopped examining its subject is worse than no test, because it
still reads as coverage.

## The repair

Pair the narrowed sweep with a **conservation assertion** at the same site:

```rust
assert_eq!(
    log.read_rows().count(), log.rows.len(),
    "every row the ladder wrote reads back: {:?}", log.rows
);
```

One line, and it converts the vacuity into a red.

## How to find them during the widening

The compiler shows you every call site — that is the moment to triage, and the
triage is one question per site: **is this test asserting about a row it put
there, or sweeping for something it expects to be absent?** The first is safe;
the second needs the guard. Do it while the errors are on screen; after the
suite is green nothing will ever point at these sites again.

Same shape on the production side, where it is `STD-003`'s no-silent-skip rule
outright: a production reader over such a collection should see **both** arms, so
the accessor is `#[cfg(test)]` rather than merely unused in production. See
[[mem.fact.design-run.change-log-degrades-state-refuses]] for the instance.
