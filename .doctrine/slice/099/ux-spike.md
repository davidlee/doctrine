# UX spike: memory CLI surface

## Status enum already exists (6 variants)

`src/memory.rs` defines: `Active`, `Draft`, `Superseded`, `Retracted`,
`Archived`, `Quarantined`. Hide-set: `Superseded | Retracted | Archived |
Quarantined`.

The enum is richer than what we scoped — `Retracted` is "this was wrong"
(better than `obsolete`), `Quarantined` is "suspected incorrect, pending
review." No need to invent new statuses; the surface just needs verbs to
reach them.

**No verb reaches any non-Active/Draft state after creation.** Every other
entity (knowledge, backlog, ADR, slice) has a `status` or `edit` verb.
Memory has neither.

## Current surface map

| Verb | Flags | Missing |
|---|---|---|
| `record` | --type, --key, --status, --summary, --tag, --path-scope, --glob, --command, --repo, --global | --lifespan, --review-by, --provenance-source, --trust, --severity |
| `show` | --format (table/json) | relations, backlinks |
| `verify` | (none) | --allow-dirty |
| `list` | --type, -f/--filter, -r/--regexp, -s/--status, -t/--tag, --format, --columns | solid, no gaps for current model |
| `find` | --path-scope, --glob, --command, --tag, --query, --type, --status, --limit, etc. | --lifespan filter |
| `retrieve` | (same as find + --min-trust) | --expand N, --lifespan |
| `sync` | install | fine |
| *(missing)* | — | status, edit, tag, resolve-links, backlinks, validate |

## Missing verbs (with stealable patterns)

### `memory status <REF> <STATE>` — minimal
Follow `knowledge status <ID> <STATE>` pattern. Kind auto-detected from
prefix (already works for uid/key refs). No resolution coupling — unlike
backlog, memory status transitions are pure state moves.

Args: reference (uid or key), target state (active|draft|superseded|
retracted|archived|quarantined). Supersede needs a `--by <OTHER_REF>` to
record the replacement.

### `memory edit <REF>` — multi-field update
Follow `backlog edit` shape but with memory-specific fields. Single
invocation updates one or more of: --summary, --title, --status,
--lifespan, --review-by, --trust, --severity. Scope (paths/globs/commands)
via separate flags. Key is immutable after creation (identity).

### `memory tag <REF> [TAGS]... [-d REMOVE]...` — tag management
Direct steal from `backlog tag`. Same semantics: add positional, remove via
-d, sort on write.

### `memory resolve-links [REF]` — wikilink extraction
No direct steal. New verb. If REF given, resolve for one memory; if absent,
resolve all (corpus-wide). Report: resolved count, dangling count, dangling
targets.

### `memory backlinks <REF>` — reverse index
New verb. No direct steal. Compute reverse edges from all bodies (wikilinks
+ relations, deduped), return sources linking to REF.

### `memory validate [REF]` — integrity check
No direct steal. Check: dangling `[[relation]]` targets, stale verified_sha
(detectable via git), draft memories past a review-by date. Like `survey` —
advisory, never writes.

### `record` flag gaps
- `--lifespan <LIFESPAN>` — semantic|episodic|procedural|working|identity
- `--review-by <DATE>` — optional scheduled review
- `--provenance-source <KIND:REF>` — repeatable, e.g. `--provenance-source code:src/lexical.rs`
- `--trust <LEVEL>` — already has a trust_level default but no CLI to set it
- `--severity <LEVEL>` — already in the schema but no CLI flag

### `find` / `retrieve` gaps
- `--lifespan <LIFESPAN>` — filter by cognitive category
- `retrieve --expand <N>` — BFS graph expansion from matched memories

### `show` gaps
- Relations section (authored `[[relation]]` rows)
- Backlinks section (on-the-fly reverse wikilinks + relations)
- `doctrine inspect` — accept memory refs

## Proposed slice split

### Slice A: SL-099 — Read-path + Data-model (keep current scope, trim)
- Surface relations in `show` / `retrieve` output
- Wikilink extractor + `resolve-links`
- `backlinks` command
- `--expand N` on retrieve
- `inspect` for memory refs
- `--lifespan`, `--review-by`, `--provenance-source` flags on `record`
- `--lifespan` filter on `find` / `retrieve`
- Ageing in sort key (identity > semantic > procedural > episodic > working)
- Suggested relations on `record` (BM25 score against existing corpus)
- `validate` — dangling edges + stale verification
- Verify `--allow-dirty`

*This is the data-model + read-path + query-surface work. Pure and impure
layers, no skill changes.*

### Slice B: SL-100 — Agent UX + Lifecycle Surface
- `memory status <REF> <STATE> [--by <OTHER>]`
- `memory edit <REF>` (summary, title, status, lifespan, review-by, trust,
  severity)
- `memory tag <REF> [TAGS]... [-d REMOVE]...`
- Skill updates: record-memory, retrieve-memory
- New skills: maintaining-memory, reviewing-memory (skeletons)

*This is the lifecycle management + agent workflow hardening work. CLI
verbs + skills, no data-model changes.*

The split is clean: Slice A adds fields and query surfaces; Slice B adds
verbs to manage them. B depends on A (needs the fields A adds to exist),
but A is independently shippable and testable.
