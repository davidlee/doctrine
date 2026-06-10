# IMP-011: Warn on record of an unverified thread (invisible to find/retrieve until verified)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`thread_expiry` (`src/retrieve.rs`, SL-008 design D6) drops any `thread` that is
not `verified` AND `reviewed` within 14 days from `find`/`retrieve`. `record`
always scaffolds `unverified`, so a freshly-recorded thread is invisible to
scope ranking (only `list`/`show` reveal it) until it is verified on a clean
tree. This surprised an agent (SL-032 handover "memory retrieval blind spot")
who read it as a staleness/indexing bug.

The behaviour is **correct** (D6 rejected surfacing unverified threads). The
remedy already shipped is documentation-only: the `/record-memory` skill now
warns (§1/§5/§7) and `mem.pattern.memory.thread-hidden-until-verified` records
the footgun.

## Proposed (deferred — engine change, not yet scheduled)

Optionally have `doctrine memory record --type thread` emit a one-line stderr
note on success: e.g. "recorded thread is hidden from find/retrieve until
`verify`d (clean tree)". Pure UX nudge on the record shell — no change to
`thread_expiry` or any read path, so the behaviour-preservation gate is
untouched. Decide whether the CLI nudge is worth the surface once the skill-doc
remedy has had time to prove insufficient.

