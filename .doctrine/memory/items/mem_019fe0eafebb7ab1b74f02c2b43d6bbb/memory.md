# `apply_tags_set` self-heals before its no-op guard

`tag::apply_tags_set(doc, adds, removes, today)` (`src/tag.rs:78`) does its
self-heal FIRST — if the `tags` key is absent it inserts `tags = []` into the
held table — and only then computes the set algebra and the set-compare no-op
guard.

So on a record MISSING `tags`, a call with an empty `adds` (and empty
`removes`) returns `Ok(false)` **having already mutated the document**. If any
other concern in the same shell reports a change and drives the shared
`write_atomic`, that `tags = []` rides out to disk on the back of an unrelated
edit — a change the changed-flag said did not happen.

Harmless in practice today: every scaffold seeds `tags = []`, so the self-heal
only fires on a hand-damaged record. But it is a live hazard for any shell that
composes several write cores on ONE held `DocumentMut` (the "one open, one
write" shape).

**The cheap defence, and the one `knowledge edit` uses:** do not call the core
at all when there is nothing to add.

```rust
let tags_changed = if adds.is_empty() {
    false
} else {
    crate::tag::apply_tags_set(&mut doc, &adds, &BTreeSet::new(), &today)?
};
```

Contrast `dep_seq::apply_status`, which runs its no-op guard BEFORE touching
the table and refuses (F-1) rather than creating an absent managed key. The two
seams disagree on absent-key posture on purpose; the asymmetry is inherited by
every caller that composes them.
