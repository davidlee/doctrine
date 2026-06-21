# Extend tagging to all entity types — generic cross-kind tag verb

## Context

`doctrine backlog tag` and `doctrine memory tag` exist but use separate,
forked implementations over different TOML storage locations. Most kinds
cannot be tagged at all (no write verb), and `list --tag` silently matches
nothing for governance (the filter reads no tags). Provide one cross-kind
`doctrine tag` verb and make `list --tag` work.

Tag coefficients in IMP-118 (SL-133) are a key motivation — a project-global
per-tag coefficient (default 1.0) feeding graph-traversal prioritisation — but
broader tagging is also a general cross-cutting classification axis for
filtering and grouping. This slice keeps tag normalisation stable so those
coefficient lookups key consistently; it does not build the coefficient.

Design decision (see `design.md` D1): rather than dispatch over per-kind tag
locations, storage is **unified to a root-level `tags` array** for every
taggable kind. Governance/RFC migrate out of `[relationships].tags`. This
collapses the verb to a single code path and makes the list-filter fix fall
out of one shared `Meta` field.

## Scope & Objectives

### Phase 1: Shared write leaf

`tag::apply_tags_set` — a root-level, edit-preserving set-merge write core,
generalised from `backlog::apply_tags` (insert-if-missing, not F-1 bail —
safe at root in toml_edit 0.22, see design D4/§5.5). Hoist
`backlog::fold_filter_tag` into `tag.rs`. Backlog's `apply_tags`/`run_tag`/
list-filter delegate. Behaviour-preserving (one backlog bail-test rewritten to
assert self-heal).

### Phase 2: Generic verb

`doctrine tag set <ID> <TAGS...> [-d/--remove …]` + `doctrine tag clear <ID>`,
across **any resolvable canonical numbered ref** (no whitelist). New
`commands/tag.rs` + `Command::Tag` wiring. `set` mirrors backlog semantics
(additive + `--remove`); `clear` removes all. Memory/non-numbered refs fail
resolution with a friendly redirect.

### Phase 3: Templates + Meta + list filter fix

Seed root `tags = []` in the slice, requirement (REQ), and concept-map (CM)
templates (backlog/knowledge/spec already seeded). Add
`#[serde(default)] tags` to `meta::Meta`; `slice::key()` + `governance::key()`
populate `FilterFields.tags` (governance covers ADR/POL/STD/RFC). Centralise
the lenient `--tag` filter-fold in `listing::build` so every list kind matches
case-insensitively.

### Phase 4: Governance/RFC migration

Strip `tags` from `[relationships]` in the ~29 ADR/POL/STD/RFC files + the 4
templates; drop the typed `tags` from governance's `Relationships`, add root
`tags` to its `Doc`, repoint the `show` render to root. Restore RFC-002's live
tags via one `doctrine tag set RFC-002 …`.

## Non-Goals

- No tag display in list columns (separate display/UX pass).
- No tag completion, suggestion, or hierarchy/namespace beyond charset.
- No tag coefficient (SL-133 / IMP-118 owns it).
- Memory tagging stays on its own forked verb (`[scope].tags`) — out of scope.
- No full hoist of backlog's tag *presentation* into the shared module
  (deferred; `design.md` OQ-1).

## Terrain

| File | Change |
|------|--------|
| `src/tag.rs` | New `apply_tags_set` (root-level write core) + hoisted `fold_filter_tag` |
| `src/commands/tag.rs` (new) | `doctrine tag set`/`clear` handler + dispatch |
| `src/commands/cli.rs` | Register `Command::Tag` |
| `src/backlog.rs` | Delegate `apply_tags`/`run_tag`/list-filter to `tag::*`; rewrite bail-test |
| `src/meta.rs` | `Meta` gains `#[serde(default)] tags`; update test helper |
| `src/slice.rs` | `slice::key()` populates tags |
| `src/governance.rs` | `key()` populates tags; drop `Relationships.tags`; `Doc` root tags; repoint `show`; fix 2 `Meta` literals |
| `src/adr.rs`, `src/policy.rs` | Fix `Meta` struct-literal sites (A2) |
| `src/listing.rs` | Centralise `--tag` filter-fold (calls `tag::fold_filter_tag`) |
| `install/templates/{slice,requirement,concept-map}.toml` | Seed root `tags = []` |
| `install/templates/{adr,policy,standard,rfc}.toml` | Move `tags` to root |
| `.doctrine/{adr,policy,rfc}/**` | Strip stale `[relationships].tags` (~29 files) |

## Dependencies

- `tag::normalize_tag` — shared write chokepoint, already exists (SL-100).
- `integrity::parse_canonical_ref` + `entity::id_path` — the universal
  resolver + path builder (reused, no parallel impl).
- `listing.rs` — `FilterFields` + `tags_admit()` already filter; `--tag` clap
  arg already on shared `ListArgs`.
- IMP-118 (SL-133) — consumes tag data; soft downstream (defaults to 1.0).
