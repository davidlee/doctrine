# SL-151: Non-contiguous TOML sections cause opaque parse failures

## Context

`toml` 0.8.23 rejects duplicate table headers per the TOML spec. If an
entity TOML has `[relationships]` appearing after `[[relation]]` rows (or
any repeated `[section]`), both `toml::from_str` and `toml_edit` fail with
`"duplicate key relationships"` — a raw parse error that names the file path
but not the entity's canonical id, and offers no remediation hint.

All 25 entity kinds are affected. The `validate` command currently checks id
integrity + relation edges but NOT TOML well-formedness, so a non-contiguous
TOML is invisible until another verb tries to read it.

Detailed blast-radius analysis: ISS-049.

## Scope & Objectives

1. **Shared entity-parse wrapper** — introduce `dtoml::parse_entity_toml<T>(text, prefix, id)` that wraps `toml::from_str`, injects the canonical entity id into parse errors. No speculative remediation hint — the raw `toml` error already describes *what* went wrong; the wrapper adds *which entity*. Route the main read paths (`read_meta`, `read_slice`, `read_item`, `read_record`, `read_doc`) through it.

2. **Augment `doctrine validate`** — add a schema-agnostic full-Toml parse
   (`toml::from_str::<toml::Value>`) to `integrity::scan_kind` so `validate`
   catches non-contiguous TOML proactively. The finding is a hard error
   (non-zero exit). No catalog performance impact — `scan_kind` is only
   called by `validate`, not by `scan_entities`.

## Non-Goals

- **Normalisation / auto-fix on read** (Approach A in ISS-049). Merging
  non-contiguous sections silently is lossy and violates doctrine's
  no-silent-data-loss principle. May be considered as a `--fix` mode later
  but is explicitly out of scope.
- **Pre-parse structural scanner.** The `toml` crate's parser already detects
  non-contiguous sections — no separate lexer needed. The wrapper just
  improves error context (canonical id) around the existing parse.
- **TOML well-formedness validation beyond parse errors.** Other parse errors
  (syntax, type mismatch) are caught by the parser already; this slice adds
  canonical-id error context and the proactive `validate` check.
- **Write-seam changes.** The write paths use `toml_edit::DocumentMut`
  which always produces contiguous output. Non-contiguous TOMLs come from
  manual edits; prevention at write time is out of scope.

## Summary

Two changes, sequenced:
1. Add `dtoml::parse_entity_toml<T>(text, prefix, id) → T` — wraps
   `toml::from_str` with canonical-id error context
2. Add `toml::from_str::<toml::Value>` parse to `integrity::scan_kind` —
   feeds `validate` with proactive TOML well-formedness detection

Route `read_meta`, `read_id`, `read_slice`, `read_item`, `read_record`,
`read_doc` through the shared wrapper. No structural scanner — the parser
already detects non-contiguous sections; the wrapper just names the entity.

## Risks

- **Caller surface width.** `read_meta`/`read_id` callers span ~13 modules —
  each needs a `prefix` argument added. The change per caller is mechanical
  (one `&str` argument), but the plan should account for the breadth.
- **IMP-109 sequencing.** IMP-109 ("catalog scan: read each entity TOML
  once") may change the parse topology in `catalog/scan.rs`. SL-151's changes
  are orthogonal (entity read paths, not catalog scan) — no conflict, but
  IMP-109 may want to adopt `parse_entity_toml` later.

## Affected Surface

| Module | Change |
|---|---|
| `src/dtoml.rs` | New: `parse_entity_toml<T>(text, prefix, id) → T` |
| `src/meta.rs` | Route `read_meta`/`read_id` through shared wrapper; add `prefix` param |
| `src/integrity.rs` | `scan_kind`: add full-Toml parse (`toml::Value`) for validate |
| `src/slice.rs` | Route `read_slice` through shared wrapper |
| `src/backlog.rs` | Route `read_item` through shared wrapper |
| `src/knowledge.rs` | Route `read_record` through shared wrapper |
| `src/governance.rs` | Route `read_doc` through shared wrapper |
| ~13 callers of `read_meta` | Add `prefix` argument (mechanical, one `&str` per call) |

## Verification

- **VT**: Unit tests for `validate_sections` with well-formed TOML,
  non-contiguous TOML, TOML with `[section]` inside string bodies
  (no false positive), and empty TOML.
- **VT**: Unit tests for `parse_entity_toml` — error message includes
  canonical entity id.
- **VT**: Integration test — `doctrine validate` exits non-zero on a
  fixture with non-contiguous TOML.
- **VA**: `just gate` (clippy zero warnings) across workspace.
- **VH**: Manual validation — edit an entity TOML to be non-contiguous,
  run `doctrine validate` and confirm clear diagnostic with canonical id.

## Follow-Ups

- IMP-080: standalone plan.toml validation (late-surface parse failure,
  different target — plan is not an entity TOML but benefits from the
  shared-wrapper pattern).
- IMP-109: single-parse catalog scan — may want to adopt the shared
  wrapper.
- Normalisation `--fix` mode (Approach A) — if demand surfaces after
  diagnostic-only lands.
