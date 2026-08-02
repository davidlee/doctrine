# IMP-347: Stall sweep misses out-of-directory entity writes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The stall sweep — walking each planned phase's edit set against the slice's
declared selectors, so no phase stalls at the conformance boundary — carries an
unstated assumption: that authoring an entity writes **inside that entity's own
directory**.

Several entity-authoring verbs do not. `doctrine spec req add` writes the
requirement to `.doctrine/requirement/<NNN>/` plus a slug symlink alongside —
outside the spec's tree entirely, because a product spec's functional and
quality requirements are peer `REQ-NNN` entities, not spec-local rows (the
single hard rule the entity model exists to enforce).

`src/conformance.rs:131` — `undeclared_paths` flags every delta path with no
matching selector — so the phase lands undeclared paths and fails its
conformance gate, at the end, after the authoring work is done.

## Where it bit

SL-233 PHASE-01 (finding F-1, blocking). The sweep in `plan.md` walked every
phase's edit set and still missed it; the phase stalled mid-flight and cost a
consult. Recovery was to repeat the narrow bootstrap — allocate REQ-414…438,
declare their 50 exact paths in a second selector-only commit before any body
commit. No slice in the corpus had ever declared a requirement-tree selector,
so there was no precedent to copy from either.

## Shape of a fix

The durable form is a rule in the sweep's own instructions: *before authoring
any new entity kind, check where its bytes actually land against the selector
list* — the verb's write set, not the entity's conceptual home. That belongs
wherever the stall sweep is taught (the `/plan` skill's sweep step).

Better, if it is cheap: make it answerable rather than remembered. A verb that
reports an entity-authoring command's write set — or a dry-run conformance
check over a phase's *projected* paths — turns a discipline into a query. Worth
scoping before committing to the prose-only fix.

Two adjacent facts worth carrying into whatever fixes this, both from the same
phase:

- The wildcard escape (`.doctrine/requirement/**`) was rejected deliberately —
  it grants authority over all pre-existing requirements, the corpus-wide
  wildcard `design.md:480` already declined for specs.
- `NNN**` is an uncompilable glob ("recursive wildcards must form a single path
  component"), so a single selector row per entity is not available. It takes
  two: `NNN/**` for contents and `NNN*` for the directory entry and slug
  symlink.

## Second instance — SL-233 reconcile (2026-08-02)

SL-233's conformance report closed with eleven undeclared paths, and every one
is the slice's **own process output**: the two dispatch bookkeeping files
(`boundaries.toml`, `funnel.toml`); the governance the work itself minted (the
`DEC-100` record, its body and its slug symlink, plus the `DEC-099` supersession
edit); the `RV-324` ledger, its body and its slug symlink; and the slice's own
two entity files (`notes.md`, `slice-233.toml`).

**This is the same class arriving from the opposite direction, and the
distinction is worth keeping.** The PHASE-01 instance above is a *knowable*
write set — `spec req add` lands bytes in a directory you can look up before you
run it, so the sweep could in principle have caught it. These eleven are not
knowable: a selector list declares what a slice sets out to **change**, and it
cannot name an id that does not exist until the work **mints** it. Requiring the
declaration would mean declaring `DEC-100` before `DEC-100` was allocated.

So the prose-only fix proposed above ("check where its bytes actually land")
does **not** reach this half. A rule an author cannot follow is not a fix. What
this instance argues for is the *other* branch — making it answerable rather
than remembered — plus a prior question the first instance did not raise:
**should process output be in the conformance denominator at all?** Minted
governance records, review ledgers, dispatch bookkeeping and the slice's own
entity files are a closed, recognisable set. If conformance excluded them by
kind, eleven of SL-233's thirteen undeclared rows would never have been
reported, and the two that carried real signal (`src/commands/verify.rs`,
`tests/runbook_fixture/**` — a genuine declaration gap in the hand-enumerated
command tier) would have stood alone instead of at 15% of the noise.

Disposition at SL-233 reconcile: **accepted, no selector action.** Recorded here
so the exclusion-by-kind question is carried rather than re-derived — it is the
part of this item with the most leverage and the least design.
