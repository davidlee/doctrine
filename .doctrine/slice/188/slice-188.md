# CLI id-form normalisation: accept both prefixed and bare canonical ids uniformly

## Context

The CLI surface is split into three id-form conventions for the same entity:

1. **PREFIXED-only** (`SL-123`): ~15 verbs via `integrity::parse_canonical_ref`
2. **BARE-only** (`123`): ~16 verbs via raw `u32` arg (no `value_parser`)
3. **BOTH** accepted: ~30 verbs via `governance::parse_entity_ref`

This is the single most expensive recurring friction in the case notes (RFC-011),
appearing 10+ times across multiple sessions. Every agent pays 1-2 retries per
verb first-reach, every session. The AGENTS.md boot guardrail says "cite the
prefixed canonical id everywhere" — but half the CLI surface rejects it, and
the rejection error is opaque (`invalid digit found in string` from
`parse::<u32>`, not "expected bare slice number like 123, not SL-123").

Originates from IMP-227 (`.doctrine/backlog/improvement/227/`).

## Scope & Objectives

Unify the three conventions into one: **accept both prefixed (`SL-123`) and
bare (`123`) forms everywhere**, and provide consistent, helpful error messages
when parsing fails.

### In scope

- **Phase 1** — Eliminate BARE-only: add `value_parser = parse_cli_id` to every
  `u32` arg in `src/slice.rs` (SelectorCommand::{Add, Note, List, Rm}) and
  `src/dispatch.rs` (all `slice: u32` fields in CLI structs).

- **Phase 2** — Eliminate PREFIXED-only: replace `parse_canonical_ref` consumers
  with `parse_entity_ref` in verbs that currently reject bare numbers
  (relation, facets, tags, explore, reseat). Each call site supplies the kind's
  prefix and a human-readable label.

- **Phase 3** — Improve error messages: replace the raw `u32` parse error
  (`invalid digit found in string`) with clap-level errors that cite both
  accepted forms.

### Out of scope

- Changing the internal function signatures of `parse_canonical_ref` itself (it
  remains prefixed-only by design — used where the kind must be explicit from
  the reference string).
- Adding new id forms beyond prefixed and bare.
- Changing how ids are *rendered* (output always uses canonical `PREFIX-NNN`).

## Summary

The unifying function `governance::parse_entity_ref(prefix, label, ref)` already
exists, is tested, and accepts both forms. The work is mechanical: route every
CLI id argument through it.

### Affected surface

| File | What changes |
|---|---|
| `src/slice.rs` | SelectorCommand variants — add `value_parser` to `id: u32` fields |
| `src/dispatch.rs` | All CLI struct fields typed `slice: u32` — add `value_parser` |
| `src/commands/relation.rs` | Replace `integrity::n` with `parse_entity_ref` for `source`/`target` |
| `src/commands/facet.rs` | Replace `integrity::n` with `parse_entity_ref` |
| `src/commands/tag.rs` | Replace `integrity::n` with `parse_entity_ref` |
| `src/commands/dep_seq.rs` | Replace `integrity::n` with `parse_entity_ref` for `source`/`target` |
| `src/integrity.rs` | `run_reseat` already uses `parse_canonical_ref` — switch to `parse_entity_ref` |
| `src/explore.rs` (or equivalent) | inspect / blockers / explain / map focus — `parse_canonical_ref` → `parse_entity_ref` |

### Key design decision

The approach is **Phase 2 Option A** from IMP-227: replace `parse_canonical_ref`
call sites individually with `parse_entity_ref(prefix, label, ref)`. This is
simpler than modifying `parse_canonical_ref` to accept bare numbers (which would
need kind-context injection), and reuses the proven dual-form parser.

### Risk

- **Low risk.** The parser already has dual-form test coverage
  (`governance.rs:1040`). Existing tests should pass unchanged — the change
  only makes the CLI *more* permissive, never less.
- One subtlety: `parse_entity_ref` does a literal prefix strip (not
  case-insensitive), so `AdR-7` would still fail — this is existing behaviour
  and intentional per the function's doc comment.

## Verification

- `doctrine slice selector add SL-188 ...` works (currently: parse error)
- `doctrine dispatch setup --slice SL-188` works (currently: parse error)
- `doctrine link 188 governed_by ADR-001` works (currently: "not a canonical ref")
- `doctrine needs 188 SL-001` works (currently: "not a canonical ref")
- `doctrine review new --facet design --target 188` works (currently: "not a canonical ref")
- All existing tests pass unchanged
- Error messages for invalid ids mention both acceptable forms

## Follow-Ups

- None identified — this is a self-contained CLI normalisation.
