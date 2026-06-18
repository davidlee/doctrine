# Memory read-path relations and agent UX hardening

## Context

Doctrine's memory system has an authored relation graph (180 `[[relation]]`
rows across 100 memories) and wikilink cross-references in body text — but
neither surfaces at any CLI read path. `memory show` and `memory retrieve`
don't include relations. `inspect` refuses memory refs. The catalog excludes
memories. Wikilinks in body text (`[[mem.pattern.…]]`) are never parsed or
resolved. Skills actively route around the relation system rather than
encouraging connection-making.

`scratch/memory-spec.local.md` describes the intended model: derived wikilink
resolution, backlink queries, graph expansion, and skills that build a living
relation graph. This slice closes the gap between that model and the current
implementation, plus hardens the general agent UX around memory read/write
surfaces.

Full gap analysis: `capability-gaps.md`.

## Scope & Objectives

### 1. Surface authored `[[relation]]` rows at read surfaces

- `memory show` — include a `relations` section (label + target) in text and
  JSON output
- `memory retrieve` — include relations in the output block
- `doctrine inspect` — accept memory refs (`mem_<uid>`, `mem.<key>`) and
  render inbound + outbound edges
- (Catalog scan inclusion of memories is deferred — the user confirms the
  current exclusion is fine for its only consumer.)

### 2. Wikilink extractor (on-the-fly, no persistence)

- Regex-extract `[[mem.<key>]]` / `[[mem_<uid>]]` from memory bodies,
  skipping fenced code blocks and inline code. Corpus-wide extraction is
  cheap (~0.007s at current scale) — no persisted `links.out`/
  `links.missing` fields needed.
- Resolve extracted targets against the memory registry (uid, key, and
  shorthand without `mem.` prefix)
- `doctrine memory resolve-links [ID]` — extract + resolve wikilinks for
  a specific memory or all memories, report resolved vs dangling

### 3. Backlinks and graph expansion

- `doctrine memory backlinks <ID>` — extract wikilinks from all memory
  bodies, build reverse index, return sources that link to the target
- `expand_link_graph(ID, depth)` — BFS from a root memory following
  extracted wikilinks up to configurable depth; output structured nodes
  with depth annotation
- Integrate into `memory retrieve` as an optional `--expand N` flag for
  contextual signal expansion
- All computed on-the-fly from body text — no persistence, no derived
  metadata fields

### 4. Agent UX hardening

- **Reconsider verify-on-clean-worktree.** The current rule prevents an
  agent mid-work from attesting a memory. Options to evaluate:
  (a) allow verify on dirty tree with a warning/flag; (b) allow verify
  that stamps the working-tree state (checkout_state_id) rather than a
  clean commit; (c) document the tradeoff and keep the rule but provide
  a workflow escape.
- **Record-memory skill:** mention `doctrine link` for creating formal
  `[[relation]]` edges. Encourage building the graph, not just inline refs.
- **Retrieve-memory skill:** mention relations, backlinks, `--links-to`,
  graph expansion. Add a connection-making step.
- **New skills:**
  - `/maintaining-memory` — validate against current code, handle lifecycle
    (supersede, archive, deprecate), re-scope stale memories
  - `/reviewing-memory` — structured audit for stability gates; skeleton
    with key headings, don't fully flesh until used

### 5. Schema additions (fields)

Add the following fields to the memory TOML schema (all optional, backward
compatible):

- `audience` — `["human"]`, `["agent"]`, or both (default both if absent)
- `visibility` — `["pre"]` for pre-hook surfacing (empty by default)
- `requires_reading` — list of file paths memory readers should read first
- `provenance.sources` — `[{kind, ref, note}]` structured provenance
- `review_by` — scheduled review date

(`links.out`/`links.missing` are NOT persisted — computed on-the-fly from
body text; see objective 2.)

### 6. Status lifecycle

Add `deprecated`, `superseded`, `obsolete`, `archived` status values to the
memory lifecycle. Default `show`/`retrieve`/`find` exclude `deprecated`,
`superseded`, `obsolete` (same as the spec-driver model). Archival is a soft
exclusion (visible with `--include-archived`).

## Non-Goals

- Changing the storage backend (forgettable compatibility is separate work)
- Modifying the relation write path — memory labels remain free-form per
  `mem.pattern.link.memory-label-fork`
- Catalog scan inclusion of memories (deferred — current exclusion is fine)
- Full staleness computation rewrite (the existing mechanism works)
- Unified `inspect` across all entity kinds (memories specifically)
- Embedding/semantic retrieval
- Markdown parity with forgettable backend
- `owners` field (team ownership — no current consumer)
- Changing `trust_level` to `confidence` (naming is a cosmetic concern; not
  worth the migration churn right now)

## Risks & Assumptions

- **Wikilink extraction performance.** Corpus-wide regex extraction
  is ~0.007s at current scale (193 memories). Grow linearly with
  corpus size — re-evaluate if it ever exceeds a human-noticeable
  threshold.
- **Backward compatibility.** All new TOML fields are optional with
  defaults. Existing memories and queries are unaffected.
- **Verify-on-dirty-tree.** Changing this rule has implications for
  staleness computation (a dirty-tree attestation is inherently
  less trustworthy than a commit-attested one). This needs design
  discussion before the change lands.

## Verification / Closure Intent

- Every new field appears in `memory show --format json` and round-trips
- `memory show` and `memory retrieve` include relations for memories
  that have them
- `memory resolve-links` populates `links.out`/`links.missing`
- `memory backlinks <ID>` returns correct reverse edges
- `memory retrieve --expand 2` returns expanded graph nodes
- `doctrine inspect mem_<uid>` renders inbound + outbound edges
- Skills updated: record-memory, retrieve-memory; new maintaining-memory
  and reviewing-memory skeletons committed
- Existing test suites stay green unchanged (behaviour-preservation gate)
- All new functionality has test coverage (TDD per phase)
