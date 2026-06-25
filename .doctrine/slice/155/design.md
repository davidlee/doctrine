# Design — SL-155

## Decisions

**D1 — Hide terminal states by default.** Revisions have four statuses (`proposed | started | done | abandoned`) and an orthogonal approval axis (`none | requested | approved | rejected`). The list's primary value is "do I have any outstanding revisions?", so terminal states (`done`, `abandoned`) are hidden by default, revealed via `--all`. The hide-set is `&["done", "abandoned"]`. No approval-based hiding (approvals are orthogonal — a `done` revision with `approved` is still done).

**D2 — Tags opt-in via `--columns`, not default.** The scope follows governance precedent: `GOV_COLUMNS` does not default-show tags, and the revision list surface mirrors that pattern. Tags are available via `--columns tags` and filterable via `--tag <value>`, but they are not in the default visible set (`id, status, approval, title`). The `--json` output always includes tags inline.

**D3 — REC shape, not governance shape.** The revision list rides the REC pattern (read all TOMLs → project to typed row → column model), not the governance `meta::read_metas` path. Revisions are few (10 in corpus), so the REC `read_all` approach is fine. Unlike REC, revisions HAVE a status axis, so the `REV_STATUSES` const + `validate_statuses` guard + hide-set are wired in.

**D4 — No `created`/`updated` columns.** The revision template stamps `updated` but `RevDoc` doesn't parse it (it's informational). The list won't surface dates unless a follow-up adds them. Governance lists don't show dates in default columns either.

**D5 — No `slug` column.** The scope's default columns mirror `GOV_COLUMNS` (`id, status, approval, title`). Slugs are not a governance list column; adding one here would create an inconsistency without justification. The slug is accessible via `--json` output and `revision show`, not via the list.

---

## Code impact

### Scope attribution note

The scope's "Affected surface" lists `src/spec.rs — C2 (two template comment fixes)`. This is a misattribution: C2 items are template files (`install/templates/spec-tech.toml`, `install/templates/spec-product.toml`, `install/templates/interactions.toml`). `src/spec.rs` is only C3 (doc comment fix). The code impact table below shows the correct mapping. The scope should be corrected.

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

`#[serde(default)]` ensures existing tagless revision TOMLs deserialise without error — a round-trip test verifies this.

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
const REV_COLUMNS: [Column<RevDoc>; 5] = [
    Column { name: "id",       header: "id",       cell: |d| canonical_id(d.id),            paint: Fixed(Cyan) },
    Column { name: "status",   header: "status",   cell: |d| d.status.as_str().to_string(), paint: ByValue(status_hue) },
    Column { name: "approval", header: "approval", cell: |d| d.approval.as_str().to_string(), paint: None },
    Column { name: "tags",     header: "tags",     cell: |d| d.tags.join(", "),             paint: None },
    Column { name: "title",    header: "title",    cell: |d| d.title.clone(),               paint: Alternate(…) },
];
const REV_DEFAULT: &[&str] = &["id", "status", "approval", "title"];
```

Tags are a selectable column but not in the default visible set (D2, following governance precedent).

#### CLI integration

The `RevisionCommand` enum currently holds: `New`, `Show`, `Status`, `Change`, `Approve`, `Apply`, `Paths`. `List` is added alongside them — structurally a peer variant:

```rust
// src/revision.rs — RevisionCommand enum (existing variants + one addition)
enum RevisionCommand {
    New { … },
    Show { … },
    Status { … },
    Change { … },
    Approve { … },
    Apply { … },
    Paths,
    // SL-155: new list verb
    List {
        #[command(flatten)]
        list: CommonListArgs,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}
```

Dispatch in `run_revision` (existing `match` arm sequence):

```rust
fn run_revision(cmd: RevisionCommand, …) -> anyhow::Result<()> {
    match cmd {
        RevisionCommand::New { … } => run_new(…),
        RevisionCommand::Show { … } => run_show(…),
        // … existing arms …
        RevisionCommand::List { list, path } => run_list(path, list.into_list_args(color)),
    }
}
```

#### Functions

| Function | Responsibility |
|---|---|
| `read_revs(dir: &Path) -> Vec<RevDoc>` | Parse all `revision-NNN.toml` files (mirrors `read_recs`) |
| `key(d: &RevDoc) -> FilterFields` | Project to filterable fields (canonical id, slug, title, status, tags) |
| `list_rows(root: &Path, args: ListArgs) -> String` | Validate statuses, retain, sort, render |
| `run_list(path, args)` | Resolve root, call `list_rows`, write to stdout |

#### Template

`install/templates/revision.toml`: add `tags = []` after `updated = "{{date}}"`.

#### Show-surface tag deferral (IMP-144)

Tags appear in `--json` output for `revision show` (EX-06). Human-readable (prose) tag rendering in `revision show` is deferred to IMP-170 G2 — the scope's non-goal section already excludes G1-G7 show gaps from this slice. The JSON surface is the read-path for this slice; the prose surface follows in IMP-170.

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
| `list_rows_columns_selects_and_reveals_tags` | `--columns id,status,tags` reveals tags column (not default) |
| `list_rows_unknown_column_is_the_uniform_error` | `--columns bogus` errors with available set (SL-037 uniform contract) |
| `render_revision_toml_includes_tags` | Template renders `tags = []` |
| `tagless_revision_round_trips` | A revision TOML without a `tags` field survives read→render→parse without corruption |

All tests are `#[cfg(test)]` unit tests in `src/revision.rs`, following the existing revision test pattern (no CLI golden tests — the list verb's integration surface is small enough that unit tests covering all paths suffice; CLI golden pattern is reserved for higher-risk verb surfaces).

---

## Verification

| ID | Criteria | Mode |
|---|---|---|
| EN-01 | `just gate` zero warnings | VT |
| EN-02 | All existing revision tests stay green unchanged | VT |
| EN-03 | New revision list tests pass (table + JSON + filtering + hide-set + round-trip) | VT |
| EX-01 | `doctrine revision list` shows only non-terminal revisions (none in current corpus → empty table header) | VT |
| EX-02 | `doctrine revision list --all` shows all 10 revisions | VT |
| EX-03 | `doctrine revision list --tag <t>` filters correctly after tagging a revision | VT |
| EX-04 | `doctrine revision list --status bogus` errors with known-set list | VT |
| EX-05 | `doctrine supersede ADR-012 ADR-004` succeeds, authoring the edge | VA |
| EX-06 | `doctrine revision show REV-001 --json` includes `tags` in output | VT |
| EX-07 | `doctrine revision list --columns bogus` errors with available column set | VT |
| EX-08 | `doctrine tag set REV-001 test-tag` succeeds (REV now taggable) | VT |
