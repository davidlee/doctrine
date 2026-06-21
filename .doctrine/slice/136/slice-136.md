# Extend tagging to all entity types — generic cross-kind tag verb

## Context

`doctrine backlog tag` and `doctrine memory tag` exist but use separate,
forked implementations with different TOML storage locations. Seven entity
kinds need tagging: backlog (done), memory (done, forked), ADR/POL/STD/RFC
(have tags in schema, no write verb, broken list filter), knowledge (has
tags, no write verb), spec (has tags, no write verb), slice (no tags at
all).

Tag coefficients in IMP-118 (SL-133) are a key motivation — but broader
tagging is also a general cross-cutting classification axis for filtering
and grouping.

## Scope & Objectives

### Phase 1: Generic write verb

`doctrine tag set <ID> <TAGS...>` + `doctrine tag clear <ID>` — works
across all entity kinds. Resolves the tag storage location per-kind:

| Kind | Tag location in TOML |
|------|---------------------|
| Backlog | root-level `tags` |
| Memory | `[scope].tags` |
| ADR/POL/STD/RFC | `[relationships].tags` |
| Knowledge | root-level `tags` |
| Spec (PRD/SPEC) | root-level `tags` |
| Slice | root-level `tags` (new) |

Reuses `tag::normalize_tag` (shared) and `tag::apply_tags_set` (extracted
from backlog's `apply_tags` into a kind-parameterised leaf).

### Phase 2: Slice model addition

Add `tags = []` to the slice TOML scaffold. Add `tags: Vec<String>` to
`SliceDoc` struct. Migration: existing slices with no `tags` key default to
empty.

### Phase 3: Governance list filter fix

`governance::key()` currently returns `Vec::new()` for tags (Codex
BLOCKER-2). Read tags from `[relationships].tags` and populate the
`FilterFields` so `list --tag` works.

### Phase 4: Per-kind tag subcommands (nice-to-have)

`doctrine adr tag` / `doctrine spec tag` / `doctrine rfc tag` — thin
wrappers around the generic verb, for discoverability.

## Non-Goals

- No tag display in list columns (that's a separate display/UX pass)
- No tag completion or suggestion
- No tag hierarchy/namespace enforcement beyond charset
- No migration of existing knowledge/spec/requirement tags (they already
  parse correctly)

## Terrain

| File | Change |
|------|--------|
| `src/tag.rs` | Extract `apply_tags_set` from backlog — kind-parameterised write leaf with per-kind tag location |
| `src/commands/tag.rs` (new) | `doctrine tag set`/`clear` CLI handler |
| `src/main.rs` | Register `Tag` subcommand |
| `src/slice.rs` | Add `tags` field to `SliceDoc`, scaffold template |
| `src/governance.rs` | Fix `key()` to return tags from `[relationships].tags` |
| `install/templates/slice.toml` | Add `tags = []` |
| `src/backlog.rs` | Refactor `apply_tags` to call shared `tag::apply_tags_set` |
| `src/memory.rs` | Refactor memory tag write to call shared leaf (optional) |

## Dependencies

- `src/tag.rs` — shared `normalize_tag` already exists
- Tag validation charset `[a-z0-9_:-]` — shared, unchanged
- `listing.rs` — `FilterFields` + `tags_admit()` already support filtering
- IMP-118 (SL-133) — tag coefficients consume tag data; this slice is a
  soft prerequisite (defaults to 1.0 when absent)
