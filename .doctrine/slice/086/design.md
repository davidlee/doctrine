# SL-086 Design: Agent-facing CLI UX hardening

## 1. IMP-090 — Positional query on `memory find`

### Current behavior

`doctrine memory find cli` → `unexpected argument 'cli' found`. Only `--query
<QUERY>` works. The error gives no hint toward the correct flag.

### Target behavior

`doctrine memory find cli` ≡ `doctrine memory find --query cli`. Zero or one
positional arg. Mutually exclusive with `--query` — providing both is an error.

### Code impact

**`src/main.rs`** — `MemoryCommand::Find` variant:

```rust
Find {
    /// Positional query (zero or one; maps to --query).
    query: Option<String>,           // NEW positional

    #[arg(long = "path-scope")]
    path_scope: Vec<String>,
    // ... existing fields unchanged ...
}
```

- Rename the existing `--query` field to `flag_query` (with
  `#[arg(long = "query")]` to preserve the user-facing flag name) to avoid
  namespace collision with the positional `query`. Both are `Option<String>`;
  if both are `Some` → `anyhow::bail!("cannot specify both a positional query
  and --query")`.
- Dispatch passes `query.or(flag_query)` to `retrieve::run_find` as `free_query`.

**`src/retrieve.rs`** — `run_find` signature unchanged (it already accepts `free_query:
Option<String>`). The merge happens in the shell (`main.rs`), not in the pure
layer.

### Design decisions

- D1: Mutually exclusive positional + `--query`. Clear, no surprise. The scope
  says "maps to --query" which implies substitution.
- D2: Merge in the shell (main.rs), not in the retrieve module. The retrieve
  layer stays clap-free (ADR-001).

---

## 2. IMP-091 — Pagination + truncation notice

### Current behavior

`--limit` truncates silently. Agent has no way to know results were truncated.

### Target behavior

Both `memory find` and `memory retrieve` get `--offset N` and `--page N`.
When results are truncated, a trailing notice line appears (human output only):

```
5 of 142; --page 2 for next or specify a higher --limit
```

`--json` output suppresses the notice — the array length communicates count.

### Flag semantics

- `--offset N` — skip first N results. Default 0.
- `--page N` — sugar: `offset = (page - 1) * limit`. Mutually exclusive with
  `--offset`. Page 1 = no offset (default).
- Both clap-validated: `--page 0` → error, `--offset` accepts `usize`.

### Code impact

**`src/main.rs`** — both `Find` and `Retrieve` variants:

```rust
Find {
    // ... existing fields ...
    /// Skip first N results (default 0).
    #[arg(long, default_value_t = 0)]
    offset: usize,

    /// Page number (1-based; sugar over offset). Mutually exclusive with --offset.
    #[arg(long, conflicts_with = "offset")]
    page: Option<usize>,

    /// Max results to show.
    #[arg(long)]
    limit: Option<usize>,
    // ...
}
```

Pre-dispatch resolution:
```rust
let offset = match page {
    Some(p) if p == 0 => anyhow::bail!("--page must be >= 1"),
    Some(p) => (p - 1) * limit.unwrap_or(RETRIEVE_LIMIT_DEFAULT),
    None => args.offset,
};
```

**`src/retrieve.rs`** — both `run_find` and `run_retrieve` gain pagination params.

Resolution (shell-side, before calling retrieve):

```rust
// --limit 0 is rejected for both commands.
if args.limit == Some(0) {
    anyhow::bail!("--limit must be >= 1");
}

// find: limit defaults to None (unlimited, all results shown).
//       When --limit is explicitly set, cap at RETRIEVE_LIMIT_MAX.
let find_limit: Option<usize> = args.limit.map(|l| l.min(RETRIEVE_LIMIT_MAX));

// retrieve: limit defaults to RETRIEVE_LIMIT_DEFAULT (5), capped at MAX.
let retrieve_limit: usize =
    args.limit.unwrap_or(RETRIEVE_LIMIT_DEFAULT).min(RETRIEVE_LIMIT_MAX);

// Page offset: page size uses explicit --limit or RETRIEVE_LIMIT_DEFAULT.
let page_size = args.limit.unwrap_or(RETRIEVE_LIMIT_DEFAULT);
let resolved_offset = match args.page {
    Some(p) if p == 0 => anyhow::bail!("--page must be >= 1"),
    Some(p) => (p - 1) * page_size,
    None => args.offset,
};
```

**`run_find`** — signature gains `offset: usize`, `limit: Option<usize>`, `format: Format`.
Before rendering, slice the ranked candidates:
```rust
let total = ranked.len();
let visible: Vec<&Candidate> = ranked.iter()
    .skip(offset)
    .take(limit)
    .collect();
let shown = visible.len();
// render visible (format_find_table or format_find_json)
```

**`run_retrieve`** — signature gains `offset: usize`. The holdback + pagination
pipeline changes from `select_shown(filter, limit)` to:
```rust
// Holdback is applied before counting/pagination.
let eligible: Vec<&Candidate> = ranked.iter()
    .filter(|c| !held_back(c.memory, floor))
    .collect();
let total = eligible.len();
let visible: Vec<&Candidate> = eligible.into_iter()
    .skip(offset)
    .take(limit)
    .collect();
let shown = visible.len();
```
This replaces `select_shown` — holdback-then-offset-then-limit, in that order.

**Truncation notice** — appended after rendering, for both commands:
```rust
if resolved_format == Format::Table && shown < total {
    let next_page = (offset / limit) + 2; // 1-based
    writeln!(io::stdout(), "{shown} of {total}; use --page {next_page} for next or specify a higher --limit")?;
}
```
Edge case: when `offset >= total`, `shown = 0` and `shown < total` still holds —
emit a notice: `0 of 142; no results at this offset; reduce --offset or --page`.

### Design decisions

- D3: `--offset` and `--page` added to both `find` and `retrieve` for
  consistency.
- D4: Truncation notice suppressed under `--json`. Array length is its own
  count signal; a free-text notice line would break JSON parsing.
- D5: Notice uses the next page number (computed from offset/limit), not a
  fixed suggestion like `--limit 20`. If user passes `--limit 8` and `--page 1`,
  notice says `--page 2` not `--limit 20`.
- D6: Total count semantics differ by command:
  - `find`: total = `ranked.len()` (holdback-exempt — all candidates count).
  - `retrieve`: total = post-holdback count (held-back memories are suppressed,
    not truncated; they don't count as "matching").
- D14: `find` has no default limit — without `--limit`, all results are shown.
  Only when `--limit` is explicitly set does truncation apply. `retrieve` retains
  `RETRIEVE_LIMIT_DEFAULT` (5). This preserves the current `memory find`
  behaviour (unlimited) while keeping `memory retrieve` safe against
  accidentally dumping the entire memory store.

---

## 3. IMP-092 — `--json` on `memory find`

### Current behavior

`memory find` outputs only a bespoke human table format via `format_find`. No
`--json` or `--format` flag.

### Target behavior

Add `--format table|json` and `--json` shorthand, using the same `listing::Format`
enum as every other list/find command.

### Code impact

**`src/main.rs`** — `MemoryCommand::Find` variant:

```rust
Find {
    // ... existing fields ...
    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    json: bool,
    // ...
}
```

Pre-dispatch resolution: `let resolved = if json { Format::Json } else { format };`

**`src/retrieve.rs`** — `run_find` gains a `format: Format` parameter. `format_find`
is renamed `format_find_table` and called only under `Format::Table`. A new
`format_find_json` function serializes `&[Candidate]` via the shared envelope:

```rust
fn format_find_json(cands: &[Candidate<'_>]) -> anyhow::Result<String> {
    let rows: Vec<MemoryFindRow> = cands.iter().map(MemoryFindRow::from).collect();
    listing::json_envelope("memory_find", &rows)
}
```

**`MemoryFindRow`** — a new serde struct in `retrieve.rs` mirroring the find
table columns:

```rust
#[derive(Serialize)]
struct MemoryFindRow {
    uid: String,
    #[serde(rename = "type")]
    kind: String,
    status: String,
    staleness: String,
    trust: String,
    severity: String,
    spec: String,        // matched dimension label (or "-")
    title: String,
}
```

### Design decisions

- D7: Do NOT flatten `CommonListArgs` into `Find`. `CommonListArgs` bundles
  `--filter`/`--regex`/`--columns`/`--all` — irrelevant to a search command
  that already has its own scope/filter flags.
- D8: JSON row shape mirrors the find table columns exactly — same field set,
  same names, just structured. No surprise migration for agents switching
  between `--json` and table output.
- D9: `--json` wins over `--format` (standard A-9 precedence from `listing::build`).

---

## 4. IMP-093 — `doctrine status` dashboard

### Current behavior

No single command for project orientation. Agent must run `slice list`, `backlog
list`, `next`, `boot --check`, and `git log` separately.

### Target behavior

`doctrine status [--json]` — single command, 10–20 lines:

```
Work
  slices: 2 active (1 blocked), 4 total
  backlog: 3 issues, 1 improvement, 2 chores, 1 risk
  next up: SL-086 (design), SL-074 (ready)

Blocked slices
  SL-082 blocked by SL-047 — reconcile engine
  SL-074 blocked by SL-077 — dispatch staging

Blocked backlog
  ISS-003 blocked by SL-082 — phase ordering

Boot
  boot.md fresh (2 min ago) from commit a3f7b2c

Recent commits
  a3f7b2c plan(SL-086): phase sheets — 2 min ago
  1f4d9a8 fix(SL-076): check drops parse-time diagnostics — 15 min ago
  9c2e3b1 design(SL-086): lock design — 1 hour ago
```

### Data sources

| Section | Source | Notes |
|---|---|---|
| Work → slices | corpus scan (slice-scoped) | Count active (not `done`/`abandoned`); blocked = active ∧ has unresolved `needs` edges |
| Work → backlog | backlog scan | Count by kind, `open`-only |
| Work → next up | priority engine (`priority::surface::next`) | Top 5 actionable, id+status only |
| Blocked slices | dep/seq graph: slices with unresolved `needs` edges AND not terminal | Top 5, each with its blocker ids |
| Blocked backlog | dep/seq graph: backlog items with unresolved `needs` edges AND `open` | Top 5, each with its blocker ids |
| Boot | `boot --check` + file stat | Staleness + commit that wrote it |
| Recent commits | `git log -5 --format="%h %s — %ar"` | Shell (impure); called from `status::run`, passed as data to pure `assemble_status` |

**Blocked = hard `needs` only.** Soft `after` sequence edges are ordering hints, not
blockers (D11). An item is "blocked" only if ≥1 `needs` edge points to a
non-resolved target. Resolved means `done`/`accepted`/`closed`/`resolved` as
appropriate for the target kind.

### Code impact

**New module: `src/status.rs`** — pure leaf (ADR-001):

```rust
// Pure assembly layer: composes existing scan + priority + dep/seq reads.
// Does NOT walk the corpus itself — delegates to per-kind readers.

pub(crate) struct Status {
    pub(crate) work: WorkSection,
    pub(crate) blocked_slices: Vec<BlockedItem>,
    pub(crate) blocked_backlog: Vec<BlockedItem>,
    pub(crate) boot: BootSection,
    pub(crate) recent_commits: Vec<CommitLine>,
}

pub(crate) struct WorkSection {
    pub(crate) slice_count: SliceCounts,
    pub(crate) backlog_counts: BTreeMap<String, usize>, // kind → count
    pub(crate) next_up: Vec<NextItem>,
}

pub(crate) struct BlockedItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) blocked_by: Vec<String>,
}
```

**`src/main.rs`** — new command variant:

```rust
Status {
    /// Output format (table | json).
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    json: bool,

    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}
```

Dispatched to `status::run(path, format, json)`.

**Pure/impure split** (ADR-001):
- `status::assemble_status(root, slice_counts, backlog_counts, next_up, blocked,
  boot_info, commits)` is a pure function: receives all data as plain structs,
  returns `Status`. No clock, rng, git, or disk.
- `status::run` is the impure shell: finds root, calls existing scan functions,
  runs `boot --check` and `git log`, then hands the collected data to
  `assemble_status`.
- `status::render_human` and `status::render_json` are pure render functions
  taking `&Status` → `String`.

### Empty-state handling

- No active work → single line `No active work.`
- No blocked items → suppress "Blocked" sections entirely
- No boot.md → `boot.md missing`
- No commits → suppress "Recent commits" section

### JSON output shape

```json
{
  "kind": "status",
  "work": {
    "slices": { "active": 2, "blocked": 1, "total": 4 },
    "backlog": { "issue": 3, "improvement": 1, "chore": 2, "risk": 1 },
    "next_up": [
      { "id": "SL-086", "status": "design", "title": "..." }
    ]
  },
  "blocked_slices": [
    { "id": "SL-082", "title": "...", "blocked_by": ["SL-047"] }
  ],
  "blocked_backlog": [
    { "id": "ISS-003", "title": "...", "blocked_by": ["SL-082"] }
  ],
  "boot": {
    "staleness": "fresh",      // content-diff (boot_check) — fresh|stale|missing
    "age_seconds": 120,        // mtime delta, informational only
    "commit": "a3f7b2c"
  },
  "recent_commits": [
    { "hash": "a3f7b2c", "subject": "plan(SL-086): ...", "relative_time": "2 min ago" }
  ]
}
```

### Design decisions

- D10: Separate "Blocked slices" and "Blocked backlog" sections — not merged.
  Agents reason about slice pipeline and backlog intake differently.
- D11: "Blocked" = hard `needs` edges only (not soft `after` sequence). Soft
  sequence edges are ordering hints, not true blockers.
- D12: Recent commits via `git log` (impure shell). The boot staleness already
  gives time-since orientation, but commit context is high-value for agent
  orientation and cheap to fetch.
- D13: Boot staleness is driven by content-diff (`CheckReport.stale` from
  `boot_check`). `stale` when on-disk ≠ recomputed; `fresh` when identical;
  `missing` when no boot.md exists. `age_seconds` (mtime delta from file stat)
  is informational only — it does not affect the staleness classification.

---

## 5. Verification alignment

| IMP | VT (test) | EN (entry) | EX (exit) |
|---|---|---|---|
| 090 | Golden test: `doctrine memory find cli` outputs same as `--query cli` | Positional + `--query` together errors cleanly | Positional query finds expected results |
| 091 | Golden test: truncation notice appears when `--limit 2` with 3+ matches; `memory find` without `--limit` shows all results; `--limit 0` errors; `--page 2` alone does not error despite offset default | `--page 0` errors; `--offset` + `--page` conflict | `--page 2 --limit 5` shows results 6-10; `--json` has no notice |
| 092 | Golden test: `memory find --json` output is valid JSON with `kind: "memory_find"` | `--format bogus` errors | JSON rows match table column set 1:1 |
| 093 | Golden test: `doctrine status` output contains expected sections | Command runs to completion on empty repo | `--json` output parses; empty repo shows `No active work.` |

### Goldens

The repo has an established golden-test pattern (see `mem.pattern.test.cli-goldens`).
Each IMP adds at least one black-box CLI golden test pinning the exact output shape.

---

## 6. Non-architectural notes

- No new dependencies required. `serde` and `serde_json` are already in the tree
  (used by all other `--json` commands).
- `git log` is already invoked from `src/git.rs` — the status module's git call
  follows the same impure shell pattern.
- **Args-struct ceiling.** `MemoryCommand::Find` currently has 9 fields, `Retrieve`
  has 11. Adding query+offset+page+limit+format+json pushes Find to 15 and
  Retrieve to 17. **Mitigation**: extract the shared scope/filter fields
  (path_scope, glob, command, tag, memory_type, status, include_draft) plus the
  new pagination/format fields (offset, page, limit, format, json, flag_query)
  into a `FindRetrieveArgs` struct that both variants flatten via
  `#[command(flatten)]`. This is the established pattern (`CommonListArgs`,
  `RecordArgs`) and follows DRY — each shared field is defined once rather than
  duplicated across both variants. The positional query stays on `Find` only.
  Note: `offset` carries `default_value_t = 0` but is mutually exclusive with
  `--page` via `conflicts_with` — clap 4 treats a default_value as "not present"
  for conflict resolution, so `--page 2` alone resolves correctly.
