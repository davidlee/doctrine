# IMP-317: Route retrieve staleness through the shared claim-surface constructor

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`retrieve::git_facts` (`src/retrieve.rs:556-557`) derives a path set from a
memory's scope **raw**: it gates on `m.scope.paths.is_empty()` and passes the
array straight to `commits_touching`. SL-230 builds a single claim-surface
constructor and adopts it in `verify` and `validate` (design D11); this is the
third consumer, deliberately left unconverted.

Adopting it fixes, in one call-site swap:

- glob-only memories get no retrieval-side staleness at all (the `is_empty()`
  gate skips them);
- `scope.globs` is invisible to drift;
- absolute and symlinked scope entries are not canonicalised (SL-230 F-15/F-20);
- pathspec magic in scope values is not neutralised (SL-230 F-18).

## Why it was deferred, not fixed

Converting `git_facts` changes **retrieval ranking corpus-wide** — staleness feeds
ordering. That is precisely the decision SL-230's **OQ-2** defers, on the stated
ground that it "would reclassify a large fraction of the corpus at once and shift
retrieval ordering broadly". Taking it inside a body-write slice would have been
scope expansion under cover of a bug fix, in a design already carrying five review
rounds.

The bound is on *this slice*, not on the correctness. SL-230 chose the
constructor's signature — `(root, memory, dir)`, borrowing nothing from `verify`'s
command context — specifically so adoption here is a call-site swap rather than a
redesign. Carried as SL-230 **R7**.

## Gate

Blocked on **OQ-2**: does own-directory / claim-surface drift feed retrieve-side
staleness ranking, or only `validate`? Answer that first — this item is the
implementation of "yes", and should be closed as `wont-do` if the answer is "no",
with R7 restated as permanent and intended rather than provisional.

Raised as RV-307 F-24 against SL-230's design.
