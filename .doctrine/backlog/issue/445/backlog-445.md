# ISS-445: lazyspec duplicates the backlog terminal vocabulary outside status_class

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found by SL-238 PHASE-07's class sweep (`mem.pattern.review.sweep-defect-class-not-instance`):
having removed the hardcoded terminal literal from `src/commands/dep_seq.rs`, sweep
for the *class* — a second copy of a kind's terminal vocabulary living outside
`priority::partition::status_class`.

`src/lazyspec.rs:193` holds one:

```rust
"issue" | "improvement" | "chore" | "risk" | "idea" => match status {
    "triaged" => "review",
    "started" => "in-progress",
    "resolved" | "closed" => "complete",
    _ => "draft",
},
```

`resolved | closed` is the backlog terminal set, spelled a second time. It is the
same STD-001 violation SL-238 §6 removes from `--prune`, and the same REQ-238
routing breach: a per-kind terminality question that does not route through
`partition::status_class`.

**Not identical to `--prune`'s case, and the difference matters for whoever takes
this.** This is a *projection to a foreign wire vocabulary* (`complete` /
`in-progress` / `review` / `draft`), not a terminality test — so the repair is not
simply calling `status_class`. The arms above it (spec, ADR) map non-work kinds
whose vocabularies `status_class` does not partition the same way. The likely shape
is: route the terminal question through `authored_class`/`status_class` and keep
only the non-terminal wire mapping local. Scope the fix against `IMP-105` (extend
the lazyspec projection to the remaining kinds), which touches the same table.

Sibling census from the same sweep, recorded so it is not re-derived: the other
`"resolved"` hits in `src/` are RFC's own status vocabulary (`src/rfc.rs`, legitimate
— RFC statuses are not backlog statuses), enum-to-string renderings
(`design_run/inquiry.rs`, `relation_query.rs`), and test fixtures. `src/backlog.rs`'s
24 hits are SL-238 PHASE-08's leg and are already owned.
