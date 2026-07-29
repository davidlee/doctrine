# IMP-355: Kind-local .prettierignore on first record write

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Low priority. A mitigation, not a fix — the exposure it addresses is
recoverable and loud.**

When Doctrine writes the *first* record into a kind folder, it could also drop a
`.prettierignore` containing `*` beside it. Cheap, idempotent, and it makes
`.doctrine/**` prose invisible to a client project's Markdown formatter without
anyone having to know it should be.

## Why kind-folder-local rather than repo-root

Writing a root `.prettierignore` entry from `doctrine install` is the obvious
move and it is the wrong one: seeding a host-tool config file is the platform
reaching into a host-project convention, which POL-002 forbids. A file Doctrine
writes *inside its own folder* is not that — it is Doctrine describing its own
tree, and prettier's ignore-file resolution honours it either way.

That distinction is the whole reason this is worth doing at all, and it is why
the trigger is "first record write" rather than install time: install-time
seeding is a statement about the host repo; first-write is a statement about a
folder Doctrine owns.

## What it mitigates

A formatter run over `.doctrine/**` does not corrupt anything — measured under
SL-233: markers survive byte-identically, and prettier cannot parse TOML at all,
so the authored entity tier is out of reach entirely. The exposure is Markdown
prose only, and the consequence chain is:

1. bodies change (prettier inserts a blank line after every marker), so every
   section fingerprint moves;
2. that is a foreign edit, so ordinary mutation entry-refuses — the user is
   told, not silently ignored;
3. one `adopt` recovers, and the document then sits at the formatter's fixed
   point;
4. **the cost is evidence** — DEC-066 invalidates attestations bound to the old
   bytes, and since a format run touches essentially every section, one stray
   invocation invalidates the review evidence for a whole document.

So this buys avoiding a one-time whole-document re-review, not avoiding data
loss. Sizing it above "low" would be overreacting.

## Caveats worth carrying into the design

- **Prettier-specific.** `mdformat`, `dprint` and `markdownlint --fix` have
  their own ignore mechanisms and are unmeasured. A generic answer would need
  several files, which is probably worse than the problem.
- **It is still a host-tool artefact**, just a well-placed one. Whether even
  this crosses POL-002 deserves a moment's thought at design time rather than
  being treated as settled by the folder location.
- **Not every kind folder holds Markdown worth ignoring.** The TOML tier is
  already immune, so the value concentrates where prose lives.

## Provenance

Surfaced during SL-233 PHASE-06's design gate (RV-323 round 6), where three
grammar rules turned out to have been derived from an untested hypothesis about
a client project running a formatter. The measurements above are in
`.doctrine/slice/233/sketches/marker-grammar.md` §(f), and the probe behind them
is at `.doctrine/slice/233/sketches/probe/title-differential.py`.

SL-233 §(f) documents `.doctrine/**` as not formatter-safe Markdown and
explicitly declines to have the installer write a root `.prettierignore`. This
item is the narrower thing that might be acceptable instead.
