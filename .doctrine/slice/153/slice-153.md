# CLI verbs for spec-internal edges (descends_from, parent, interactions)

## Context

All 23 `(label, source-set)` rule rows in `RELATION_RULES` (`src/relation.rs`) have
CLI verb coverage except three spec-internal edges:

- **`descends_from`** (SPEC→PRD) — scalar `descends_from = "PRD-NNN"` in spec TOML.
  No CLI subcommand; hand-edited today.
- **`parent`** (SPEC→SPEC, and PRD→PRD per SL-065) — scalar `parent = "<ref>"` in
  spec TOML. Same gap. Subtype-aware: target must equal the source subtype.
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
   fields via flags: `--descends-from PRD-NNN | --clear-descends-from` and
   `--parent <ref> | --clear-parent` (≥1 required; set/clear per field mutually
   exclusive). Edit-preserving (a new `dep_seq::apply_scalar` core mirroring
   `apply_status`; CHR-019 proved root insert is safe above trailing `[[relation]]`).
   `descends_from` is tech-only (target → PRD); `parent` is subtype-aware (target
   subtype == source subtype). Forward-validate kind-shape AND target existence.
   Idempotent (re-set same value / clear-absent → no-op, mtime holds).

2. **`doctrine spec interactions add`** — append an `[[edge]]` row to a tech
   spec's `interactions.toml` (`spec.rs::append_member` shape). Each edge carries
   `target = "SPEC-NNN"`, `--type <text>` (required free-text), `--notes <text>`
   (optional), per the `Interaction` struct. Forward-validate the target as a SPEC
   kind + existence. Idempotent on **target** (target-as-PK: one edge per target).

3. **`doctrine spec interactions remove`** — remove the `[[edge]]` row(s) to a
   target (`dep_seq::remove_after` shape). Idempotent (absent → no-op, count 0); no
   target validation (removing a dangling edge is valid).

Affected surface:
- `src/spec.rs` — `SpecCommand::Edit` + `SpecCommand::Interactions{Add,Remove}`
  clap shapes and thin `run_*` shells; the `interactions.toml` write/remove
  (currently read-only). (NB: the dispatch lives in `src/spec.rs`, not the empty
  `src/commands/spec.rs`.)
- `src/dep_seq.rs` — new `apply_scalar` pure core (set/clear one top-level optional
  scalar, edit-preserving) for `descends_from`/`parent`.
- `src/relation.rs` — no changes; these are typed-tier edges already declared
  `TypedVerbOnly`. (Product `parent` (PRD→PRD) is authorable but undeclared in
  `RELATION_RULES` — left for the follow-up, validated inline here.)

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

- **UX review of all relation-authoring CLI surfaces** (consistency + coverage) —
  absorbs the `RELATION_RULES` product-`parent` (PRD→PRD) under-declaration: SL-065
  added the field + render + registry validation but no table row and no template
  example. This slice validates product `parent` inline and leaves the table
  untouched (Non-Goal). The review closes the whole gap (table honesty + the
  PRD-parent row + VT-1 golden, and any other surface where a relation verb is
  missing or inconsistent). Captured as a backlog item.
