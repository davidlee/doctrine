# SL-151 design: Non-contiguous TOML sections — clear diagnostics

## Decisions

### D1 — Shared entity-parse wrapper, no structural pre-scan

`dtoml::parse_entity_toml<T: DeserializeOwned>(text, prefix, id) → T` wraps
`toml::from_str` and injects canonical-entity-id context on parse failure.
No separate structural scanner — `toml::from_str` *is* the detection; the
wrapper just improves the error message by naming the entity.

**Why no remediation hint.** The raw `toml` error already says
`"duplicate key 'relationships'"` + line number — it tells the user *what*
went wrong. The wrapper adds *which entity* (`SL-007`). Adding a speculative
hint ("check for non-contiguous sections") would be misleading for other
parse error types (type mismatch, invalid escape). The canonical-id context
is the prize; the raw error is already informative.

**Why not a pre-parse structural scan.** The catalog already parses every
TOML; adding a separate partial lexer for non-contiguous-header detection
would duplicate that parse with real corner-case risk (string bodies, inline
tables). The wrapper costs one `with_context` call per parse — zero marginal
performance impact.

### D2 — `validate` augments `scan_kind` with a schema-agnostic full-Toml parse

After the existing `meta::read_id` (id-only scan), `scan_kind` does a second
parse as `toml::Value` — schema-agnostic, catches any well-formedness error
including non-contiguous sections. Review's status-less TOML parses fine as
`toml::Value` (it's valid TOML, just missing a `status` key for
`read_meta`'s typed deserialize).

**Why `toml::Value` not `read_meta`.** `read_meta` hard-fails on review
(missing `status`) — the whole reason `read_id` exists. `toml::Value` is
schema-agnostic and parses any valid TOML regardless of which keys are
present.

**Performance isolation.** `scan_kind` is only called by `validate` (not by
the catalog's `scan_entities`). No catalog performance impact.

### D3 — Route main read paths through shared wrapper, not every path

Five read paths get the wrapper: `meta::read_meta`, `meta::read_id`,
`slice::read_slice`, `backlog::read_item`, `knowledge::read_record`,
`governance::read_doc`. These are the high-traffic entity-read paths where a
user encountering a non-contiguous TOML needs the canonical-id diagnostic.

Lower-traffic or specialised reads (`review.rs`, `rec.rs`, `revision.rs`,
`concept_map.rs`, `memory.rs`, `lazyspec.rs`, `dispatch.rs`, `dtoml.rs`
config) are out of scope — they can adopt the wrapper later.

## API

### `dtoml::parse_entity_toml`

```rust
/// Parse an entity TOML with canonical-id error context.
///
/// Wraps `toml::from_str`. On parse failure, injects the entity's canonical
/// id so the user sees which entity is broken. The raw `toml` error already
/// describes *what* went wrong.
///
/// Pure leaf (ADR-001): owned text in, no IO, no config dependency.
pub(crate) fn parse_entity_toml<T: DeserializeOwned>(
    text: &str,
    prefix: &str,
    id: u32,
) -> anyhow::Result<T> {
    toml::from_str(text).with_context(|| {
        format!("{prefix}-{id:03}: TOML parse failed")
    })
}
```

**Error shape (example):**

```
SL-007: TOML parse failed

Caused by:
    TOML parse error at line 15, column 1:
      |
    15 | [relationships]
      | ^
    duplicate key `relationships` in document root
```

The canonical-id prefix (`SL-007`) is the new information. The raw `toml`
error already identifies the problem and line number.

### `integrity::scan_kind` — signature change

```rust
fn scan_kind(
    root: &Path,
    kind: &'static KindRef,
    diagnostics: &mut Vec<String>,   // NEW parameter
) -> anyhow::Result<KindSnapshot>
```

After the existing `read_id` call, reads the full TOML text and parses as
`toml::Value`. Any parse error is pushed to `diagnostics` with canonical-id
context. Existing id-integrity checks are unchanged.

### `read_meta` / `read_id` — signature change

```rust
// Before:
pub(crate) fn read_meta(tree_root: &Path, stem: &str, id: u32) -> anyhow::Result<Meta>

// After:
pub(crate) fn read_meta(tree_root: &Path, stem: &str, id: u32, prefix: &str) -> anyhow::Result<Meta>
```

Each caller (per-kind `show`/`list`) passes its known prefix. The wrapper
replaces `toml::from_str(&text).with_context(...)`.

## Affected surface

| File | Change |
|---|---|
| `src/dtoml.rs` | **New:** `parse_entity_toml<T>(text, prefix, id) → T` |
| `src/meta.rs` | `read_meta`: swap raw `toml::from_str` for `dtoml::parse_entity_toml`. `read_id`: same. Add `prefix` parameter to both. |
| `src/integrity.rs` | `scan_kind`: add `diagnostics: &mut Vec<String>` param; after `read_id`, do `toml::from_str::<toml::Value>(&text)`, push errors. `id_integrity_findings`: allocate `diagnostics`, pass to `scan_kind`, append to findings. |
| `src/slice.rs` | `read_slice`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, "SL", id)` |
| `src/backlog.rs` | `read_item`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, kind.prefix(), id)` |
| `src/knowledge.rs` | `read_record`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, kind.prefix(), id)` |
| `src/governance.rs` | `read_doc`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, &g.kind.prefix, id)` |
| Callers of `read_meta`/`read_id` | Add `prefix` argument — each caller knows its kind's prefix |

## Caller survey for `read_meta` / `read_id`

`read_meta` callers (each needs `prefix` added):

| Caller | Prefix |
|---|---|
| `src/slice.rs` (show/list) | `"SL"` |
| `src/adr.rs` (show/list) | `"ADR"` |
| `src/policy.rs` (show/list) | `"POL"` |
| `src/standard.rs` (show/list) | `"STD"` |
| `src/spec.rs` (show/list, product) | `"PRD"` |
| `src/spec.rs` (show/list, tech) | `"SPEC"` |
| `src/rfc.rs` (show/list) | `"RFC"` |
| `src/revision.rs` (show/list) | `"REV"` |
| `src/review.rs` (show/list) | `"RV"` |
| `src/requirement.rs` (show/list) | `"REQ"` |
| `src/rec.rs` (show/list) | `"REC"` |
| `src/backlog.rs` (show/list, all 5 kinds) | `"ISS"`/`"IMP"`/`"CHR"`/`"RSK"`/`"IDE"` |
| `src/knowledge.rs` (show/list, all 4 kinds) | `"ASM"`/`"DEC"`/`"QUE"`/`"CON"` |
| `src/integrity.rs` (reseat) | Via `kind.kind.prefix` |

`read_id` callers:

| Caller | Prefix |
|---|---|
| `src/integrity.rs` (`scan_kind`) | `kind.kind.prefix` |

## Not changed (out of scope)

| File | Reason |
|---|---|
| `catalog/scan.rs` | Already wraps errors into `CatalogDiagnostic` with entity key |
| `review.rs`, `rec.rs`, `revision.rs` read paths | Lower-traffic; can adopt wrapper later |
| `concept_map.rs`, `memory.rs`, `lazyspec.rs`, `dispatch.rs`, `dtoml.rs` (config) | Config/specialised reads, not entity TOMLs |
| Write paths | Use `toml_edit::DocumentMut` — always contiguous output |

## Verification

| ID | Kind | What it proves |
|---|---|---|
| VT-1 | Unit | `parse_entity_toml` passes through valid TOML identically to raw `toml::from_str` |
| VT-2 | Unit | `parse_entity_toml` on non-contiguous TOML errors with `"{prefix}-{id:03}: TOML parse failed"` prefix |
| VT-3 | Unit | `scan_kind` flags a non-contiguous `[relationships]` in a fixture |
| VT-4 | Unit | `scan_kind` produces no diagnostics on valid TOML |
| VT-5 | Integration | `doctrine validate` exits non-zero on a non-contiguous entity TOML fixture |
| VT-6 | Integration | `doctrine show SL-NNN` on non-contiguous TOML errors with canonical id in message |
| VA-1 | Agent | `just gate` — clippy zero warnings across workspace |
| VH-1 | Human | Manually edit an entity TOML to be non-contiguous, run `doctrine validate`, confirm clear diagnostic with canonical id |

## Open questions

None — all three design decisions (D1–D3) are locked by user approval.

## Risks

- **`mem.pattern.parse.toml-error-classification-fragile`** — the wrapper does NOT match on error text; it adds context *around* the existing error with no conditional logic. Zero version-fragility.
- **IMP-109 sequencing** — IMP-109 ("single-parse catalog scan") may touch `catalog/scan.rs` parse paths. SL-151's changes are orthogonal (they touch entity read paths, not catalog scan) — no conflict, but IMP-109 may want to adopt `parse_entity_toml` later.
- **Caller surface width.** `read_meta`/`read_id` callers span ~13 modules — each needs a `prefix` argument added. The change per caller is mechanical (one `&str` argument), but the plan should account for the breadth.
