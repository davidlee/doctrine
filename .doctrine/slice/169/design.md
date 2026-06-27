# SL-169: Design — columns & tags read-surface wiring

## Design decisions

### D1: `--columns` on relation list/census

**Rationale:** The render pipeline already defines `RELATION_COLUMNS` (4 cols:
`source`, `label`, `target`, `state`) and `CENSUS_COLUMNS` (5 cols: `label`,
`count`, `resolved`, `unresolved`, `free_text`) and already calls
`listing::select_columns(…, default, None)`. Only the `--columns` CLI arg and
the thread through `run_relation_list`/`run_relation_census` to the
`render_list_table`/`render_census_table` helpers are missing.

**Mechanism:** Add `--columns` to `RelationCommand::List` and
`RelationCommand::Census`. Thread `columns: Option<String>` through to
`render_list_table`/`render_census_table`, replacing the hardcoded `None` with
`columns.as_deref()`.

**Test impact:** Extend `tests/e2e_list_columns_golden.rs` to cover
`RelationRow` and `CensusRow` in the cross-verb column model net.

### D2: `tags` column in per-kind `COLUMNS` arrays

For every taggable kind whose `COLUMNS` array lacks a `tags` entry:

| Kind | File | Constant |
|---|---|---|
| slice | `src/slice.rs` | `SLICE_COLUMNS` |
| adr/policy/standard | `src/governance.rs` | `GOV_COLUMNS` |
| spec | `src/spec.rs` | spec columns |
| rfc | `src/rfc.rs` | RFC columns |
| knowledge | `src/knowledge.rs` | knowledge columns |

Each gets a `listing::Column` entry using `paint_tag` (following `backlog.rs`
line 1086–1095):

```rust
listing::Column {
    name: "tags",
    header: "tags",
    cell: |row| row.tags.join(", "),
    paint: listing::ColumnPaint::PerToken {
        split: |row| row.tags.clone(),
        render: listing::paint_tag,
    },
},
```

The per-kind row structs must carry `tags: Vec<String>` — verify each already
does (most read from `Meta.tags` which SL-136 unified to root-level storage).

### D3: Tags in default columns — conditional (show iff any row tagged)

Following `backlog.rs` lines 1207–1242: tags column is **conditionally visible**
by default — spliced into the effective default set only when at least one
displayed row carries non-empty tags. `--columns` with an explicit column list
bypasses the conditional logic entirely (user's order wins, tags shown iff
requested).

For each kind's list dispatch, before `select_columns`:
```rust
let any_tagged = rows.iter().any(|r| !r.tags.is_empty());
let effective_default: Vec<&str> = if any_tagged {
    // insert "tags" before "title"
    DEFAULT.iter().flat_map(|c| if *c == "title" { vec!["tags", "title"] } else { vec![*c] }).collect()
} else {
    DEFAULT.to_vec()
};
```

Affects: `slice`, `governance` (adr/policy/standard), `spec`, `rfc`,
`knowledge`, `revision`. `backlog`, `memory`, and `concept-map` already have
tags in their defaults (memory/concept-map are always-on; backlog is
conditional — no change needed).

### D4: REC and review tag surfaces + taggable set

**Write gate:** Add `"REC"` and `"RV"` to `src/tag.rs:TAGGABLE`. The write
path is generic (root-level `tags` array via `tag::apply_tags_set`) — only the
prefix gate prevents writes now.

**Read wiring for REC:** REC list uses a column model — add `tags` column
entry and conditional default following D2/D3. `rec show` and `--json` must
render tags.

**Read wiring for review:** Review list is special (derived status, findings
count, await state). Add `tags` column with conditional default. `review show`
already renders a structured findings list — add `tags` to the metadata
header. `review show --json` must include `tags` in the `review` object.

**Test impact:** Extend e2e goldens to cover REC/review tag read surfaces.

### D5: concept-map column header casing

Normalise `CONCEPT_MAP_COLUMNS` header fields from Title Case to lowercase:

```rust
// Before:
header: "ID",     header: "Status",  header: "Tags",
header: "Slug",   header: "Title",

// After:
header: "id",     header: "status",  header: "tags",
header: "slug",   header: "title",
```

The `name` fields are already lowercase — only `header` changes.

---

## Code impact

| File | Change |
|---|---|
| `src/commands/relation.rs` | Add `--columns` to `List` and `Census`; thread through |
| `src/relation_query.rs` | Refactor `render_list_table`/`render_census_table` to accept `Option<&str>` columns |
| `src/slice.rs` | Add `tags` column + conditional default |
| `src/governance.rs` | Add `tags` column + conditional default; verify `FilterFields.tags` wired |
| `src/spec.rs` | Add `tags` column + conditional default |
| `src/rfc.rs` | Add `tags` column + conditional default |
| `src/knowledge.rs` | Add `tags` column + conditional default |
| `src/revision.rs` | Add conditional default for `tags` (column already exists) |
| `src/rec.rs` | Add `tags` column + conditional default + show/JSON wire |
| `src/review.rs` | Add `tags` column + conditional default + show/JSON wire |
| `src/tag.rs` | Add `"REC"`, `"RV"` to `TAGGABLE` |
| `src/concept_map.rs` | Lowercase `header` fields in `CONCEPT_MAP_COLUMNS` |
| `tests/e2e_list_columns_golden.rs` | Add `RelationRow`/`CensusRow` coverage; REC/review tag coverage |
| `tests/e2e_list_conformance.rs` | Add relation/census to the parse-conformance matrix |

---

## Verification

- Every existing list golden must stay green unchanged (behaviour-preservation
  gate for kinds not being modified).
- New goldens: `relation list --columns`, `relation census --columns`, each
  kind's `list` with tags default (tagged + untagged cases), each kind's
  `show`/`--json` with tags.
- `doctrine tag set REC-001 test-tag` must succeed (was: "REC is not taggable
  yet").
- `doctrine tag set RV-001 test-tag` must succeed (was: "RV is not taggable
  yet").
- `relation list --columns source,label,state` must select/order columns.
- `relation census --columns label,count` must select/order columns.
- `concept-map list` headers must render as lowercase.
- `just gate` green throughout.
