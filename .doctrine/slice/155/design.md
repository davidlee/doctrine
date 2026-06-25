# Design — SL-155

## Decisions

**D1 — Hide terminal states by default.** Revisions have four statuses (`proposed | started | done | abandoned`) and an orthogonal approval axis (`none | requested | approved | rejected`). The list's primary value is "do I have any outstanding revisions?", so terminal states (`done`, `abandoned`) are hidden by default, revealed via `--all`. The hide-set is `&["done", "abandoned"]`. No approval-based hiding (approvals are orthogonal — a `done` revision with `approved` is still done).

**D2 — Tags visible by default.** Tags are a default column (`id, status, approval, tags, title`). The `--tag <value>` filter works via `key()`. The `show`/JSON surface includes tags inline. Governance lists not showing tags is a lag, not a precedent — new surfaces should surface them.

**D3 — REC shape, not governance shape.** The revision list rides the REC pattern (read all TOMLs → project to typed row → column model), not the governance `meta::read_metas` path. Revisions are few (10 in corpus), so the REC `read_all` approach is fine. Unlike REC, revisions HAVE a status axis, so the `REV_STATUSES` const + `validate_statuses` guard + hide-set are wired in.

**D4 — No `created`/`updated` columns.** The revision template stamps `updated` but `RevDoc` doesn't parse it (it's informational). The list won't surface dates unless a follow-up adds them. Governance lists don't show dates in default columns either.

---

## Code impact

### Cluster A — one-line fixes (`src/relation.rs`, `src/spec.rs`, `src/tag.rs`, 4 templates)

| ID | File | Change |
|---|---|---|
| C1 | `src/relation.rs` ~L408-415 | Parent row: `sources: &[SPEC]` → `&[SPEC, PRD]`, `target: Kinds(&[SPEC])` → `Kinds(&[SPEC, PRD])` |
| C2a | `install/templates/spec-tech.toml` L19 | Comment: `tech-only` → `subtype-aware (SPEC or PRD)` |
| C2b | `install/templates/spec-product.toml` | Add `# parent = "PRD-NNN"` example after `tags = []` |
| C2c | `install/templates/interactions.toml` L3 | `hand-authored in v1 (no verb)` → `authored via \`doctrine spec interactions add\`` |
| C3 | `src/spec.rs` L746 | Doc comment: remove `Tech-only, ` from `pub(crate) parent` |
| G5a | `install/templates/adr.toml` L7 | Fix supersede comment: `doctrine link … supersedes` → `doctrine supersede` |
| G5b | — | Run `doctrine supersede ADR-012 ADR-004` |
| I1 | `src/tag.rs` L16 | Add `"REV"` to `TAGGABLE` set (IMP-144 — read surface wired → write surface enabled) |

### Revision list (`src/revision.rs` ~200 new lines)

#### Struct changes

```rust
// RevDoc gains tags (IMP-144)
pub(crate) struct RevDoc {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: RevStatus,
    pub(crate) approval: Approval,
    #[serde(default)]
    pub(crate) tags: Vec<String>,   // ← NEW
}
```

#### New types

```rust
/// Known revision statuses (kebab-case, locked to RevStatus variants).
const REV_STATUSES: &[&str] = &["proposed", "started", "done", "abandoned"];

/// Statuses hidden from the default list view.
const REV_HIDDEN: &[&str] = &["done", "abandoned"];

/// JSON list row (tags stay Vec<String>).
#[derive(Serialize)]
struct ListRow {
    id: String,
    status: String,
    approval: String,
    tags: Vec<String>,
    title: String,
}
```

RevDoc serves as the column row type directly (REC pattern — no intermediate materialised type).
Column extractors stringify enums inline: `|d| d.status.as_str().to_string()`, `|d| d.tags.join(", ")`.

#### Column definitions

```rust
const REV_COLUMNS: [Column<RevDoc>; 6] = [
    Column { name: "id",       header: "id",       cell: |d| canonical_id(d.id),            paint: Fixed(Cyan) },
    Column { name: "status",   header: "status",   cell: |d| d.status.as_str().to_string(), paint: ByValue(status_hue) },
    Column { name: "approval", header: "approval", cell: |d| d.approval.as_str().to_string(), paint: None },
    Column { name: "slug",     header: "slug",     cell: |d| d.slug.clone(),                paint: None },
    Column { name: "tags",     header: "tags",     cell: |d| d.tags.join(", "),             paint: None },
    Column { name: "title",    header: "title",    cell: |d| d.title.clone(),               paint: Alternate(…) },
];
const REV_DEFAULT: &[&str] = &["id", "status", "approval", "tags", "title"];
```

#### Functions

| Function | Responsibility |
|---|---|
| `read_revs(dir: &Path) -> Vec<RevDoc>` | Parse all `revision-NNN.toml` files (mirrors `read_recs`) |
| `key(d: &RevDoc) -> FilterFields` | Project to filterable fields (canonical id, slug, title, status, tags) |
| `list_rows(root: &Path, args: ListArgs) -> String` | Validate statuses, retain, sort, render |
| `run_list(path, args)` | Resolve root, call `list_rows`, write to stdout |

#### CLI surface

```rust
// RevisionCommand gains:
List {
    #[command(flatten)]
    list: CommonListArgs,
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
},
```

Dispatch: `RevisionCommand::List { list, path } => run_list(path, list.into_list_args(color)),`

#### Template

`install/templates/revision.toml`: add `tags = []` after `updated = "{{date}}"`.

#### Tests

| Test | What it proves |
|---|---|
| `rev_statuses_matches_the_variants` | Drift canary: `REV_STATUSES` stays in lockstep with `RevStatus` enum |
| `list_rows_empty_tree_is_empty` | No revisions → empty table string (no crash) |
| `list_rows_hides_done_and_abandoned` | Default list excludes terminal revisions |
| `list_rows_all_reveals_hidden` | `--all` shows everything |
| `list_rows_filter_matches_slug_and_title` | Substring filter works |
| `list_rows_tag_filter_matches` | `--tag` filters by authored tags |
| `list_rows_unknown_status_errors` | `--status bogus` → uniform error |
| `list_rows_json_is_faithful_envelope` | JSON output has prefixed ids + tags array |
| `list_rows_columns_selects_and_reveals_tags` | `--columns id,status,tags` shows tags column |
| `list_rows_unknown_column_is_the_uniform_error` | `--columns bogus` errors with available set (SL-037 uniform contract) |
| `render_revision_toml_includes_tags` | Template renders `tags = []` |

---

## Verification

| ID | Criteria | Mode |
|---|---|---|
| EN-01 | `just gate` zero warnings | VT |
| EN-02 | All existing revision tests stay green unchanged | VT |
| EN-03 | New revision list tests pass (table + JSON + filtering + hide-set) | VT |
| EX-01 | `doctrine revision list` shows only non-terminal revisions (none in current corpus → empty table header) | VT |
| EX-02 | `doctrine revision list --all` shows all 10 revisions | VT |
| EX-03 | `doctrine revision list --tag <t>` filters correctly after tagging a revision | VT |
| EX-04 | `doctrine revision list --status bogus` errors with known-set list | VT |
| EX-05 | `doctrine supersede ADR-012 ADR-004` succeeds, authoring the edge | VA |
| EX-06 | `doctrine revision show REV-001 --json` includes `tags` in output | VT |
| EX-07 | `doctrine revision list --columns bogus` errors with available column set | VT |
| EX-08 | `doctrine tag set REV-001 test-tag` succeeds (REV now taggable) | VT |
