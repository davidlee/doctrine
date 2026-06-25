# CLI verbs for spec-internal edges (descends_from, parent, interactions)

## Context

All 23 `(label, source-set)` rule rows in `RELATION_RULES` (`src/relation.rs`) have
CLI verb coverage except three spec-internal edges:

- **`descends_from`** (SPEC→PRD) — scalar `descends_from = "PRD-NNN"` in spec TOML.
  No CLI subcommand; hand-edited today.
- **`parent`** (SPEC→SPEC) — scalar `parent = "SPEC-NNN"` in spec TOML. Same gap.
- **`interactions`** (SPEC→SPEC) — `interactions.toml` `[[edge]]` array. No CLI
  subcommand to add/remove edges; hand-edited today.

(`mem_019efe57896f` inventories the full relation-edge CLI coverage as of
2026-06-25.)

These three are the last hand-edit-only relation edges in the corpus. Every other
edge can be authored via `doctrine link`, a specific typed verb (`supersede`,
`spec req add`, `review new --target`, `rec new --owning-slice/--decision`,
`revision new --originates-from`, `revision change add`, `concept-map add`), or
both.

## Scope & Objectives

Add CLI subcommands to `doctrine spec` so that a spec's `descends_from`, `parent`,
and `interactions` edges can be authored and removed without hand-editing TOML:

1. **`doctrine spec edit`** — set or clear the `descends_from` and `parent` scalar
   fields on a spec. Edit-preserving (the `toml_edit`/`DocumentMut` idiom from
   `relation::append_edge`/`mem.pattern.entity.edit-preserving-status-transition`).
   Forward-validate target refs against the kind registry (SPEC→PRD for
   descends_from, SPEC→SPEC for parent). Idempotent on re-set to same value.

2. **`doctrine spec interactions add`** — append an `[[edge]]` row to a tech
   spec's `interactions.toml`. Each edge carries `target = "SPEC-NNN"` and
   `type = "free-text"` (the free-text interaction type, per the existing
   `Interaction` struct in `src/spec.rs`). Forward-validate the target as a
   SPEC kind. Idempotent on duplicate.

3. **`doctrine spec interactions remove`** — remove a matching `[[edge]]` row
   from `interactions.toml`. Idempotent (absent edge is a no-op).

Affected surface:
- `src/commands/spec.rs` — new subcommand handlers and CLI args
- `src/spec.rs` — may need a write seam for `interactions.toml` (currently only
  reads); edit-preserving scalar write for `descends_from`/`parent`
- `src/relation.rs` — no changes needed; these are typed-tier edges, the
  `RELATION_RULES` table already declares them `TypedVerbOnly`

## Non-Goals

- Not adding `link` writability for these labels — they remain `TypedVerbOnly`
  per the existing `RELATION_RULES` table (designed for typed verbs).
- Not changing the storage shape of `descends_from`/`parent`/`interactions` —
  they stay in their current tier-2/3 structures.
- Not adding a general-purpose `spec edit` command for arbitrary metadata —
  only the fields that represent relation edges.
- Not migrating existing hand-edited data — the commands must be idempotent
  with existing on-disk state.

## Summary

Three small, targeted CLI subcommands that close the last hand-edit-only gap in
the relation-edge vocabulary: `spec edit` (for descends_from/parent scalars) and
`spec interactions add/remove` (for interactions.toml edges). Each follows the
established edit-preserving, idempotent, forward-validated pattern.

## Shipped memory update

Once the CLI verbs are implemented, update the shipped signpost memory
`mem.signpost.doctrine.relating-entities` (`memory/mem.signpost.doctrine.relating-entities/memory.md`)
to replace the stale "What still requires hand-editing" section with current
inventory:
- drop the stale examples (slice-to-ADR, product-to-product — now covered by
  `governed_by`/`consumes` via `doctrine link`)
- list the three new `spec` subcommands as the CLI surface for
  `descends_from`/`parent`/`interactions`

Shipped-memory authoring flow (`mem.pattern.distribution.shipped-memory-authoring`):
1. Edit `memory/<key>/memory.md`
2. `touch src/corpus.rs && cargo build` (RustEmbed re-embed)
3. `doctrine memory sync` (materialise into `.doctrine/memory/shipped/`)

## Follow-Ups

(none identified)
