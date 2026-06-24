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

Corpus-wide application (precompute pattern — build once, filter O(1)):
1. Collect all memories via `collect_all` (TOML-only, no body reads — cheap)
2. Build `known_uids` + `key_to_uid` maps
3. Build the full backlinks index (one pass over all memories, reads all bodies)
4. Precompute `has_outbound: BTreeSet<String>` — uids with relations OR body
   wikilinks (one pass, reads all bodies)
5. For each filtered row, two O(1) set lookups: `!has_outbound.contains(uid)
   && no backlinks entry`. No per-row body reads.

**Performance**: O(n) body reads total (step 3 + 4), O(1) per-row filter
(step 5). The original sketch had the backlinks index rebuilt inside `retain`
→ O(n²) — fixed here.

**Double `collect_all`**: `filtered_list` already calls `collect_all`
internally. The orphan path calls it again for the index build. Accepted for
now — `collect_all` is TOML-parse-only (no body reads), fast enough. Future
cleanup could refactor `filtered_list` to accept a pre-collected `&[Memory]`.

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

### 3. Add orphan filter in `list_rows` — precompute, then inline closure

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

Add `orphans: bool`. When true, build the backlinks index + outbound set **once**
before the filter, then use an inline closure with O(1) set lookups. No
standalone `is_orphan` function — the signature would invite O(n²) misuse:

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

    if orphans {
        let all = collect_all(root)?;
        let (known_uids, key_to_uid) = known_link_maps(&all);
        let backlinks = backlinks_index_for_all(&all, root, &known_uids, &key_to_uid);

        // Precompute: which uids have ANY outbound (relations OR body wikilinks)
        let has_outbound: BTreeSet<String> = all
            .iter()
            .filter(|m| {
                !m.relations.is_empty()
                    || !extract_wikilinks(&read_body(root, &m.uid)).is_empty()
            })
            .map(|m| m.uid.clone())
            .collect();

        rows.retain(|m| {
            let no_outbound = !has_outbound.contains(&m.uid);
            let no_inbound = backlinks.get(&m.uid).map_or(true, BTreeSet::is_empty)
                && m.key.as_ref().map_or(true, |key| {
                    backlinks.get(key).map_or(true, BTreeSet::is_empty)
                });
            no_outbound && no_inbound
        });
    }

    match format { /* ... render unchanged ... */ }
}
```

**Design decision**: no standalone `is_orphan(memory, all, root)` function.
The signature invites the O(n²) mistake (rebuilding the index per row). The
inline closure captures the precomputed structures, making the correct pattern
the only pattern.

### 4. New helper: `backlinks_index_for_all` (corpus-wide)

Extract the body-scanning loop from `backlink_rows_for` into a reusable
corpus-wide function. **Write the refactor gate test first** (test 5 below)
then extract — existing `backlink_rows_for` delegates to the new function.
Behaviour-preserving: existing `memory show` and MCP `memory_show` tests
must stay green unchanged.

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

### 5. Wire dispatch

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

### 6. Update callers of `run_list`

Check for any other call sites of `run_list` (e.g., MCP handler, boot path) —
add `orphans: false` to each.

### 7. Gate check

Callsites in `src/commands/guard.rs` — the `MemoryCommand::List` pattern match
needs the new field destructured (or `..` catch-all).

### Tests (TDD — write in this order)

1. `test_backlinks_index_for_all_matches_per_memory` (**refactor gate, write
   first**): the new corpus-wide function produces the same backlinks as
   calling the existing `backlink_rows_for` for each memory individually.
   Seed 3 memories with wikilinks + relations, compare the two paths.
   Red → extract → green → refactor `backlink_rows_for` to delegate.
2. `test_orphans_flag_returns_true_orphans`: seed 4 memories — A links to B, B
   links to C (via wikilink), D is fully isolated. `list --orphans` returns
   only D.
3. `test_orphans_flag_with_type_filter`: seed orphan fact + orphan pattern;
   `list --orphans --type pattern` returns only the pattern.
4. `test_orphans_flag_key_resolved_backlinks`: A has key `mem.foo.bar`, B has
   wikilink `[[mem.foo.bar]]`. A is NOT an orphan (has inbound via key
   resolution).
5. `test_orphans_flag_empty_corpus`: no memories, no panic, empty output.

## References

- [[mem.concept.backlog.work-intake-membership]] — backlog membership test
- `src/links.rs` — `backlinks_index`, `extract_wikilinks`, `resolve_wikilink`
- `src/memory.rs` — `backlink_rows_for` (~line 2927), `filtered_list` (~2718),
  `list_rows` (~2730), `run_list` (~2771), `normalize_backlink_target` (~2910),
  `known_link_maps`
- `.agents/skills/dreaming/SKILL.md` step 3
