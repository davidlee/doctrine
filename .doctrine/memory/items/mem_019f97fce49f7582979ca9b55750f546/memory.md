# Both merge sides bumping the same assert counter to the same value auto-merges to a wrong total

A count assertion is the one place where git's textual merge is reliably *wrong
while clean*. Two branches each add one entity and each bump

```rust
assert_eq!(visible.len(), 52);   // base
```

to `53`. Both sides changed the line to **identical text**, so git resolves it
with no conflict — but the merged tree contains *both* additions and needs
`base + Δ_ours + Δ_theirs` = `54`. The suite then goes red on a merge that
reported clean, or worse, goes green against a number that no longer describes
the corpus.

**How to apply.** Treat any `assert_eq!(<something>.len(), N)` / census / total
as merge-hostile:

- After any merge that touched both sides of a counted corpus, **re-derive N
  empirically** (run the census verb, count the entities) rather than trusting
  the merged literal.
- Prefer asserting a *delta* or a computed expectation over a hard-coded total
  where the corpus is expected to grow.
- Don't reason about it from the diff — a clean auto-merge shows no hunk at all
  for the counter line.

Observed at SL-227: two independent additions (`graph` and `library`) each bumped
the same visible-entity counter, git auto-merged the identical edit, and the
resulting total was wrong. The true census (53) was established empirically after
the merge, not read off the merged literal.
