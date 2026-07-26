# IMP-317: Route retrieve staleness through the shared claim-surface constructor

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Two consumers derive a path set from a memory's scope **raw** — they gate on
`scope.paths.is_empty()` and pass the array straight to `commits_touching`:

- `validate`'s staleness check (`src/memory.rs:3413-3421`);
- `retrieve::git_facts` (`src/retrieve.rs:556-557`).

SL-230 builds a claim-surface constructor for `verify` and deliberately converts
neither (design D11). Adopting one fixes, for both:

- glob-only memories get no drift signal at all (the `is_empty()` gate skips them);
- `scope.globs` is invisible to drift;
- absolute and symlinked scope entries are not canonicalised (SL-230 F-15/F-20);
- pathspec magic in scope values is not neutralised (SL-230 F-18).

## Why it was deferred, not fixed

Two reasons, and the second was discovered late.

**1. It changes corpus-wide ranking.** Staleness feeds retrieval ordering. That is
precisely the decision SL-230's **OQ-2** defers, on the stated ground that it
"would reclassify a large fraction of the corpus at once and shift retrieval
ordering broadly". Taking it inside a body-write slice would have been scope
expansion under cover of a bug fix.

**2. `verify`'s surface is the wrong surface to adopt** (RV-307 F-27). `verify`
asks *is this evidence dirty now*, where canonicalisation is mandatory —
a pathspec naming a symlink reads clean while its target is modified. `validate`
and `retrieve` ask *how many commits touched this since `verified_sha`*, where
canonicalisation is actively wrong: it resolves against today's checkout and then
queries yesterday-to-today history, so a committed symlink retarget counts 1
through the declared path and 0 through the resolved one (measured, git 2.54.0).

So this item is **not** "call the existing constructor from two more places". It
is "build a *history-stable* claim surface" — same input, different resolution
rules — and then adopt it.

## Cost, corrected

SL-230 round 5 claimed adoption would be a call-site swap because the constructor
takes `(root, memory, dir)`. RV-307 **F-28** falsified that: neither consumer has
a `dir`. `memory_health_findings(root, &[Memory], today)` (`src/memory.rs:3400`)
receives none, `run_validate` discards `_dir` (`:3480-3483`), and `collect_all`
(`:2826-2834`) unions `items/` and `shipped/` into one `Vec<Memory>` — so the
row's origin is not merely absent but unrecoverable from `uid`, which is why
`read_body` (`:2788-2797`) probes both roots. `retrieve::git_facts` and its caller
(`src/retrieve.rs:628`) are the same.

Adoption therefore requires threading item-directory provenance through
`collect_all` and `memory_health_findings` — a dataflow change, not two arguments.

## Gate

Blocked on **OQ-2** (QUE-175): does claim-surface drift feed retrieve-side
staleness ranking, or only `validate`? Answer that first — this item is the
implementation of "yes", and should be closed as `wont-do` if the answer is "no",
with SL-230 **R7** restated as permanent and intended rather than provisional.

Raised as RV-307 F-24 against SL-230's design; scope and cost corrected by F-27
and F-28.
