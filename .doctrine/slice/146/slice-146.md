# Config coefficient CLI — doctrine config get/set for priority coefficients, kind_weights, tag_coefficients

## Context

RFC-002's priority scoring program (ADR-015, SL-133) introduced a multi-dimensional
priority scoring engine with configurable coefficients, kind weights, and tag
coefficients stored in `doctrine.toml` under the `[priority]` section. SL-136
then shipped `doctrine tag` for per-entity tag management, and SL-135 shipped
read-side config parsing in `src/priority/config.rs`.

Missing: a CLI surface to inspect or modify these project-wide coefficients.
Currently an operator must hand-edit `doctrine.toml` directly — no `config show`,
no `config set`, no discovery. This is the last un-surfaced piece of the scoring
system: capture exists (config in TOML), computation exists (scoring engine),
but the *consumption surface* (RFC-002 thesis) is absent.

### What already exists

- `src/priority/config.rs` — `PriorityConfig` struct, `load()` reads `[priority]`
  from `doctrine.toml`, clamps coefficients to safe ranges
- `src/dtoml.rs` — THE single shared `doctrine.toml` reader for `[conduct]`,
  `[verification]`, `[estimation]`, `[value]`, `[dispatch]` (does NOT read
  `[priority]` — that's the priority leaf's own reader)
- `src/tag.rs` — `apply_tags_set` root-level edit-preserving write via `toml_edit`
  (SL-136 precedent for editing TOML in-place)
- Current `doctrine.toml` has `[priority.coefficients]` with `value=1.0, risk=1.0`
  and `[priority.consequence]` with `dep_coeff=0.5, ref_coeff=1.0`

## Scope & Objectives

### Phase 1: `doctrine config show --priority` — read surface

A subcommand that dumps the active `[priority]` section with resolved defaults
for absent keys. The output shows both the TOML source values and the clamped
values the scoring engine actually uses (since `load()` clamps silently).
`-P`/`--priority` selects the `[priority]` section.

### Phase 2: `doctrine config set --priority` — write any leaf key

Write any leaf key under `[priority]` (scalars and map entries):
- `doctrine config set --priority coefficients.value <f64>`
- `doctrine config set --priority coefficients.risk <f64>`
- `doctrine config set --priority consequence.dep_coeff <f64>`
- `doctrine config set --priority consequence.ref_coeff <f64>`
- `doctrine config set --priority kind_weights.<KIND> <f64>` — upsert a kind weight
- `doctrine config set --priority tag_coefficients.<TAG> <f64>` — upsert a tag coeff
- `doctrine config set --tag <TAG> <f64>` — shortcut for tag_coefficients

Edit-preserving via `toml_edit` — reads file, mutates the `DocumentMut`,
writes back. Missing sections created via `entry().or_insert()` (safe per
CHR-019 / RV-129). Value is clamped inline and the clamped value is displayed
to the user; unchanged values are a no-op.

### Phase 3: `doctrine config get --priority` — read a single key

- `doctrine config get --priority coefficients.value` — displays the resolved
  value (file value or default if absent)

### Phase 4: `doctrine config unset --priority` — remove any leaf key

Remove any leaf key, restoring the engine default on next `load()`:
- `doctrine config unset --priority coefficients.value` — scalar
- `doctrine config unset --priority kind_weights.<KIND>` — map entry
- `doctrine config unset --priority tag_coefficients.<TAG>` — map entry
- `doctrine config unset --tag <TAG>` — shortcut

Idempotent (absent key is a no-op). Section-level paths (`coefficients`)
are refused — only fully-qualified leaf keys accepted.

All values clamped per `PriorityConfig::load()`'s existing policy (silent clamp,
no hard error).

## Non-Goals

- No `doctrine config` verb for non-priority sections (`[dispatch]`, `[conduct]`,
  `[estimation]`, `[value]`, `[verification]`). The `show`/`set`/`get` arg parser
  is designed to be extensible but this slice only wires `[priority]`.
- No batch/multi-key operations in one invocation (single key per call).
- No diff/history tracking of config changes.
- No validation beyond the scoring engine's existing clamp policy (no schema
  or bounded-range rejection — matches `PriorityConfig::load()`'s tolerance).
- No web/MCP config surface (CLI only).
- No editing of `.doctrine/` entity TOML files or `governance.md`.

## Affected Surface

| Path | Change |
|------|--------|
| `src/commands/config.rs` (new) | `doctrine config` verb — `show`, `set`, `get`, `unset` subcommands |
| `src/commands/cli.rs` | Register `Command::Config` |
| `src/priority/config.rs` | Expose `read_priority_table()` (shared parse), `clamp_general`/`clamp_dep` (pub(crate)) |
| `src/commands/mod.rs` | Declare `pub(crate) mod config;` |
| `doctrine.toml` | Test fixtures may exercise writes |
| `tests/` | E2E golden tests for `config show`/`set`/`get`/`unset` |

## Risks & Assumptions

- **toml_edit edit-preserving works for root-level inserts.** Confirmed by
  CHR-019 / RV-129: `doc.as_table_mut().insert("priority", …)` lands the
  `[priority]` header first, scalar keys inside it. Safe.
- **Existing PriorityConfig load path is tolerant.** Missing `[priority]` section
  → defaults. Unknown keys → ignored. Malformed values → defaults. The write
  path also tolerates missing sections by creating them.
- **`config show` needs dual output** (raw TOML + clamped effective) to avoid
  confusing operators when their hand-entered value got clamped silently.

## Verification / Closure Intent

- `doctrine config show --priority` on a `doctrine.toml` without `[priority]` prints defaults
  with `# default` annotation
- `doctrine config set --priority coefficients.value 2.0` updates the file and
  prints the new clamped value
- `doctrine config set --priority coefficients.value 99e9` clamps to `COEFF_MAX`
  and prints a note about clamping
- `doctrine config get --priority coefficients.value` prints the resolved value
- `doctrine config set --priority kind_weights.SL 3.0` adds the entry
- `doctrine config unset --priority kind_weights.SL` removes the entry
- `doctrine config unset --priority coefficients.value` removes the scalar key (next `load()` restores default `1.0`)
- `doctrine config set --tag "area:cli" 0.9` writes to `tag_coefficients."area:cli"`
- Existing `survey`/`next`/`explain` output uses the new config without restart
  (config is re-read at each invocation via `load()`)

## Open Questions (resolved in design)

- ~~Path syntax~~ → Dot-separated, section-relative (`coefficients.value`). Section selected by `--priority` flag. Confirmed in design.
- ~~`config show` scope~~ → `[priority]` only; `--all` deferred.
- ~~`config show` format~~ → Flattened dotted keys with inline `# default` / `# clamped from N` annotations, subsection header comments, `--json` flag.

## Summary

One new CLI verb (`config`) with four subcommands (`show`, `set`, `get`, `unset`)
targeting the `[priority]` section of `doctrine.toml`. Reuses `toml_edit` patterns
from SL-136 for edit-preserving writes and the existing clamped `PriorityConfig`
for effective-value display. Closes the last consumption-surface gap in the
scoring system (RFC-002).

## Follow-Ups

- Extend to other `doctrine.toml` sections (`[dispatch]`, `[conduct]`, etc.)
- `doctrine config set --dry-run` to preview changes without writing
- `doctrine config show --all` for full-file display
