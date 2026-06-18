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

**New skill: reviewing-memory**

Skeleton with tracked headings. Structured audit for stability gates:
before releases, migrations, large refactors, or when agent confusion
detected. Procedure:
1. Pull highest-impact via `memory validate` corpus-wide
2. Prioritize: scoped + attested + high commit count
3. Apply checklist: provenance, freshness, metadata efficiency, scope,
   actionability, duplication
4. Thread hygiene: archive/convert lingering threads
5. Produce outcomes, not notes — every reviewed item ends in verified,
   corrected, superseded, archived, or promoted

**New skill: dreaming**

Unified memory corpus maintenance posture — covers both reactive
(change-triggered: files move, commands change, invariants shift,
duplicates found) and proactive (periodic/idle-time improvement). One
skill, two entry paths.

Procedure:
1. **Validate.** Run `memory validate` corpus-wide. For each finding,
   decide: fix it now, capture as backlog item, or note and defer.
2. **Prune.** Identify memories past their `review_by` date, unverified
   threads past expiry, `working`-lifespan memories older than N days.
   Archive or retract stale ones.
3. **Link.** For recently-recorded or recently-edited memories, run
   suggested relations (or corpus-wide BM25 pairwise for uncapped
   exploration). Run `doctrine link` for high-confidence matches.
   Check for orphaned memories (no inbound edges, no outbound edges) —
   these may need scope or relations to be findable.
4. **Backlog grooming.** Findings from validation, pruning, or fact-checking
   that aren't immediately fixable become backlog items (risks for
   not-yet-surfaced issues, chores for cleanup, improvements for
   enhancements). Don't let discoveries evaporate.
5. **Fact-check.** Spot-check high-severity memories against current
   code/docs. Pick a sample (e.g. top 5 by severity × staleness). Verify
   the claim still holds. If not: correct the memory (`memory edit`),
   supersede it, or flag it for human review (`quarantined`).
6. **Report.** Produce a brief summary of what was done, what was found,
   and what was deferred to the backlog. The report is the handoff — the
   next agent shouldn't re-do the same checks.

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
