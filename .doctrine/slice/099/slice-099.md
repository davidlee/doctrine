# Memory read-path relations and data-model hardening

## Context

Doctrine's memory system has an authored relation graph (180 `[[relation]]`
rows across 100 memories) and wikilink cross-references in body text — but
neither surfaces at any CLI read path. `memory show` and `memory retrieve`
don't include relations. `inspect` refuses memory refs. Wikilinks in body
text (`[[mem.pattern.…]]`) are never extracted or resolved.

This slice adds the read-path surfaces, wikilink extraction, backlinks,
graph expansion, missing `record` flags, the `lifespan` field, ageing in
the sort key, suggested relations, a `validate` command, and a
`verify --allow-dirty` escape.

Lifecycle verbs (`status`, `edit`, `tag`) and skill updates are deferred to
SL-100. Full gap analysis: `capability-gaps.md`. UX spike: `ux-spike.md`.

## Design principles

- **Wikilinks and edges coexist.** Inline `[[mem.…]]` wikilinks carry
  contextual meaning where it belongs inline in prose. Formal
  `[[relation]]` edges (`doctrine link`) are introspectable, queryable,
  structural connections. Skills guide agents to use either or both.
- **Treat the link set as a single graph.** Graph ops (backlinks, expansion,
  suggested relations) take the deduplicated union of wikilink targets and
  `[[relation]]` targets.
- **No persistence for derived data.** Wikilinks regex-extracted on-the-fly
  from body text (~0.007s corpus-wide at current scale).

## Scope & Objectives

### 1. Surface authored `[[relation]]` rows at read surfaces

- `memory show` — include a `relations` section (label + target) in text
  and JSON output
- `memory retrieve` — include relations in the output block
- `doctrine inspect` — accept memory refs (`mem_<uid>`, `mem.<key>`) and
  render inbound + outbound edges

### 2. Wikilink extractor + backlinks + graph expansion

- `memory resolve-links [REF]` — regex-extract `[[mem.<key>]]` /
  `[[mem_<uid>]]` from body (skip fenced/inline code). If REF given,
  resolve for one memory; if absent, corpus-wide. Report resolved count,
  dangling count, dangling targets.
- `memory backlinks <REF>` — build reverse index (wikilinks + relations,
  deduped) across all memories, return sources linking to the target
- `memory retrieve --expand N` — BFS from matched memories following the
  union of wikilink targets and relation targets up to depth N; output
  structured nodes with depth annotation

### 3. `record` flag gaps

- `--lifespan <LIFESPAN>` — semantic|episodic|procedural|working|identity
- `--review-by <DATE>` — optional scheduled review date
- `--provenance-source <KIND:REF>` — repeatable, e.g.
  `--provenance-source code:src/lexical.rs`
- `--trust <LEVEL>` — low|medium|high (schema already has trust_level,
  no CLI to set it)
- `--severity <LEVEL>` — critical|high|medium|low|none (schema already has
  ranking.severity, no CLI to set it)

### 4. Lifespan field + filter + ageing

- `lifespan` TOML field — orthogonal to `memory_type` (content form).
  Values: `semantic`, `episodic`, `procedural`, `working`, `identity`.
  Optional, default unset.
  - `semantic` — durable knowledge about the codebase
  - `episodic` — event knowledge: "this happened"
  - `procedural` — how-to, recipes, workflows
  - `working` — current task context, ephemeral
  - `identity` — self-model: subsystem maps, signposts
- `--lifespan` filter on `find` and `retrieve`
- **Ageing in sort key.** The recency component of the sort key is modulated
  by lifespan: `identity` never decays; `semantic` decays slowest;
  `procedural` moderate; `episodic` faster; `working` fastest. A `working`
  memory ages out of ranking quickly; an `identity` memory stays fresh
  regardless of age.

### 5. Suggested relations on record

After `memory record` writes a new memory, score its body against the
existing corpus (reuse BM25 ranker). Report the top N matches: "you might
want to link to: …" — agent can then run `doctrine link` to create edges.

### 6. `memory validate [REF]`

Advisory, never writes. Checks:
- Dangling `[[relation]]` targets (target uid/key doesn't resolve to an
  existing memory)
- Stale `verified_sha` (commit behind HEAD by N commits — detectable via
  git)
- Draft memories past their `review_by` date
If REF given, validate one memory; if absent, corpus-wide.

### 7. Verify `--allow-dirty`

`memory verify --allow-dirty` stamps the working-tree state
(`checkout_state_id`) instead of a clean commit. Without the flag,
behaviour is unchanged: dirty tree → refused. The flag makes the tradeoff
explicit — a dirty-tree attestation is less trustworthy, but enables the
mid-work attest-then-commit workflow.

## Non-Goals (deferred to SL-100)

- `memory status <REF> <STATE>` — lifecycle transitions
- `memory edit <REF>` — multi-field update
- `memory tag <REF> [TAGS]... [-d REMOVE]...` — tag management
- Skill updates (record-memory, retrieve-memory, new maintaining/reviewing/dreaming)

## Non-Goals (out of both slices)

- Changing the storage backend (forgettable compatibility is separate work)
- Modifying the relation write path — memory labels remain free-form per
  `mem.pattern.link.memory-label-fork`
- Catalog scan inclusion of memories (deferred)
- Full staleness computation rewrite
- Embedding / semantic retrieval
- Fields: `audience`, `visibility`, `requires_reading`, `owners`
- Changing `trust_level` to `confidence` (naming, not worth churn)

## Risks & Assumptions

- **Wikilink extraction.** Corpus-wide regex ~0.007s at 193 memories.
  Scales linearly — re-evaluate if it exceeds a human-noticeable threshold.
- **Backward compatibility.** All new TOML fields are optional with
  defaults. Existing memories and queries unaffected.
- **Verify --allow-dirty.** A dirty-tree attestation is less trustworthy
  than a commit-attested one — the flag makes the tradeoff explicit.

## Verification / Closure Intent

- `memory show` and `memory retrieve` include relations for memories that
  have them
- `memory resolve-links [REF]` reports resolved vs dangling wikilinks
- `memory backlinks <REF>` returns correct reverse edges
- `memory retrieve --expand 2` returns expanded graph nodes
- `doctrine inspect mem_<uid>` renders inbound + outbound edges
- New `record` flags round-trip through `show --json`
- `--lifespan` filter on `find`/`retrieve` works correctly
- Ageing correctly modulates sort-key recency per lifespan category
- Suggested relations appears after `record` and is scorable
- `memory validate` catches dangling edges and stale verification
- `verify --allow-dirty` stamps working-tree state; plain `verify` still
  refuses dirty tree
- Existing test suites stay green unchanged
- All new functionality has test coverage (TDD per phase)

## Follow-up: SL-100

Lifecycle verbs (`status`, `edit`, `tag`) + skill updates. Creates a hard
`needs` dependency: SL-100 needs SL-099's new fields to be writable before
it can add verbs to manage them.
