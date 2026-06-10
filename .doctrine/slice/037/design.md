# SL-037 Design — Shared list column model

Resolves IMP-009 (slug-free defaults), IMP-013 (the deferred list/show shape
lift — this slice's edit is its trigger), IMP-014 (cross-verb golden harness).

## 1. Problem & current behaviour

`src/listing.rs` (SL-025) is the pure read spine: it owns the *invariant* axes —
`Filter`/`FilterFields`, `Format{Table,Json}`, `build`, `retain`,
`render_table(grid)`, `json_envelope`, `validate_statuses`, `canonical_id`. The
*variant* axis — which columns a kind shows and how — is hand-rolled in each
kind's bespoke `format_rows`/`render_table` (table grid) and typed `*Row` struct
(JSON). Four list verbs render a `slug` column today:

| verb | table columns | title col? | JSON row type |
|---|---|---|---|
| backlog | `id kind status slug title` | yes | `BacklogRow` |
| slice | `id status[?][⚠] phases slug title` | yes | `SliceRow` (phases structured) |
| spec | `id status slug #members` | **no** | `SpecRow` (members: usize) |
| governance (adr/policy/standard) | `id status slug title` | yes | `GovRow` (all String) |
| memory | `uid type status trust key title` | n/a (no slug) | `MemoryRow` — **deferred, stays bespoke (D9 / IMP-017)** |

Problems: (a) slug is long, volatile, and *never authoritative* (the prefixed id
is identity) yet dominates table width — IMP-009; (b) the per-kind table-grid
assembly is duplicated structure that IMP-013 deferred lifting until an edit next
reshaped it — this slice is that edit; (c) the cross-verb render surface has no
golden net — IMP-014.

## 2. Decisions

- **D1 — Lift the *table* column projection into a shared column model on
  `listing.rs`.** A per-kind ordered set of columns drives the table; selection
  is shared, declaration is per-kind. This is the IMP-013 lift, bounded to the
  table axis.
- **D2 — JSON stays typed per-kind; the column model is table-only.** A
  `String` cell extractor cannot carry JSON's structured values (slice `phases`
  is an object, spec `members` an int). The shared JSON part (`json_envelope` +
  the build→retain→sort skeleton) is *already* lifted in SL-025; the typed `*Row`
  struct is irreducibly per-kind. `--columns` does not touch JSON. Preserves
  SL-025 D7 (faithful rows). *(User-confirmed.)*
- **D3 — `--columns a,b,c` is the opt-in API** (supersedes a one-off `--slug`
  boolean — the long-term presentation surface). Selects and orders visible table
  columns by name; validated against the kind's available set with one uniform
  error (parity with `validate_statuses`, SL-025 A-2). `None` → the kind's
  default set.
- **D4 — Default visible sets omit slug; spec swaps slug→title.** slug remains an
  *available* column (revealed by `--columns …,slug`). *(User-confirmed swap.)*
- **D5 — Extractors are non-capturing `fn(&R)->String`.** External context
  (e.g. the governance id prefix) is pushed into the row type `R`, not captured.
  Keeps `Column<R>` simple/`Copy`, no `Box<dyn Fn>` in the leaf.
- **D6 — `build()`'s signature is unchanged; columns ride `ListArgs`.** The
  verb pulls `let columns = args.columns.take();` *before* `build(args)`, so
  `build` keeps returning `(Filter, Format)` and its ~10 in-leaf tests + 6 call
  sites stay green unchanged (behaviour-preservation on shared machinery). The
  `columns` field is additive; every existing `ListArgs` literal uses
  `..Default::default()`, so none break. *(Revised from a `Presentation` return
  after the adversarial pass found the destructure churn; user deferred the
  internal seam.)*
- **D7 — `--columns` under `--json` is ignored** (JSON is faithful/full),
  documented on the flag. Minor; revisit if a JSON projection is ever wanted.
- **D8 — IMP-014 golden harness rides this slice** as the regression net for the
  cross-verb table/JSON churn. *(User-confirmed.)*
- **D9 — memory is deferred; it stays bespoke this slice.** The column model
  migrates the four slug-bearing verbs only. Memory has no slug (IMP-009's driver
  is absent), its cells are security-scrubbed (`scrub_line`, memory-spec §
  Security — a generic column layer risks a future unscrubbed column), and it is
  the strongest case of the over-abstraction IMP-013 warned of. Migrating it has
  no triggering edit. `--columns` rides the shared `CommonListArgs`, so it is
  **accepted-but-ignored on `memory list`** (documented no-op). Memory's adoption
  is deferred-until-condition under **IMP-017** (trigger: next edit to memory list
  rendering). *(User-confirmed after an explicit challenge.)*

## 3. The column model (`listing.rs`, new)

```rust
/// One table column for a kind's row type `R`: a `name` (the `--columns`
/// selector token — shell-safe, lowercase), a `header` (display text; usually
/// == name), and a pure non-capturing cell extractor (D5). Table-ONLY (D2).
/// NOT `#[derive(Copy)]` — derive would add a spurious `R: Copy` bound; columns
/// are only ever borrowed (`&[Column<R>]` / `Vec<&Column<R>>`), never moved.
pub(crate) struct Column<R> {
    pub(crate) name: &'static str,
    pub(crate) header: &'static str,
    pub(crate) cell: fn(&R) -> String,
}

/// Resolve the visible, ordered selection. `requested` = parsed `--columns`
/// (None → `default`, taken verbatim). Each requested name is validated against
/// `available`; an unknown name is one uniform `anyhow` error listing the
/// available tokens (A-2 parity). Requested order is preserved; duplicates are
/// permitted (the user asked for them) — OQ-2 resolved: subset+order, dups pass.
pub(crate) fn select_columns<'a, R>(
    available: &'a [Column<R>],
    default: &[&str],
    requested: Option<&[String]>,
) -> anyhow::Result<Vec<&'a Column<R>>> {
    let pick = |name: &str| {
        available.iter().find(|c| c.name == name).ok_or_else(|| {
            let known = available.iter().map(|c| c.name).collect::<Vec<_>>().join(", ");
            anyhow::anyhow!("unknown column `{name}` (available: {known})")
        })
    };
    match requested {
        None => default.iter().map(|n| pick(n)).collect(), // default names are curated-valid
        Some(names) => names.iter().map(|n| pick(n)).collect(),
    }
}

/// Header row + one cell-row per `R`, over `render_table`. Empty rows → ""
/// (header suppressed, §5.5). Replaces every kind's bespoke table assembler.
pub(crate) fn render_columns<R>(rows: &[R], cols: &[&Column<R>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);
    grid.push(cols.iter().map(|c| c.header.to_string()).collect());
    grid.extend(rows.iter().map(|r| cols.iter().map(|c| (c.cell)(r)).collect()));
    render_table(&grid)
}
```

`ListArgs` gains `columns: Option<Vec<String>>` (additive; `Default`-friendly).
The verb reads it via `args.columns.take()` before `build(args)` (D6); `build`
and `render_table` are untouched (the column layer sits above the latter).

## 4. Per-kind migration

Each kind: pick a table row type `R`, declare `const`/fn `available: [Column<R>]`
+ `default: &[&str]`, then `select_columns(...).render_columns(rows, &sel)`.
JSON paths unchanged.

- **governance** — `R = GovRow` (already all-String, id pre-prefixed → satisfies
  D5; the JSON and table rows coincide here). Columns: `id status slug title`;
  default `[id, status, title]`. Built per-call where `GovKind` is in scope so
  the prefixed id is materialised before extraction.
- **backlog** — `R = BacklogItem` (or a small display row). Columns
  `id kind status slug title`; default `[id, kind, status, title]`.
- **slice** — `R = (Meta, Option<PhaseRollup>)` (the *existing* row tuple).
  Extractors reuse `canonical_id`, `decorated_status`, `phases_cell` as
  non-capturing closures. Columns `id status phases slug title`; default
  `[id, status, phases, title]`. (`status` column keeps the `?`/`⚠` markers — a
  column *value*, not a separate column.)
- **spec** — `R = (Meta, usize)` (existing). Columns `id status slug title
  members` with `members` carrying `header = "#members"` (the `#` is
  shell-hostile as a token). Default `[id, status, title, members]` — the
  slug→title swap (D4). spec still emits one labelled block per subtype: it
  resolves the selection once and calls `render_columns` per block.

Net deletion: each kind's bespoke header/grid assembler collapses into a column
table + one `render_columns` call.

## 5. `--columns` flag wiring (`main.rs`)

`CommonListArgs` gains:

```rust
/// Select/order visible table columns, e.g. `--columns id,status,slug`. Unknown
/// names error with the available set. No effect with `--json` (D7); ignored on
/// `memory list`, which is not on the column model this slice (D9 / IMP-017).
#[arg(long, value_delimiter = ',')]
pub(crate) columns: Option<Vec<String>>,
```

`into_list_args` lowers it to `ListArgs.columns`. No new positional/bool, so no
clippy arg-ceiling risk (mem `cli-handler-args-struct`). Validation lives in the
leaf (`select_columns`), not clap — `--columns` is a free `Vec<String>` (a typed
`ValueEnum` per kind would drag kind-specific enums into the shared bundle and
re-fracture the spine, A-3).

## 6. Verification

- **Unit (per kind, pure):** default table omits slug (spec shows title);
  `--columns id,slug` selects + orders + reveals slug; an unknown column errors
  with the available list; JSON output byte-identical to pre-change (slug + typed
  values intact); filter still matches on slug when hidden.
- **`select_columns` (leaf):** default path; requested subset/reorder; duplicate
  names pass; unknown name → uniform error; empty available guard.
- **IMP-014 golden harness (new):** a cross-verb black-box test (mem
  `black-box-cli-golden`, `stale-cargo-bin-exe`) over a fixed corpus pinning,
  per verb, the bytes of: default table, `--columns` table, and `--json`. Asserts
  every surface, not just the JSON envelope (mem `conformance-asserts-surface`).
- `cargo clippy` zero warnings (bins/lib only — not `--all-targets`); `just
  check` green; behaviour-preservation gate: pre-existing filter/JSON suites stay
  green unchanged (D2 guarantees JSON is untouched).

## 7. ADR / governance alignment

- **ADR-001 (module layering).** The model lives in the `listing` leaf; clap
  stays command-side (D3/§5). No engine→command cycle. ✓
- **SL-025 design (A-1/A-2/A-3/D7).** Reuses the one-error validation idiom
  (A-2), keeps clap out of the leaf (A-3), preserves faithful JSON (D7, via D2).
- **ADR-004 (outbound-only relations).** Backlog→slice edges stay deferred
  (C-VII); IMP-009/013/014 reconcile to terminal at `/close`.

## 8. Risks

- **R1 — the IMP-013 config-surface warning.** IMP-013 feared a shared
  row-builder needing more config than the duplication removed. Mitigation: D2
  bounds the lift to the table axis and D5 keeps extractors trivial; slice's
  markers and spec's count are absorbed as ordinary column values, no
  per-kind config flags. *Most worth attacking in the adversarial pass.*
- **R2 — wide cross-verb churn.** Every list golden moves. Mitigation: the
  IMP-014 harness (D8) lands as the net.
- **R3 — spec subtype labelling.** The per-subtype block structure must survive
  the column lift (selection resolved once, applied per block). Covered by §4 +
  a spec golden.
- **R4 *(resolved, D9)* — memory is the 6th `build()` caller on the shared
  spine** (`memory list` consumes `CommonListArgs`, `main.rs:1245`), so
  `--columns` reaches it. Resolved by deferring memory (D9): it stays bespoke,
  `--columns` is an accepted-but-ignored no-op there (documented), and adoption
  is tracked under IMP-017. The "uniform spine" claim is about the *filter* axes
  (the query surface SL-025 guards), not presentation — so the no-op is a bounded
  documented gap, not a spine fracture.
- **R5 — memory's `scrub_line` security invariant** (hostile-input defense on
  trust/key/title cells). Out of scope here (memory deferred), but IMP-017
  records it as the invariant any future memory column model must carry into its
  extractors + pin with a security test.

## 9. Open questions

- OQ-1 *(resolved, D3)* — unknown-column error shape: reuse `validate_statuses`
  idiom.
- OQ-2 *(resolved, §3)* — `--columns` semantics: ordered subset, duplicates
  permitted.
- OQ-3 *(resolved, D9)* — memory vs the shared `--columns` flag. Resolved:
  memory is **deferred** (stays bespoke; `--columns` accepted-but-ignored,
  documented; adoption tracked under IMP-017). Chosen after an explicit challenge
  weighed the security-scrubbing coupling + the absent trigger against bare
  flag-uniformity.
