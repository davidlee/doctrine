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

1. **Pre-parse structural check** — a function that scans raw TOML text for
   non-contiguous `[section]` / `[[array]]` headers, producing a clear
   diagnostic: `"SL-007: section [relationships] appears non-contiguously
   (lines 5, 15) — merge the sections"`. The check must handle string bodies
   (multi-line basic strings, literal strings) and inline tables correctly
   — no false positives on headers that appear inside values.

2. **Augment `doctrine validate`** — add the non-contiguity check to
   `integrity::scan_kind` so `validate` catches these proactively. The
   finding is a hard error (non-zero exit) — non-contiguous TOML is always
   a problem.

3. **Shared entity-parse wrapper** — introduce a function (e.g.
   `dtoml::parse_entity_toml`) that wraps `toml::from_str`, injects the
   canonical entity id into parse errors, and calls the structural check
   first. Route the main read paths (`read_meta`, `read_slice`,
   `read_item`, `read_record`, `read_doc`) through it. Low-risk read
   paths (config, dispatch, lazyspec) may remain on raw `toml::from_str`.

## Non-Goals

- **Normalisation / auto-fix on read** (Approach A in ISS-049). Merging
  non-contiguous sections silently is lossy and violates doctrine's
  no-silent-data-loss principle. May be considered as a `--fix` mode later
  but is explicitly out of scope.
- **TOML well-formedness validation beyond non-contiguous sections.**
  Other parse errors (syntax, type mismatch) are caught by the parser
  already; this slice only adds the proactive check for the
  non-contiguous-section class of errors.
- **Write-seam changes.** The write paths use `toml_edit::DocumentMut`
  which always produces contiguous output. Non-contiguous TOMLs come from
  manual edits; prevention at write time is out of scope.

## Summary

Three changes, sequenced:
1. Add `dtoml::validate_sections(text) → Vec<SectionFinding>`
2. Add non-contiguity check to `integrity::scan_kind` → feeds `validate`
3. Add shared `dtoml::parse_entity_toml<T>(text, entity_ref) → T` and
   route main read paths through it for canonical-id error context

## Risks

- **TOML header detection fragility.** A regex/line-based scan that doesn't
  understand TOML string bodies can false-positive on `[section]` text
  inside multi-line strings or literal strings. Mitigation: the scan must be
  string-body-aware (skip `"""..."`"` and `'''...'''` bodies, inline
  tables). `mem.pattern.parse.toml-error-classification-fragile` records that
  error classification by text matching is version-fragile — canary tests
  must pin the observed shapes.
- **IMP-109 sequencing.** IMP-109 ("catalog scan: read each entity TOML
  once") may change the parse topology in `catalog/scan.rs`. If IMP-109 lands
  first, the shared-wrapper surface adjusts; if SL-151 lands first, IMP-109
  benefits from it. Both should work independently.

## Affected Surface

| Module | Change |
|---|---|
| `src/dtoml.rs` | New: `validate_sections`, `parse_entity_toml` |
| `src/integrity.rs` | Add non-contiguity check to `scan_kind` |
| `src/commands/validate.rs` | Wire new findings into output |
| `src/meta.rs` | Route `read_meta`/`read_id` through shared wrapper |
| `src/slice.rs` | Route `read_slice` through shared wrapper |
| `src/backlog.rs` | Route `read_item` through shared wrapper |
| `src/knowledge.rs` | Route `read_record` through shared wrapper |
| `src/governance.rs` | Route `read_doc` through shared wrapper |

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
