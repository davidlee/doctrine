# Notes SL-025: Uniform DRY CLI surface: shared list/show/filter/render contract

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — Spine leaf (commit 4314ce8)

Shipped the pure, clap-free read spine `src/listing.rs` and relocated
`render_table` out of `meta.rs`. Zero CLI behaviour change (no kind rides the
spine yet — that begins PHASE-02).

### Public surface (`src/listing.rs`, all `pub(crate)`)
- `canonical_id(prefix: &str, id: u32) -> String` — `{prefix}-{id:03}`. The id-form
  authority for prefixed kinds (memory is exempt; uid is its canonical id). Prefix
  is a *parameter*, not read from `entity::Kind` → leaf imports no entity (EX-4).
- `Format { Table, Json }` — plain enum; `FromStr` (table|json, clean err),
  `Display`, `#[derive(Default)]` with `#[default] Table`. NOT `clap::ValueEnum`
  (A-3); command side wires `#[arg(value_parser = Format::from_str)]`.
- `FilterFields { canonical, slug, title, status, tags }` — one projection per row,
  serving both match domains (A-1). Filter-only; never a render type.
- `Filter { substr, regex, status, tags, all }` — resolved+pre-compiled. No
  `PartialEq` (regex_lite::Regex isn't Eq); hand `Debug` prints the pattern str.
- `ListArgs { substr, regexp, case_insensitive, status, tags, all, format, json }` —
  `#[derive(Default)]` plain struct: the **clap-free leaf mirror** of the
  command-side `CommonListArgs`. (Deviation from design §5.2's illustrative loose
  `build(...)` params — clippy `too_many_arguments` + `fn_params_excessive_bools`
  forced the struct. No contract change; PHASE-02 fills it from parsed CommonListArgs.)
- `build(args: ListArgs) -> Result<(Filter, Format)>` — lowercases substr once,
  pre-compiles regex (case flag baked via `(?i)` prefix; invalid → clean anyhow
  error, no panic), folds `--json` over `--format` (A-9).
- `retain<R>(rows, &Filter, is_hidden: Fn(&str)->bool, key: Fn(&R)->FilterFields)
  -> Vec<R>` — FILTER-ONLY, preserves input order (ordering is per-kind, §5.3).
  substr→slug+title; regex→canonical+slug+title (distinct domains); status OR; tags
  OR; axes AND. Hide-set suppressed when `all` OR any explicit `status`.
- `validate_statuses(given: &[String], known: &[&str]) -> Result<()>` — uniform
  error naming the bad value + the known set (A-2). READ/filter input only.
- `render_table(rows: &[Vec<String>]) -> String` — relocated verbatim from meta.rs
  (incl. `COL_GAP`). Empty → `""` (header suppressed, §5.5).
- `json_envelope<T: Serialize>(kind, rows) -> Result<String>` — `{kind, rows}`,
  pretty-printed.

### Relocation impact (zero output change)
- `meta::render_table` + `COL_GAP` deleted; `meta::format_list` now renders over
  `crate::listing::render_table`. `meta::sort_and_filter` KEPT (the surviving
  sort-by-id helper; its dead filter-half is removed when callers migrate, PHASE-02+).
- Callers repointed to `crate::listing::render_table`: `slice.rs:410`,
  `backlog.rs:636`, `spec.rs:946`. `memory.rs` has its own private renderer (not
  the shared one) — untouched.

### Decisions / gotchas (durable)
- **Self-clearing `#![expect(dead_code)]`** on `listing.rs`: the spine has no
  non-test consumer until PHASE-02, and the repo denies dead_code. Precedent:
  SL-008 PHASE-01 (`retrieve.rs`). PHASE-02 (adr + `listing::build`) retires it —
  an `expect` that becomes fulfilled is itself an error, forcing its removal.
  Recorded as `mem.pattern.lint.dead-code-self-clearing-leaf`.
- regex-lite 0.1 (NOT full regex, D9) added to Cargo.toml + workspace deps.

### Gate
- 581 unit tests pass (was 555 at HEAD: +29 listing, −3 render_table moved out of
  meta). Behaviour-preservation suites (entity, registry, meta readers,
  is_divergent) green **unchanged** (VT-3). `just check` clean (clippy zero
  warnings, fmt). e2e suites green.
