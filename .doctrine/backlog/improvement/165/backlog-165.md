# IMP-165: Add --orphans flag to memory list to find unlinked memories

## Problem

The dreaming skill's maintenance loop (step 3: **Link**) says:

> Check for orphans — memories with no inbound and no outbound relations
> (`memory show <REF>` renders empty relation list)

Today this requires running `doctrine memory show` on *every memory individually*
and inspecting the relation/backlink fields — an O(n) manual process. `memory
validate` catches dangling outbound links and stale verification but does not
detect orphanhood.

**Corpus snapshot** (2026-06-24, 232 memories):
- 66 true orphans (in=0, out=0) — 28% of the corpus
- 61 dead ends (in=0, out>0) — another 26%
- 26 orphans have no key, making them invisible to `memory find`

## Design

Add a `--orphans` flag to `doctrine memory list` that filters to memories with
zero inbound backlinks AND zero outbound relations/wikilinks. Reuses the
existing pure `backlinks_index` machinery in `src/links.rs`.

### Flag semantics

- `--orphans`: when set, post-filter the result set to memories where:
  - **inbound backlinks count** = 0 (no other memory links to this one, via
    authored `[[relation]]` or wikilink `[[mem.xxx]]`)
  - **outbound relation count** = 0 (no `[[relation]]` blocks in the TOML, no
    `[[mem.xxx]]` wikilinks in the body)
- Compatible with all existing filters: `--type`, `--filter`, `--tag`,
  `--status`, `--format`, `--json`, `--columns`.
- No new fields needed in the output — the existing table rows suffice; the
  flag is purely a filter.

### Reuse

`backlinks_index` in `src/links.rs` is already used by `backlink_rows_for` for
per-memory queries. The index is pure and fast: it builds `BTreeMap<String,
BTreeSet<String>>` mapping target uid/key → set of source uids.

Corpus-wide application:
1. Collect all memories (`collect_all` already called by `filtered_list`)
2. Build `known_uids` + `key_to_uid` maps
3. Build the wikilink and relation storage (same shape as `backlink_rows_for`
   does for one memory, but corpus-wide)
4. Compute `backlinks_index`
5. For each memory, count outbound (from `Memory.relations` + body wikilinks)
   and inbound (from the backlinks index, resolved through key→uid)
6. Retain only those with both counts = 0

### Edge cases

- Key-based backlinks resolve through key→uid: if memory A has key
  `mem.foo.bar` and memory B links to `mem.foo.bar`, that counts as an inbound
  for A.
- Wikilinks in the body targeting an unknown target still count as outbound
  (they're trying to link, even if unresolved).
- `--orphans` combined with `--status draft` would show only draft-orphans.
  Correct and useful.

## Implementation Sketch

### 1. Add flag to `MemoryCommand::List` (clap derive)

In `src/memory.rs`, ~line 233-244:

```rust
/// List recorded memories, newest first; AND-filter on the shared spine.
List {
    #[arg(long = "type", value_parser = MemoryType::parse)]
    memory_type: Option<MemoryType>,

    #[command(flatten)]
    list: crate::CommonListArgs,

    /// Show only orphaned memories: zero inbound backlinks and zero outbound relations.
    #[arg(long)]
    orphans: bool,                    // ← NEW

    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
},
```

### 2. Thread `orphans` through `run_list` → `list_rows`

`run_list` (~line 2774) currently does:

```rust
pub(crate) fn run_list(
    writer: &mut impl Write,
    path: Option<PathBuf>,
    type_f: Option<MemoryType>,
    args: ListArgs,
) -> Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    write!(writer, "{}", list_rows(&root, type_f, args)?)?;
    Ok(())
}
```

Change signature to accept `orphans: bool` and pass through to `list_rows`:

```rust
pub(crate) fn run_list(
    writer: &mut impl Write,
    path: Option<PathBuf>,
    type_f: Option<MemoryType>,
    args: ListArgs,
    orphans: bool,                    // ← NEW
) -> Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    write!(writer, "{}", list_rows(&root, type_f, args, orphans)?)?;
    Ok(())
}
```

### 3. Add orphan filter in `list_rows`

`list_rows` (~line 2730) currently:

```rust
pub(crate) fn list_rows(
    root: &Path,
    type_f: Option<MemoryType>,
    mut args: ListArgs,
) -> Result<String> {
    listing::validate_statuses(&args.status, MEMORY_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let rows = filtered_list(root, type_f, &filter)?;
    // ... render ...
}
```

Add an `orphans: bool` parameter. When true, call a new pure function
`is_orphan` on each row:

```rust
pub(crate) fn list_rows(
    root: &Path,
    type_f: Option<MemoryType>,
    mut args: ListArgs,
    orphans: bool,                    // ← NEW
) -> Result<String> {
    listing::validate_statuses(&args.status, MEMORY_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let mut rows = filtered_list(root, type_f, &filter)?;

    // NEW: post-filter to orphans
    if orphans {
        let all = collect_all(root)?;
        rows.retain(|m| is_orphan(m, &all, root));
    }

    match format { /* ... render unchanged ... */ }
}
```

### 4. New pure function `is_orphan`

```rust
/// Returns true if `memory` has zero inbound backlinks AND zero outbound
/// relations/wikilinks — a true orphan in the memory graph.
fn is_orphan(memory: &Memory, all: &[Memory], root: &Path) -> bool {
    // --- outbound: authored relations ---
    let has_relations = !memory.relations.is_empty();

    // --- outbound: body wikilinks ---
    let body = read_body(root, &memory.uid);
    let has_wikilinks = !extract_wikilinks(&body).is_empty();

    // --- inbound: backlinks index ---
    let (known_uids, key_to_uid) = known_link_maps(all);
    let backlinks_index = backlinks_index_for_all(all, root, &known_uids, &key_to_uid);

    let has_backlinks = backlinks_index
        .get(&memory.uid)
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        || memory.key.as_ref().map_or(false, |key| {
            backlinks_index
                .get(key)
                .map(|s| !s.is_empty())
                .unwrap_or(false)
        });

    !has_relations && !has_wikilinks && !has_backlinks
}
```

### 5. New helper: `backlinks_index_for_all` (corpus-wide)

Extract the body-scanning loop from `backlink_rows_for` into a reusable
corpus-wide function:

```rust
/// Build the full backlinks index for all memories in `all`. Returns a map from
/// target uid/key → set of source uids.
fn backlinks_index_for_all(
    all: &[Memory],
    root: &Path,
    known_uids: &BTreeSet<String>,
    key_to_uid: &BTreeMap<String, String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut wikilink_storage: BTreeMap<String, Vec<crate::links::Wikilink>> = BTreeMap::new();
    let mut relation_storage: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for memory in all {
        let body = read_body(root, &memory.uid);
        let resolved: Vec<crate::links::Wikilink> = extract_wikilinks(&body)
            .into_iter()
            .map(|link| {
                let target = resolve_wikilink(known_uids, key_to_uid, &link.target, link.is_uid)
                    .unwrap_or(link.target);
                crate::links::Wikilink { target, is_uid: true }
            })
            .collect();
        wikilink_storage.insert(memory.uid.clone(), resolved);

        let relation_targets: Vec<String> = memory
            .relations
            .iter()
            .map(|rel| normalize_backlink_target(&rel.target, known_uids, key_to_uid))
            .collect();
        relation_storage.insert(memory.uid.clone(), relation_targets);
    }

    let wikilinks_by_uid: BTreeMap<&str, Vec<&crate::links::Wikilink>> = wikilink_storage
        .iter()
        .map(|(uid, links)| (uid.as_str(), links.iter().collect()))
        .collect();
    let relations_by_uid: BTreeMap<&str, Vec<&str>> = relation_storage
        .iter()
        .map(|(uid, targets)| (uid.as_str(), targets.iter().map(String::as_str).collect()))
        .collect();

    backlinks_index(wikilinks_by_uid, relations_by_uid)
}
```

Then `backlink_rows_for` can be refactored to call `backlinks_index_for_all`
(DRY) — currently it duplicates the same loop. This is a behaviour-preserving
refactor of existing code; existing tests must stay green.

### 6. Wire dispatch

In the `dispatch` match arm (~line 528):

```rust
MemoryCommand::List {
    memory_type,
    list,
    path,
    orphans,                         // ← NEW: destructure
} => run_list(
    &mut io::stdout(),
    path,
    memory_type,
    list.into_list_args(color),
    orphans,                         // ← NEW: pass through
),
```

### 7. Update callers of `run_list`

Check for any other call sites of `run_list` (e.g., MCP handler, boot path) —
add `orphans: false` to each.

### 8. Gate check

Callsites in `src/commands/guard.rs` — the `MemoryCommand::List` pattern match
needs the new field destructured (or `..` catch-all).

### Tests (TDD)

1. `test_orphans_flag_returns_true_orphans`: seed 4 memories — A links to B, B
   links to C (via wikilink), D is fully isolated. `list --orphans` returns
   only D.
2. `test_orphans_flag_with_type_filter`: seed orphan fact + orphan pattern;
   `list --orphans --type pattern` returns only the pattern.
3. `test_orphans_flag_key_resolved_backlinks`: A has key `mem.foo.bar`, B has
   wikilink `[[mem.foo.bar]]`. A is NOT an orphan (has inbound via key
   resolution).
4. `test_orphans_flag_empty_corpus`: no memories, no panic, empty output.
5. `test_backlinks_index_for_all_matches_per_memory`: the new corpus-wide
   function produces the same backlinks as calling `backlink_rows_for` for each
   memory individually (refactor gate).

## References

- [[mem.concept.backlog.work-intake-membership]] — backlog membership test
- `src/links.rs` — `backlinks_index`, `extract_wikilinks`, `resolve_wikilink`
- `src/memory.rs` — `backlink_rows_for` (~line 2927), `filtered_list` (~2718),
  `list_rows` (~2730), `run_list` (~2771), `normalize_backlink_target` (~2910),
  `known_link_maps`
- `.agents/skills/dreaming/SKILL.md` step 3
