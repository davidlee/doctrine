# IMP-026: Backlog triggers actionability mask (SPEC-001 D6)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred from SL-039. SL-039 mints the authored `triggers = [{ globs, note }]`
field (PRD-009 FR-010/REQ-097 mints the field only); this item is the **mask**
that consumes it.

**The behaviour.** A path-glob trigger holds an item non-actionable until a file
set matches the globs of any of its triggers — SPEC-001 D6:
`mask(item, files) = ∃ t ∈ item.triggers · glob_admits(t.globs, files)`, over the
leaf `src/retrieve.rs` predicates (SL-008; reuse verbatim, no new glob engine).
A policy-layer mask, **never a graph edge** — it does not touch cordage ordering.

**Why blocked.** SPEC-001 **OQ-009 is open**: D6 fixes the mask *shape* but leaves
the **plan/audit file-set source unbuilt** — "the matcher needs two inputs that do
not yet exist." Building the mask requires first resolving OQ-009 (define + build
the file-set source) — a SPEC-001 amendment + design effort, out of SL-039's
vocab-reconcile scope.

**Entry condition.** Reopen SPEC-001 OQ-009; design the file-set source; then this
mask + its `backlog list`/`order` actionability integration.

Refs: SPEC-001 D6 / OQ-009, PRD-009 FR-011/REQ-098, SL-039 design (triggers field),
the leaf matchers `glob_admits`/`path_admits` (`src/retrieve.rs`).
