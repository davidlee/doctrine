# CHR-021: Audit and improve shipped memory corpus

The shipped memory corpus (`memory/` → RustEmbed → `.doctrine/memory/shipped/`)
is the orientation surface for every agent using doctrine. It's currently
uneven: some concepts are thorough, others are stubs; cross-links are sparse;
MCP response fields are undocumented; and the corpus hasn't been systematically
audited for correctness against the current CLI surface.

## Audit axes

### 1. Currency

Every shipped memory carries `reviewed` dates and some carry `review_by`.
Check each against the current codebase:

- **Factual statements** that may have drifted (command names, file paths,
  constants, CLI flags)
- **References to slice work** that has since landed or been superseded
- **Status of features** described as "in progress" or "not yet integrated"
  (e.g. `mem.fact.doctrine.sl-101-facets-unintegrated` — is this still true?)
- Re-verify anything with a `review_by` in the past

### 2. Completeness (app surface)

Map the CLI surface to the shipped corpus. Gaps to check:

- Every major command group should have a signpost or concept memory
  (e.g. `worktree`, `dispatch`, `review`, `backlog`, `slice`, `spec`, `rfc`,
  `memory`, `estimate`, `relation`, `coverage`)
- Every major concept should have a concept memory (entity engine, storage
  model, routing gate, boot snapshot, reading entities, etc.)
- Common footguns should be covered (stale binaries, RustEmbed, jail target
  dir, indexing bans, etc.)
- **MCP response fields** — the `memory_find`/`retrieve`/`show`/`list`
  response shapes are undocumented (IMP-148 Gap 7)

### 3. Accuracy

Spot-check the top-N memories by severity × trust level:

- Cross-reference cited code paths against the current tree
- Verify described behaviours by testing (e.g. "cordage denylist whole-word
  matches" — does it still work this way?)
- Check `mem.pattern.build.rust-embed-no-rerun` for completeness (does it
  cover `src/corpus.rs` and `memory/` as well as `src/install.rs` and
  `src/skills.rs`?)

### 4. Appropriateness for onboarding / inexpert users

The corpus is the first thing a new agent retrieves. Expect:

- Concepts should start with an orienting sentence — what the thing IS before
  how it works
- Signposts should tell the reader where to go next, not just list facts
- Jargon should be defined or linked (e.g. "the holdback", "anchor frame",
  "checkout_state_id")
- The two faces (local vs shipped) should be clear from the first retrieval

### 5. Cross-linking

This IS the onboarding flow for both agents and humans. Links deserve careful
design:

- Every signpost should link to its concept siblings and relevant patterns
- Every concept should link to signposts that route you there, related
  patterns, and relevant ADRs
- The graph should have a clear "start here" path: overview → core concepts
  → relevant patterns → signposts for action
- Orphan memories should be linked or justified as standalone
- The wikilink vs relation distinction should be consistent (wikilinks are
  inline in body prose; relations are authoring-time structured edges)

### 6. Discoverability / ranking

- **In a fresh repo**: with no local memories, shipped memories are the
  only results for `find`/`retrieve`. Check that key queries return the
  right top results (e.g. "how to record memory", "routing", "dispatch",
  "review", "entity engine")
- **In this repo**: shipped memories are unioned with the large local corpus.
  They should still surface appropriately — check that core concepts aren't
  drowned out by local episodic memories. Adjust weights/severity if needed.
- The `weight` and `severity` fields on shipped memories are mostly 0/none.
  Consider adding calibrated weights to key onboarding memories.

## Approach

1. Read every shipped memory (`memory/` dirs, ~31 entries) — get a baseline
2. Produce a findings ledger (per-memory: currency ok/drifted, completeness
   gap, link suggestions)
3. Triage: fix-now (typo, stale path) vs design-needed (new concept, restructure)
4. Apply edits to `memory/` source
5. Rebuild + sync (`touch src/corpus.rs && cargo build && doctrine memory sync`)
6. Verify discoverability with key queries

## Related

- IMP-148 (MCP tool help gaps — the response-field section for the concept memory)
- `mem.pattern.distribution.shipped-memory-authoring` (the authoring flow)
- `mem.concept.doctrine.memory-model` (the memory model concept itself — needs
  the MCP response-field section)
- `mem.pattern.build.rust-embed-no-rerun` (the re-embed footgun — applies to
  `src/corpus.rs` too)
