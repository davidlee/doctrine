# Design: SL-105 — `after` edge removal

## 1. CLI surface

### 1.1 `--remove` flag on `doctrine after`

```
doctrine after <SRC> <TGT> --remove [--rank N]
```

| Flag | Behaviour |
|------|-----------|
| `--remove` alone | Remove **all** `after` edges where `to == TGT`, regardless of rank |
| `--remove --rank N` | Remove edges where `to == TGT` AND `rank ≤ N` |

Output:
- Success: `IMP-059 after IMP-008 removed (2 edges)` or `(1 edge)`
- No match: error — `IMP-059 has no after edge to IMP-008`
- Idempotent re-run: same error (removal is deliberate, not silently ignored)

The `--rank` flag changes semantics between append (sets the rank) and remove
(upper bound). Documented in CLI help text.

### 1.2 `--prune` subcommand

```
doctrine after <SRC> --prune
```

Scans SRC's `after` array, probes each target:
- **Absent** (dir missing) → dropped
- **Terminal** (status `resolved` or `closed`) → dropped
- **Present + non-terminal** → kept

Output one line per dropped edge:
```
IMP-059 after IMP-008 (rank 0) dropped (dangling: IMP-008 resolved/fixed)
IMP-059 after IMP-028 (rank 2) dropped (dangling: IMP-028 resolved/done)
```

No-op prints: `IMP-059: nothing to prune`

`--remove` and `--prune` are mutually exclusive (`conflicts_with`).

## 2. Leaf layer (`src/dep_seq.rs`)

### 2.1 Pure core: `remove_after`

```rust
/// Remove `after` edges from `[relationships].after` matching `to`.
/// `rank_ceiling`: None → all ranks; Some(n) → only edges where rank ≤ n.
///
/// # Returns
/// Number of edges removed (0 if none matched).
///
/// # Errors (F-1)
/// `[relationships].after` array absent → bail (malformed entity, never create).
///
/// # Edit-preserving
/// Removes inline-table entries in reverse order via `toml_edit::Array::remove`
/// to avoid index shift. Surrounding content (comments, inert tables) untouched.
pub(crate) fn remove_after(
    doc: &mut toml_edit::DocumentMut,
    to: &str,
    rank_ceiling: Option<i32>,
) -> anyhow::Result<usize>
```

Implementation sketch:
1. Navigate `doc["relationships"]["after"]` as `&mut Array`
2. Collect indices of matching inline tables (forward scan)
3. Remove in reverse index order via `array.remove(idx)`
4. Return count

Matching predicate per inline table:
- `t.get("to").as_str() == Some(to)`
- AND, if `rank_ceiling` is `Some(n)`: `t.get("rank").as_integer() <= Some(i64::from(n))`

### 2.2 IO wrapper: `remove`

```rust
/// Read→parse→core→write-once wrapper. Returns count removed.
/// No-op (0 removed) leaves the file untouched (mtime/content hold).
/// F-1: absent array bails (same as core).
pub(crate) fn remove(
    toml_path: &Path,
    to: &str,
    rank_ceiling: Option<i32>,
) -> anyhow::Result<usize>
```

Identical pattern to existing `append`/`set_authored_status`: read `to_string`,
parse `DocumentMut`, call core, `fs::write` only if `count > 0`.

### 2.3 No changes to existing types

`RelEdit` gains no new variant — `remove_after` takes `to` and `rank_ceiling`
directly, not via `RelEdit`. The `RelEdit` enum serves the append code path
exclusively. Adding a removal variant would create dead arms in every append
caller — unnecessary.

## 3. Command shell

### 3.1 Source-only path resolution (refactor, `src/main.rs`)

`resolve_dep_seq_src(root, source, target)` lives in `main.rs` and currently
validates both source (work-like gate, self-edge refuse) and target (resolve,
work-like gate). Extract the source-only half:

```rust
/// Resolve a dep/seq source to its TOML path. Validates: canonical-ref parse,
/// work-like kind (slice or backlog). Returns the resolved path.
fn resolve_dep_seq_src_path(root: &Path, source: &str) -> anyhow::Result<PathBuf>
```

`resolve_dep_seq_src` calls `resolve_dep_seq_src_path` then validates the
target. `run_after_prune` calls `resolve_dep_seq_src_path` directly — no
target to validate.

### 3.2 Generic cross-kind CLI (`src/main.rs`)

**CLI enum changes:**

`Command::After` gains:
```rust
/// Remove matching edges instead of appending.
#[arg(long, conflicts_with = "prune")]
remove: bool,

/// Drop every dangling after edge from the source entity.
#[arg(long, conflicts_with = "remove")]
prune: bool,
```

`--remove` and `--prune` both live on `Command::After` (not a separate variant).

**`target` becomes optional**: currently `target: String` is a required positional.
Changed to `Option<String>` with `required_unless_present("prune")` — required
for append/remove, absent for prune. When `prune` is set and `target` is `None`,
the prune path is taken.

The `--rank` flag on `After` is reused — its semantics change is documented in
the help text: "On append: sets the new edge's rank (default 0). On --remove:
upper bound — only edges with rank ≤ N are removed. Ignored with --prune."

**New functions:**

```rust
/// `doctrine after <SRC> <TGT> --remove [--rank N]`
fn run_after_remove(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
    rank_ceiling: Option<i32>,
) -> anyhow::Result<()>

/// `doctrine after <SRC> --prune`
fn run_after_prune(
    path: Option<PathBuf>,
    source: &str,
) -> anyhow::Result<()>
```

`run_after_remove` reuses `resolve_dep_seq_src` for source/target validation,
then calls `dep_seq::remove`. If 0 edges removed, bails with a user-facing error.

`run_after_prune`:
1. Resolve source via `resolve_dep_seq_src_path` (source-only — no target needed)
2. Read `DepSeq` via `dep_seq::read`
3. For each `after` edge: probe target existence + terminality. Collect
   dangling findings into `Vec<DroppedEdge>` (target, rank, reason) for
   reporting, AND deduplicate target ids into a `BTreeSet<String>`
4. For each unique target: `dep_seq::remove(toml_path, &target, None)` —
   removes ALL edges to that target in one pass (avoids sloppy per-rank
   iteration where a wider rank_ceiling would re-match already-removed edges)
5. Print one line per `DroppedEdge` from the pre-collected list, or
   "nothing to prune" if empty

**Target probe for prune:**
- `integrity::parse_canonical_ref(target)` → resolve kind + id
- `integrity::ensure_ref_resolves(root, target)` → absent if fails
- Read target TOML as generic `toml::Value`, check `status` field → terminal if
  `resolved` or `closed`. Works for both backlog items and slices (both carry
  a top-level `status` key).
- Reason string: `"{status}/{resolution}"` for terminal items (matching
  `classify_dangling` format), `"absent"` for missing targets.

### 3.3 Backlog-specific (`src/backlog.rs`)

`BacklogCommand::After` gains `remove: bool` and `prune: bool` flags
(parallel to `Command::After`). `to` becomes `Option<String>` with
`required_unless_present("prune")` — required for append/remove, absent for prune.

`run_after` branches on `remove`/`prune`:
- Append (neither): existing behaviour
- `remove`: resolves source + target via `require_item`, calls `dep_seq::remove`
- `prune`: resolves source only via `require_item`, runs prune loop (same
  logic as `run_after_prune` but using backlog's `require_item` for the source
  and its own entity-kind path resolution for target probes)

Access classification: both `remove` and `prune` modes are `Write(...)`.

## 4. Test plan

### 4.1 Unit tests (`dep_seq.rs`)

| Test | What |
|------|------|
| `remove_after_all_matching` | 3 edges to X, remove all → count=3, array empty of X |
| `remove_after_rank_ceiling` | edges to X with ranks 0,2,5; ceiling=2 → removes rank 0 and 2 only, count=2 |
| `remove_after_no_match` | no edge to Y → count=0, file unchanged |
| `remove_after_mixed_targets` | edges to X, Y, Z; remove X → only X gone, Y and Z untouched |
| `remove_after_f1_refuse` | absent `after` array → error, file untouched, message non-destructive |
| `remove_io_noop_holds_mtime` | no match → mtime unchanged |
| `remove_io_round_trip_preserves_structure` | comments, inert tables survive removal |
| `remove_after_empty_array` | empty `after = []`, remove anything → count=0 |

### 4.2 E2E golden tests

| Test | What |
|------|------|
| `after_remove_single` | `doctrine after SRC TGT --remove` → removes one edge, prints count |
| `after_remove_rank_ceiling` | `doctrine after SRC TGT --remove --rank 1` → removes rank 0 edge, keeps rank 5 |
| `after_remove_nonexistent` | `doctrine after SRC ABSENT --remove` → error exit |
| `after_prune_drops_resolved` | SRC with edge to resolved item → prunes, prints dropped line |
| `after_prune_noop` | SRC with all live edges → prints "nothing to prune" |
| `after_prune_mixed` | mix of live + resolved + absent → prunes only dangling, keeps live |

### 4.3 Manual verification

After implementation, run `doctrine backlog list` — the 15 `overrides:` lines
from the current state should be gone (once all affected entities are pruned).

## 5. Design decisions

| Decision | Rationale |
|----------|-----------|
| `--remove` errors on no-match, not silent no-op | Symmetric inverse of append (idempotent) would be confusing — removal is a deliberate destructive act; silently doing nothing invites mistakes |
| `--remove --rank N` is a ceiling, not exact match | User's stated preference. Enables "clean up the low-rank stale edges" without specifying each exact rank |
| Prune reports one line per edge with rank | Enables restoration if needed; matches `overrides:` footer format |
| Probe logic in command shell, not leaf | ADR-001: leaf stays pure (no knowledge of project, entity kinds, or terminality) |
| No `RelEdit` variant for removal | Would create dead arms in every append caller. Separate function, cleaner coupling |
| Prune removes all edges per unique dangling target in one pass | Dedup target IDs, then `remove_after(doc, to, None)` once — avoids sloppy per-rank iteration where a wider rank_ceiling would re-match already-removed edges |

## 6. Risks and open questions

- **Q: Can `--prune` race with another agent resolving an item mid-prune?**
  No — the probe reads the target TOML at a point in time; if the target was
  resolved between probe and removal, the edge is still removed (it was
  dangling at probe time). This is a soft edge — no correctness hazard.

- **Q: What if `--prune` is run on a resolved/closed source?**
  No harm — resolved sources can't carry dangling edges for ordering, but
  removing their `after` edges is benign. The command shell doesn't gate on
  source status.

- **Q: Why not a separate `AfterPrune` CLI variant?**
  A flag on `After` is simpler: `--remove` and `--prune` are alternative modes
  of the same verb. A separate variant would duplicate `source`, `path`, and
  `--rank` (ignored on prune). `conflicts_with` ensures exactly one mode.
