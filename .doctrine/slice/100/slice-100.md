# Memory lifecycle verbs and agent UX hardening

## Context

SL-099 adds the read-path surfaces, wikilink extraction, backlinks, new
`record` flags, the `lifespan` field, ageing, suggested relations, and
`validate`. This slice adds the lifecycle management verbs (`status`,
`edit`, `tag`) and updates the agent skills that guide memory creation,
retrieval, and maintenance.

The Status enum (in `src/memory.rs`) already defines 6 variants: `Active`,
`Draft`, `Superseded`, `Retracted`, `Archived`, `Quarantined`. Only
`Active` and `Draft` are reachable at record time. Every other entity
surface (knowledge, backlog, ADR, slice) has a `status` or `edit` verb —
memory has neither. Memories are write-once: the only way to update any
field is hand-editing TOML.

This slice is a hard `needs` dependency on SL-099 — the verbs it adds
depend on the fields SL-099 introduces (`lifespan`, `review_by`,
`provenance.sources`, `trust`/`severity` flags).

## Scope & Objectives

### 1. `memory status <REF> <STATE> [--by <OTHER>]`

Follow `knowledge status <ID> <STATE>` pattern. Kind auto-detected from
uid/key ref. Target state must be in the memory Status vocabulary.

`--by <OTHER_REF>` is required for `superseded` (records the replacement
memory) and forbidden otherwise.

### 2. `memory edit <REF>`

Multi-field update. Single invocation updates one or more non-identity
fields:

- `--title <TITLE>` — update the display title
- `--summary <SUMMARY>` — one-line summary
- `--status <STATUS>` — lifecycle transition (alternative to `status` verb
  for single-step workflows)
- `--lifespan <LIFESPAN>` — update cognitive category
- `--review-by <DATE>` — schedule or clear review
- `--trust <LEVEL>` — low|medium|high
- `--severity <LEVEL>` — critical|high|medium|low|none
- `--key <KEY>` — update the key alias

Scope fields (paths, globs, commands) updated via separate flags matching
`record`: `--path-scope`, `--glob`, `--command`. Tags via `memory tag`.

Key is identity — once set at record time, immutable. At least one flag
required.

### 3. `memory tag <REF> [TAGS]... [-d REMOVE]...`

Direct steal from `backlog tag`. Positional args add tags; `-d` removes.
Tags lowercased, validated `[a-z0-9_:-]`. Stored set sorted. At least one
add or remove required.

### 4. Skill updates

**Record-memory skill (§2):**
- After recording a memory, guide agents to check suggested relations and
  run `doctrine link` to create edges
- Mention both `[[relation]]` edges and inline `[[mem.…]]` wikilinks — use
  edges for durable graph structure, wikilinks for contextual "see also"
- Mention `--lifespan` flag and provide selection guidance: pick the
  narrowest lifespan that fits

**Record-memory skill (§4, risk axes):**
- `--trust` and `--severity` are now CLI flags — update docs to reflect
  flags instead of "edit the TOML"

**Retrieve-memory skill:**
- Mention relations, backlinks (`--backlinks`), graph expansion
  (`--expand N`), `--lifespan` filter
- Add a connection-making step: after retrieving, check relations and
  follow relevant edges
- Mention `memory validate` for checking drift before acting on old
  memories

**New skill: maintaining-memory**

Skeleton with tracked headings. Trigger: files move, commands change,
invariants shift, memories stale or wrong, duplicates found. Operations:
1. Locate impacted memories via scope/tags
2. Validate against current code/docs/ADRs (use `memory validate`)
3. Apply minimal corrective edits (update `verified`, adjust scope,
   fix pointers)
4. Handle lifecycle: `memory status <ID> superseded --by <OTHER>`,
   `archived`, `retracted`
5. Re-scope if retrieval misses
6. De-duplicate: merge into one canonical + signpost
7. Sanity check: re-run same query, confirm corrected record ranks

**New skill: reviewing-memory**

Skeleton with tracked headings. Structured audit for stability gates:
before releases, migrations, large refactors, or when agent confusion
detected. Procedure:
1. Pull highest-impact via `--stale` (or `memory validate` corpus-wide)
2. Prioritize: scoped + attested + high commit count
3. Apply checklist: provenance, freshness, metadata efficiency, scope,
   actionability, duplication
4. Thread hygiene: archive/convert lingering threads
5. Produce outcomes, not notes — every reviewed item ends in verified,
   corrected, superseded, archived, or promoted

## Non-Goals

- Changing the Status enum (it already has the right variants)
- Unified `edit` across all entity kinds
- `memory delete` (destructive — no current need)
- Bulk operations (edit-all-by-tag, etc.)

## Dependencies

Hard `needs` SL-099 (read-path + data-model). The verbs added here operate
on fields SL-099 introduces.

## Verification / Closure Intent

- `memory status <REF> <STATE>` transitions status, refuses invalid states
- `memory status <REF> superseded --by <OTHER>` records the replacement
- `memory edit <REF> --summary "..." --lifespan semantic` updates fields
  atomically
- `memory tag <REF> foo -d bar` adds and removes correctly
- Skills committed and surfacing via `/retrieve-memory` queries
- Existing test suites green unchanged
- All new functionality has test coverage (TDD per phase)
