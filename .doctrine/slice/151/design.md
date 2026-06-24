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

Six read paths get the wrapper: `meta::read_meta`, `meta::read_id`,
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
| `src/meta.rs` | `read_meta`: swap raw `toml::from_str` for `dtoml::parse_entity_toml`. `read_id`: same. Add `prefix` parameter to both. **`read_metas` (the list funnel) also gains a `prefix` param** — it loops `read_meta` internally but holds only `stem`, so the prefix must be threaded through it too. |
| `src/integrity.rs` | `scan_kind`: add `diagnostics: &mut Vec<String>` param; after `read_id`, do `toml::from_str::<toml::Value>(&text)`, push errors. `id_integrity_findings`: allocate `diagnostics`, pass to `scan_kind`, append to findings. Also a forced `read_meta`/`read_id`/`read_metas` caller — see survey. |
| `src/slice.rs` | `read_slice`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, "SL", id)` |
| `src/backlog.rs` | `read_item`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, kind.prefix(), id)` |
| `src/knowledge.rs` | `read_record`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, kind.prefix(), id)` |
| `src/governance.rs` | `read_doc`: swap raw `toml::from_str` for `dtoml::parse_entity_toml(&text, &g.kind.prefix, id)`. `list_rows` (`read_metas`) supplies `g.kind.prefix` for **all** governance kinds at one site. |
| `src/lazyspec.rs` | **Forced caller** — `load_entity_record` (`:468`) calls `read_meta`; supply prefix from `engine_kind`. |
| `src/catalog/scan.rs` | **Forced caller** — status/title path (`:429`) calls `read_meta`; supply `kref.kind.prefix`. Hot path, but `with_context` is lazy (allocates only on error) → no runtime cost. |
| Direct `read_meta`/`read_metas`/`read_id` callers | Add `prefix` argument — each caller (or its kind-generic funnel) knows its prefix; see Caller survey. |

## Caller survey for `read_meta` / `read_id` / `read_metas`

Surveyed by grep against `src/` (production call sites only). The per-kind
`show`/`list` paths do **not** each call `read_meta` directly — most funnel
through `meta::read_metas(tree_root, stem)` (the list loop) or the
kind-generic governance funnel. The prefix is threaded at the **real** sites
below, not per-module.

**Direct `read_meta` callers (prod):**

| Caller | Prefix source |
|---|---|
| `src/slice.rs:555/584/639` (show paths) | `"SL"` |
| `src/integrity.rs:414` (reseat slug read) | `kind.kind.prefix` |
| `src/lazyspec.rs:468` (`load_entity_record`) | from `engine_kind` |
| `src/catalog/scan.rs:429` (status/title path) | `kref.kind.prefix` |
| `src/meta.rs:84` (`read_metas` internal loop) | threaded `prefix` param |

**`read_metas` callers (the list funnel — each supplies `prefix`):**

| Caller | Prefix source |
|---|---|
| `src/governance.rs:71` (`list_rows`, **all** gov kinds: ADR/POL/STD/PRD/SPEC/RFC/REV/REQ) | `g.kind.prefix` |
| `src/status.rs:292` (slice) / `:314` (rfc) | `"SL"` / `"RFC"` |
| `src/concept_map.rs:1241` | `"CM"` |
| `src/slice.rs:1265` (list) | `"SL"` |
| `src/spec.rs:1442` | `SPEC`/`PRD` per surface |

> The dropped survey rows (`adr.rs`, `policy.rs`, `standard.rs`, `rfc.rs`,
> `revision.rs`, `requirement.rs`, `review.rs`, `rec.rs` as *direct*
> `read_meta` callers) were fiction — those kinds reach `read_meta` only via
> the governance `read_metas` funnel above, at one site (`g.kind.prefix`).
> Backlog/knowledge `show` likewise route through `read_metas`, not bespoke
> per-kind `read_meta` calls.

**`read_id` callers (prod):**

| Caller | Prefix source | Note |
|---|---|---|
| `src/integrity.rs:263` (`scan_kind`) | `kind.kind.prefix` | context surfaces on `validate` |
| `src/integrity.rs:304` (alias/reseat target resolve) | `kind.kind.prefix` | error is `.ok()`-swallowed → prefix context is moot here |

## Not changed (out of scope)

| File | Reason |
|---|---|
| `catalog/scan.rs` error wrapping | Already wraps parse errors into `CatalogDiagnostic` with entity key — **but** its `read_meta` call (`:429`) is a forced in-scope caller (see survey); only the diagnostic-wrapping is untouched. |
| `review.rs`, `rec.rs`, `revision.rs` *dedicated* read paths | Lower-traffic bespoke reads; can adopt wrapper later (their `show`/`list` already flow through `read_metas`, which IS in scope). |
| `memory.rs`, `dispatch.rs`, `dtoml.rs` (config) | Config/specialised reads, not entity TOMLs. |
| Write paths | Use `toml_edit::DocumentMut` — always contiguous output. |

> `lazyspec.rs` and `catalog/scan.rs` were previously listed here as
> out-of-scope; they are **forced `read_meta` callers** and moved into the
> affected-surface table above (a `prefix` param is compile-forcing).

## Verification

| ID | Kind | What it proves |
|---|---|---|
| VT-1 | Unit | `parse_entity_toml` passes through valid TOML identically to raw `toml::from_str` |
| VT-2 | Unit | `parse_entity_toml` on non-contiguous TOML errors with `"{prefix}-{id:03}: TOML parse failed"` prefix |
| VT-3 | Unit | `scan_kind` flags a non-contiguous `[relationships]` in a fixture |
| VT-4 | Unit | `scan_kind` produces no diagnostics on valid TOML |
| VT-5 | Integration | `doctrine validate` exits non-zero on a non-contiguous entity TOML fixture |
| VT-6 | Integration | `doctrine show SL-NNN` on non-contiguous TOML errors with canonical id in message |
| VT-7 | Unit | `read_id` on a malformed id-only TOML surfaces the `"{prefix}-{id:03}: TOML parse failed"` context (wrapper path distinct from `read_meta`) |
| VA-1 | Agent | `just gate` — clippy zero warnings across workspace |
| VH-1 | Human | Manually edit an entity TOML to be non-contiguous, run `doctrine validate`, confirm clear diagnostic with canonical id |

## Open questions

None — all three design decisions (D1–D3) are locked by user approval.

## Risks

- **`mem.pattern.parse.toml-error-classification-fragile`** — the wrapper does NOT match on error text; it adds context *around* the existing error with no conditional logic. Zero version-fragility.
- **IMP-109 sequencing** — IMP-109 ("single-parse catalog scan") may touch `catalog/scan.rs` parse paths. SL-151's changes are orthogonal (they touch entity read paths, not catalog scan) — no conflict, but IMP-109 may want to adopt `parse_entity_toml` later.
- **Caller surface width.** `read_meta`/`read_id` callers span ~13 modules — each needs a `prefix` argument added. The change per caller is mechanical (one `&str` argument), but the plan should account for the breadth.
