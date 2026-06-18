# Memory read-path relations and agent UX hardening

## Context

Doctrine's memory system has an authored relation graph (180 `[[relation]]`
rows across 100 memories) and wikilink cross-references in body text — but
neither surfaces at any CLI read path. `memory show` and `memory retrieve`
don't include relations. `inspect` refuses memory refs. Wikilinks in body
text (`[[mem.pattern.…]]`) are never extracted or resolved. Skills actively
route around the relation system rather than encouraging connection-making.

`scratch/memory-spec.local.md` describes the intended model: wikilink
resolution, backlink queries, graph expansion, and skills that build a
living relation graph. This slice closes the gap between that model and the
current implementation, plus hardens the general agent UX around memory
read/write surfaces.

Full gap analysis: `capability-gaps.md`.

## Design principles

- **Wikilinks and edges coexist.** Inline `[[mem.…]]` wikilinks carry
  contextual meaning where it belongs inline in prose. Formal
  `[[relation]]` edges (`doctrine link`) are introspectable, queryable,
  structural connections. Skills should guide agents to add either or both
  as appropriate — wikilinks for "see also" context within the narrative,
  edges for durable graph structure.
- **Treat the link set as a single graph for most purposes.** Graph
  operations (backlinks, expansion) take the deduplicated union of wikilink
  targets and `[[relation]]` targets — the consumer shouldn't care which
  mechanism created the connection.
- **No persistence for derived data.** Wikilinks are regex-extracted
  on-the-fly from body text (~0.007s corpus-wide at current scale). No
  `links.out`/`links.missing` metadata fields.

## Scope & Objectives

### 1. Surface authored `[[relation]]` rows at read surfaces

- `memory show` — include a `relations` section (label + target) in text
  and JSON output
- `memory retrieve` — include relations in the output block
- `doctrine inspect` — accept memory refs (`mem_<uid>`, `mem.<key>`) and
  render inbound + outbound edges

### 2. Wikilink extractor (on-the-fly)

- Regex-extract `[[mem.<key>]]` / `[[mem_<uid>]]` from memory bodies,
  skipping fenced code blocks and inline code
- Resolve extracted targets against the memory registry (uid, key, and
  shorthand without `mem.` prefix)
- `doctrine memory resolve-links [ID]` — extract + resolve wikilinks for
  a specific memory or all memories, report resolved vs dangling

### 3. Backlinks and graph expansion

- `doctrine memory backlinks <ID>` — build reverse index (wikilinks +
  relations, deduped) across all memories, return sources linking to the
  target
- `expand_link_graph(ID, depth)` — BFS from a root memory following the
  union of wikilink targets and relation targets up to configurable depth;
  output structured nodes with depth annotation
- Integrate into `memory retrieve` as an optional `--expand N` flag for
  contextual signal expansion

### 4. Agent UX hardening

- **Reconsider verify-on-clean-worktree.** The current rule prevents an
  agent mid-work from attesting a memory. Options to evaluate during
  design:
  (a) allow verify on dirty tree, stamp checkout_state_id
  (b) allow verify with a `--allow-dirty` flag
  (c) document the tradeoff and keep the rule, provide a workflow escape
- **Record-memory skill:** guide agents to add `[[relation]]` edges via
  `doctrine link` after recording related memories. Wikilinks in body for
  contextual "see also" references; edges for durable graph structure.
  Both is fine — the graph dedupes.
- **Retrieve-memory skill:** mention relations, backlinks, `--links-to`,
  graph expansion (`--expand`). Add a connection-making step: after
  retrieving a memory, check its relations and follow relevant edges.
- **New skills:**
  - `/maintaining-memory` — validate against current code, handle lifecycle
    (supersede, archive, deprecate), re-scope stale memories
  - `/reviewing-memory` — structured audit for stability gates; skeleton
    with key headings

### 5. Schema additions (fields)

Add the following fields to the memory TOML schema (all optional, backward
compatible):

- `provenance.sources` — `[{kind, ref, note}]` structured provenance
  (code, adr, spec, commit — where this knowledge came from)
- `review_by` — scheduled review date (optional; shorter for volatile
  memories)
- `lifespan` — cognitive category, orthogonal to `memory_type` (content
  form). Values: `semantic`, `episodic`, `procedural`, `working`,
  `identity`. Default: unset (no opinion).
  - `semantic` — durable knowledge about the codebase: facts, invariants,
    concepts, terminology
  - `episodic` — event knowledge: "this happened", historical context,
    war stories, migration narratives
  - `procedural` — how-to, recipes, workflows: "do this sequence"
  - `working` — current task context, ephemeral; decays fast
  - `identity` — self-model: what doctrine is, subsystem maps, signposts

### 6. Status lifecycle

Add `deprecated`, `superseded`, `obsolete`, `archived` status values to the
memory lifecycle. Default `show`/`retrieve`/`find` exclude `deprecated`,
`superseded`, `obsolete` (same as the spec-driver model). Archival is a
soft exclusion (visible with `--include-archived`).

## Non-Goals

- Changing the storage backend (forgettable compatibility is separate work)
- Modifying the relation write path — memory labels remain free-form per
  `mem.pattern.link.memory-label-fork`
- Catalog scan inclusion of memories (deferred — current exclusion is fine)
- Full staleness computation rewrite (the existing mechanism works)
- Unified `inspect` across all entity kinds (memories specifically)
- Embedding / semantic retrieval
- Markdown parity with forgettable backend
- Fields: `audience`, `visibility`, `requires_reading`, `owners` (no
  current consumer; add when needed)
- Changing `trust_level` to `confidence` (naming is cosmetic; not worth
  the migration churn)

## Risks & Assumptions

- **Wikilink extraction performance.** Corpus-wide regex extraction is
  ~0.007s at current scale (193 memories). Scales linearly with corpus
  size — re-evaluate if it ever exceeds a human-noticeable threshold.
- **Backward compatibility.** All new TOML fields are optional with
  defaults. Existing memories and queries are unaffected.
- **Verify-on-dirty-tree.** Changing this rule has implications for
  staleness computation (a dirty-tree attestation is inherently less
  trustworthy than a commit-attested one). Decided during design.
- **Dedup semantics.** Wikilink targets and `[[relation]]` targets are
  treated as a set — if a memory links to the same target both ways, it
  appears once in graph output. Direction is preserved (wikilinks are
  outbound only; relations carry their authored label).

## Verification / Closure Intent

- `memory show` and `memory retrieve` include relations for memories that
  have them
- `memory resolve-links [ID]` reports resolved vs dangling wikilinks
- `memory backlinks <ID>` returns correct reverse edges (union of wikilinks
  + relations)
- `memory retrieve --expand 2` returns expanded graph nodes
- `doctrine inspect mem_<uid>` renders inbound + outbound edges
- New TOML fields round-trip through `memory record`, `show --json`, and
  back
- Skills updated: record-memory, retrieve-memory; new maintaining-memory
  and reviewing-memory skeletons committed
- Existing test suites stay green unchanged (behaviour-preservation gate)
- All new functionality has test coverage (TDD per phase)
