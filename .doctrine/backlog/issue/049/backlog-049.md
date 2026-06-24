# ISS-049: Entity TOMLs with non-contiguous sections cause opaque parse failures

## Root cause

`toml 0.8.23` and `toml_edit 0.22.27` both reject duplicate table headers per
the TOML spec. If an entity TOML has `[relationships]` appearing after
`[[relation]]` rows (or any repeated `[section]`), both parsers fail with
`"duplicate key relationships"` — a raw parse error with no entity context or
remediation hint.

## Blast radius

### Entity kinds & their sections (all 25 kinds affected)

The canonical layout is: root keys → `[relationships]` → `[[relation]]` (→
optional facets). A manual edit that writes `[relationships]` after
`[[relation]]` is the most likely non-contiguous pattern.

| Pattern | Kinds | Sections |
|---|---|---|
| Root + typed axes + array-of-tables | SL, ADR, POL, STD, RFC, backlog(×5), knowledge(×4) | `[relationships]` + `[[relation]]` |
| Root + review + findings | RV | `[review]` + `[target]` + `[[finding]]` |
| Root + change rows | REV | `[relationships]` + `[[change]]` |
| Root + facet + relationships | backlog-risk, knowledge(×4) | `[facet]` + `[relationships]` |
| Root + multi-section | Memory | `[scope]` + `[git]` + `[review]` + `[trust]` + `[ranking]` + `[[relation]]` |
| Optional facets | All kinds | `[estimate]` / `[value]` / `[facet]` (sparse) |

### Read paths (~20 call sites)

| File | Function | Parser | Surface |
|---|---|---|---|
| `meta.rs` | `read_meta`, `read_id` | `toml::from_str` | show/list/validate for ALL kinds |
| `catalog/scan.rs` | `status_and_title_for`, `title_for`, `read_facets` | `toml::from_str`, `toml::Table` | Corpus scan (priority graph, relation graph, inspect) |
| `slice.rs` | `read_slice` | `toml::from_str` | Slice show/edit |
| `backlog.rs` | `read_item` | `toml::from_str` | Backlog show/edit |
| `knowledge.rs` | `read_record` | `toml::from_str` | Knowledge show/edit |
| `governance.rs` | `read_doc` | `toml::from_str` | ADR/POL/STD/RFC show/edit |
| `review.rs` | review reads | `toml::from_str` | Review show |
| `rec.rs` | rec reads | `toml::from_str` | REC show |
| `revision.rs` | revision reads | `toml::from_str` | REV show |
| `concept_map.rs` | concept map reads | `toml::from_str` + `toml_edit` | CM show/edit |
| `memory.rs` | `read_catalog_record` | `toml::from_str` | Memory scan |
| `dtoml.rs` | `parse` | `toml::from_str` | `doctrine.toml` config read |
| `lazyspec.rs` | spec parse | `toml::from_str` | Spec load |
| `dispatch.rs` | dispatch config | `toml::from_str` | Dispatch config |

### Write/edit paths (5 call sites that parse→edit→write)

| File | Operation | Uses |
|---|---|---|
| `backlog.rs` / `tag.rs` | `apply_tags_set` | `toml_edit::DocumentMut` |
| `slice.rs` | `selector_add` / `selector_rm` | `toml_edit::DocumentMut` |
| `concept_map.rs` | `set_dsl` | `toml_edit::DocumentMut` |
| `memory.rs` | key/lifespan edits | `toml_edit::DocumentMut` |

None create non-contiguous sections — they parse→edit→write existing values —
but they **encounter** pre-existing damage from manual edits and fail.

## Current error quality

The `catalog/scan.rs` path already wraps errors into `CatalogDiagnostic`
(severity `Error`, names entity + file). But the `meta::read_meta` path and
individual `read_slice`/`read_item`/`read_doc` paths propagate the raw
`toml::from_str` error directly — the user sees something like:

```
Failed to parse .doctrine/slice/007/slice-007.toml: TOML parse error at line 15,
column 1: duplicate key `relationships` in document root
```

This names the file but NOT the entity clearly, and offers no remediation hint.
The `validate` command currently only checks id integrity + relation edges, NOT
TOML well-formedness.

## Desired behaviour

The tooling should either:
- **A: Normalise on read** — detect non-contiguous sections and merge them
  before parsing (lossy, risks silent data loss).
- **B: Reject with a clear diagnostic** — name the entity (canonical id), the
  duplicate section, both line numbers, and a remediation hint ("merge the two
  `[relationships]` sections into one contiguous block").

B is preferred as the first step (doctrine's philosophy: no silent data loss).
A may be considered as an advanced recovery option later.

## Recommendation

Two fix surfaces:

1. **Pre-parse structural check** — add a function (e.g.
   `dtoml::validate_sections`) that scans raw TOML text for non-contiguous
   `[section]`/`[[array]]` headers before parsing, producing a clear diagnostic:
   `"SL-007: section [relationships] appears non-contiguously (lines 5, 15) —
   merge the sections"`.

2. **Augment `doctrine validate`** — add this check to `integrity::scan_kind`
   so `validate` catches these before they break other tooling.

3. **Wrap parse errors in shared read paths** — ensure every `toml::from_str`
   call site that reads entity TOMLs includes entity context in the error (many
   already do via `.with_context(|| format!("Failed to parse {}",
   path.display()))` but could name the canonical id).

## Related

- IMP-080: "no standalone plan.toml validation — malformed plan surfaces late at
  phase parse time" — same symptom class (late surface), different target.
- IMP-109: "catalog scan: read each entity TOML once" — consolidating the double
  parse in `read_facets` is an opportunity to validate once and share the result.
- CHR-019: spike confirmed `toml_edit` root inserts are safe; the insert-at-root
  placement guarantee is a separate concern from non-contiguous section handling.
