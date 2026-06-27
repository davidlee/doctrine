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

**Dispatch topology (verified).** Column sites are NOT one-per-kind. `adr`,
`policy`, `standard`, and `rfc` all route `list` through `governance::run_list`
over the shared `GOV_COLUMNS`/`GovRow` — **one** site serves four kinds.
`adr.rs`/`policy.rs`/`standard.rs`/`rfc.rs` carry no column model and need no
tags edit. The distinct column sites in scope:

| Column site | File | Constant | Kinds served | Row type | Row carries `tags`? |
|---|---|---|---|---|---|
| governance | `src/governance.rs` | `GOV_COLUMNS` | adr, policy, standard, rfc | `GovRow` | ✅ yes |
| slice | `src/slice.rs` | `SLICE_COLUMNS` | slice | `SliceRowTuple` | ❌ add field |
| spec | `src/spec.rs` | `SPEC_COLUMNS` | spec | `SpecListRow` | ❌ add field |
| knowledge | `src/knowledge.rs` | `KN_COLUMNS` | knowledge | `KnowledgeRecord` | ✅ yes |
| revision | `src/revision.rs` | `REV_COLUMNS` | revision | `ListRow` | ✅ yes (column already present) |
| rec | `src/rec.rs` | `REC_COLUMNS` | rec | `ListRow` | ❌ add field |
| review | `src/review.rs` | `REVIEW_COLUMNS` | review | `ListRow` | ❌ add field |

Where the row type lacks `tags: Vec<String>` (slice, spec, rec, review) the
field is added first, populated from the unified root-level `tags` (SL-136). The
extractor (`Meta.tags` / equivalent) is generic — the work is the struct field +
its construction site, not new storage. `revision` already renders `tags` (D2 no
column change; D3 only).

Each new column gets a `listing::Column` entry using `paint_tag` (following
`backlog.rs` line 1086–1095):

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

### D3: Tags in default columns — conditional (show iff any row tagged), via one shared helper

Tags column is **conditionally visible** by default — spliced into the effective
default set only when at least one displayed row carries non-empty tags.
`--columns` with an explicit list bypasses this entirely (`select_columns`
ignores `default` when an explicit list is given — user's order wins, tags shown
iff requested).

**No parallel implementation (CLAUDE.md DRY).** `backlog.rs:1237–1250` already
encodes this splice inline. Replicating that ~6-line `flat_map` into 8 more kinds
is forbidden duplication. Instead, lift the rule into **one** helper in
`listing.rs` and route every site — including the existing backlog one — through
it:

```rust
/// Splice `tags` immediately before `title` in a default column set, IFF
/// `any_tagged`. Returns an owned set (never mutates the caller's const).
/// `--columns` callers bypass this: `select_columns` ignores `default` when an
/// explicit list is supplied. Kinds whose default lacks `title` are unaffected
/// (every `list` kind currently carries `title`).
pub(crate) fn default_with_tags<'a>(base: &[&'a str], any_tagged: bool) -> Vec<&'a str> {
    if !any_tagged {
        return base.to_vec();
    }
    base.iter()
        .flat_map(|&c| if c == "title" { vec!["tags", "title"] } else { vec![c] })
        .collect()
}
```

Each kind's list dispatch then collapses to two lines:
```rust
let any_tagged = rows.iter().any(|r| !r.tags.is_empty());
let effective_default = listing::default_with_tags(DEFAULT, any_tagged);
let sel = listing::select_columns(&KIND_COLUMNS, &effective_default, columns.as_deref())?;
```

**Refactor-first:** rewrite `backlog.rs` onto `default_with_tags` in the same
change (its inline block is the prototype, not a second implementation). Its
existing goldens are the behaviour-preservation proof — they must stay green
unchanged through the refactor.

Affects the 7 column sites of D2: `governance::run_list` (covers adr/policy/
standard/rfc in one edit), `slice`, `spec`, `knowledge`, `revision`, `rec`,
`review` — plus the `backlog` refactor. `memory` and `concept-map` are always-on
(no conditional) — untouched by D3.

### D4: REC and review tag surfaces + taggable set

**Write gate:** Add `"REC"` and `"RV"` to `src/tag.rs:TAGGABLE`. The write
path is generic (root-level `tags` array via `tag::apply_tags_set`) — only the
prefix gate prevents writes now.

**Read wiring for REC:** REC list uses a column model — add `tags` column
entry and conditional default following D2/D3. `rec show` and `--json` must
render tags.

**Read wiring for review:** Review list is special (derived status, findings
count, await state) but still dispatches through `select_columns(&REVIEW_COLUMNS,
REVIEW_DEFAULT, …)` (`review.rs:1698`) — the conditional default threads through
`default_with_tags` identically to every other site; the derived columns are
orthogonal. `review show` already renders a structured findings list — add `tags`
to the metadata header. For `--json`: `show_json` emits `{ "kind":"review",
"review": ShowJson{ …flatten ReviewDoc, status, awaiting }, "body" }`
(`review.rs:1612`). `tags` rides the flattened `ReviewDoc`, so the field appears
inside the `review` object once `ReviewDoc` carries+serialises root-level `tags`:

```json
{ "kind": "review",
  "review": { "id": 183, "tags": ["governance"], "status": "open", "awaiting": "author", … },
  "body": "…" }
```

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
| `src/listing.rs` | Add `default_with_tags` helper (D3 — single source of the splice rule) |
| `src/backlog.rs` | Refactor inline splice (1237–1250) onto `default_with_tags`; goldens stay green |
| `src/commands/relation.rs` | Add `--columns` to `List` and `Census`; thread through pub `render_list`/`render_census` |
| `src/relation_query.rs` | Thread `Option<&str>` columns through pub `render_list`/`render_census` into the private table helpers |
| `src/governance.rs` | Add `tags` to `GOV_COLUMNS` + conditional default in `governance::run_list` — covers adr/policy/standard/rfc in one edit (rfc.rs/adr.rs/policy.rs/standard.rs untouched) |
| `src/slice.rs` | Add `tags` field to `SliceRowTuple` + `tags` column + conditional default |
| `src/spec.rs` | Add `tags` field to `SpecListRow` + `tags` column + conditional default |
| `src/knowledge.rs` | Add `tags` column + conditional default (`KnowledgeRecord.tags` already present) |
| `src/revision.rs` | Add conditional default for `tags` (column already exists) |
| `src/rec.rs` | Add `tags` field to `ListRow` + `tags` column + conditional default + show/JSON wire |
| `src/review.rs` | Add `tags` field to `ListRow` + `tags` column + conditional default + show/JSON wire |
| `src/tag.rs` | Add `"REC"`, `"RV"` to `TAGGABLE` |
| `src/concept_map.rs` | Lowercase `header` fields in `CONCEPT_MAP_COLUMNS` |
| `tests/e2e_list_columns_golden.rs` | Add `RelationRow`/`CensusRow` coverage; REC/review tag coverage |
| `tests/e2e_list_conformance.rs` | Add relation/census to the parse-conformance matrix |

---

## Verification

- **Behaviour-preservation, stated precisely.** Goldens for unmodified kinds
  (`memory`, `concept-map` data rows, requirements) stay byte-identical. For the
  7 modified column sites the conditional gate is the safety argument: an
  *untagged* fixture corpus produces byte-identical output (tags column never
  splices), so only goldens whose fixtures carry tagged rows regenerate — and
  those regenerate intentionally to show the new column. The backlog refactor
  (D3) must leave its goldens green unchanged (proof the helper is behaviour-
  equal to the inline splice).
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
