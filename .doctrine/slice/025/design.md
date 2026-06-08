# Design SL-025: Uniform DRY CLI surface: shared list/show/filter/render contract

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The `doctrine` read/inspect surface (`list`, `show`, filtering, output rendering)
is hand-rolled per entity kind. An agent-UX review (preflight under this slice)
found the surface contradicts its own governance and diverges along every read
axis. The findings, F-1…F-7, are enumerated in `slice-025.md`; the headline is
F-1: the boot guardrail says "read entities via `doctrine <kind> show <ID>`, not
raw files," yet `slice show` and `adr show` do not exist.

The design problem is **not** "make every kind's output identical" — kinds carry
legitimately different columns (slice: phase rollup; spec: member count; backlog:
kind + resolution; memory: type + trust). It is: **lift the *invariant* axes of
the read surface (flags, filter semantics, id form, terminal-default, output
format) onto a shared contract, while leaving the *variant* axis (per-kind
columns and per-kind flags) local** — and do so in a shape that minimises churn
over the codebase's lifetime, not just now.

## 2. Current State

The write engine is already kind-blind: `entity.rs` materialises every kind from
a `Kind { dir, prefix, scaffold }` descriptor (D2: "a Kind is *data*, not a
trait"). The **read** side has no equivalent.

Shared substrate today — `meta.rs`, numeric kinds only (slice/adr/spec):
- `Meta { id, slug, title, status }`; `read_meta(s)(tree, stem)`.
- `sort_and_filter(rows, status)` — exact single-status filter only.
- `render_table(&[Vec<String>])` — the one layout authority. Already DRY.
- `format_list(rows)` — bare `{:03}` id, 4 cols, no header (only `adr` still uses
  it).

Hand-rolled, per kind:

| kind | list assembler | id form | header | terminal default | filters | show |
|---|---|---|---|---|---|---|
| adr | `meta::format_list` | bare | no | none | status | **no** |
| slice | own `format_slice_rows` (+phases, ⚠ divergence) | bare | yes | none | status | **no** |
| spec | own `format_spec_rows` (+#members, per-subtype) | bare | yes | none | status | yes |
| backlog | own `BacklogItem`/`ListFilter`/`format_rows` | **prefixed** | no | **hide-terminal** | substr+kind+tag+all | yes |
| memory | own (named/uid entity) | uid | no | none | type+status+tag | yes |

`registry.rs` is the spec-only FK-integrity index — not a general read layer.
`backlog` (SL-020, newest) is the de-facto reference shape: prefixed ids,
hide-terminal default, the richest filter set. It is also the kind that does *not*
share `meta.rs`. `-p/--path` is the one already-uniform flag.

Status vocabularies (terminal predicate inputs): `slice` string (terminal today =
`done` only); `adr` enum proposed/accepted/rejected/superseded/deprecated; `spec`
enum draft/active/superseded; `memory` enum active/draft/superseded/archived;
`backlog` enum open/triaged/started/resolved/closed (terminal = resolved/closed).

## 3. Forces & Constraints

- **ADR-001 module layering** — leaf ← engine ← command, no cycles. The new
  spine is a pure leaf consumed by the command layer.
- **`entity.rs` D2** — Kind is data, not a trait. The design must not reintroduce
  a trait-per-kind shape for the variant axis.
- **Behaviour-preservation gate** — engine suites (`entity.rs`, `registry.rs`,
  `meta.rs` readers) stay green *unchanged*. Output-snapshot tests change
  legitimately (bare→prefixed ids, fewer default rows, new headers).
- **Lifetime-churn metric** (the explicit design driver, §7 D1) — minimise edits
  summed over the codebase's life: a new *shared* flag → one edit; a new *kind*
  → inherits the spine; a new *kind-specific* column/flag → local, zero
  shared-seam blast radius.
- **House lint regime** — `Vec<String>`+`concat` string assembly (no
  `push_str(format!)`); no `as` casts; `BTreeMap/Set` not `Hash*`; `#[expect(…,
  reason)]` not bare `allow`; gate is plain `cargo clippy`.
- **Pure/imperative split** — filter/format/render are pure; only the thin shell
  reads disk and writes stdout.
- **Storage rule** — unaffected; this slice touches presentation, not authored
  data shapes.

## 4. Guiding Principles

1. **Unify the invariant axis, localise the variant axis.** Flags, filter
   semantics, id form, terminal-default, format dispatch are shared and mandatory.
   Columns and kind-specific flags stay in the kind.
2. **Mandatory spine, not convention.** The shared arg bundle and the format
   dispatch are required structure — a kind cannot quietly drift back to bespoke
   flags (the failure mode that produced today's state).
3. **Data over traits for variation; a narrow trait only for the truly uniform.**
   A thin filter-axis cut (status / haystack / tags) may be a small trait or
   accessor closures — explicitly *not* a render abstraction.
4. **Compose, don't pre-partition.** Shared flags live in composable
   `#[derive(Args)]` bundles; which bundle a flag belongs to can emerge as the
   second/third kind needs it.

## 5. Proposed Design

### 5.1 System Model

Two new/changed seams, split by concern (C4-ish component view):

```
command layer (main.rs + per-kind run_list/run_show)
        │  flattens CommonListArgs; supplies per-kind is_terminal + columns
        ▼
listing.rs  (NEW, pure leaf)  ── the kind-blind read spine
   Filter, retain<…>, Format, render dispatch, json envelope, canonical_id,
   render_table (moved from meta.rs)
        │
meta.rs  (kept, narrowed)  ── numeric authored-toml reader
   Meta, read_meta(s), sort_and_filter
        │
entity.rs (unchanged)  ── Kind { dir, prefix, scaffold }; prefix feeds canonical_id
```

`render_table` moves from `meta.rs` to `listing.rs`: it is generic layout, used
by every kind, not numeric-toml-specific. `meta.rs` keeps exactly its numeric
reader charter.

### 5.2 Interfaces & Contracts

**Shared input contract** — one composable bundle, flattened into every `list`
variant (illustrative; field-level clap attrs elided):

```rust
#[derive(clap::Args)]
pub(crate) struct CommonListArgs {
    /// Substring filter on slug+title (case-insensitive).
    #[arg(long, short = 'f')]
    pub filter: Option<String>,
    /// Regex over canonical-id + slug + title.
    #[arg(long, short = 'r')]
    pub regexp: Option<String>,
    /// Make the regex case-insensitive.
    #[arg(long, short = 'i')]
    pub case_insensitive: bool,
    /// Status filter, multi-value (`-s draft,active`); any value reveals terminal.
    #[arg(long, short = 's', value_delimiter = ',')]
    pub status: Vec<String>,
    /// Tag filter, repeatable (OR logic).
    #[arg(long, short = 't')]
    pub tag: Vec<String>,
    /// Show every state, including terminal.
    #[arg(long, short = 'a')]
    pub all: bool,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Table)]
    pub format: Format,
    /// Shorthand for `--format json`.
    #[arg(long)]
    pub json: bool,
}
```

Kind-specific flags (`memory --type`, `backlog --kind`, …) sit beside the flatten
on each kind's `list` variant. A flag shared by a *subset* of kinds becomes its
own small bundle, flattened only by those kinds (compose, don't pre-partition).

`-p/--path` stays a per-variant field (it is a root locator, not a list filter) —
unchanged.

**Shared filter contract** — pure, generic over the filterable axes:

```rust
pub(crate) struct Filter {
    pub substr: Option<String>,           // lowercased once
    pub regex: Option<regex::Regex>,      // pre-compiled (case flag baked in)
    pub status: Vec<String>,              // empty = no status constraint
    pub tags: Vec<String>,                // empty = no tag constraint
    pub all: bool,
}

impl Filter {
    /// Build from args; compiles the regex (clean error on a bad pattern) and
    /// applies the `--json` → `Format::Json` coercion at the call site.
    pub(crate) fn from_args(a: &CommonListArgs) -> anyhow::Result<(Filter, Format)>;
}

/// Keep a row iff: not (terminal-hidden) AND substr-match AND regex-match AND
/// status-match AND tag-match. Terminal-hide is suppressed when `all` OR any
/// explicit `status` is given. Accessors expose the three filterable axes; the
/// terminal predicate is per-kind.
pub(crate) fn retain<R>(
    rows: Vec<R>,
    f: &Filter,
    is_terminal: impl Fn(&str) -> bool,
    status_of: impl Fn(&R) -> &str,
    haystack: impl Fn(&R) -> String,   // slug + " " + title (+ canonical for regex)
    tags_of: impl Fn(&R) -> &[String],
) -> Vec<R>;
```

(Accessor closures, not a trait, keep the variant types — `Meta`, `BacklogItem`,
memory's row — free of a shared supertype; a thin `Filterable` trait is an
acceptable equivalent at build time. Either way it is filter-only, never render.)

**Shared render contract** — pure:

```rust
#[derive(Clone, Copy, clap::ValueEnum)]
pub(crate) enum Format { Table, Json }

/// `SL` + `025` → `"SL-025"`. The single id-form authority.
pub(crate) fn canonical_id(prefix: &str, id: u32) -> String;

/// Generic table layout (moved from meta.rs). Header is row 0 when present.
pub(crate) fn render_table(rows: &[Vec<String>]) -> String;

/// Wrap kind-faithful row values in the shared envelope.
pub(crate) fn json_envelope<T: serde::Serialize>(kind: &str, rows: &[T])
    -> anyhow::Result<String>;   // { "kind": "...", "rows": [ ... ] }
```

**`show` seam** (separate, smaller): a shared ref→(kind, dir, id) resolver +
toml-as-data + md-body reassembly, parameterised like the existing `spec`/
`backlog` `show`. `slice show <SL-NNN>` and `adr show <ADR-NNN>` are added;
`spec`/`backlog`/`memory` `show` gain `--format json`.

### 5.3 Data, State & Ownership

- `listing.rs` owns no state — pure functions over caller-supplied rows.
- Each kind owns its **row type** and its **grid/column projection** (the variant
  axis): slice keeps `phases`/`⚠`, spec keeps per-subtype + `#members`, backlog
  keeps `kind`/`resolution`, memory keeps `type`/`trust`. Each kind owns its row
  **serde shape** for JSON (faithful mirror).
- Each kind owns its `is_terminal(&str) -> bool` predicate. `slice` extends to
  `done || superseded`; `adr` = rejected/superseded/deprecated; `spec` =
  superseded; `memory` = superseded/archived; `backlog` unchanged
  (resolved/closed). Draft is never terminal (memory-local visibility axis).
- The id prefix is owned by `entity::Kind.prefix` (already present) and consumed
  by `canonical_id`. Memory is the exception: its uid *is* its canonical id, so
  it does not route through `canonical_id`.

### 5.4 Lifecycle, Operations & Dynamics

`list` flow, every kind:
1. command layer parses `CommonListArgs` (flattened) + kind-specific flags.
2. `Filter::from_args` → `(Filter, Format)` (regex compiled, `--json` folded).
3. kind reads its rows (existing readers: `meta::read_metas`, `backlog::read_all`,
   memory's lister).
4. `retain(rows, &filter, kind::is_terminal, …)` — shared.
5. branch on `Format`:
   - `Table` → kind assembles its grid (prefixed ids via `canonical_id`, header
     row) → `render_table`.
   - `Json` → kind serialises its faithful rows → `json_envelope`.
6. shell writes to stdout.

`show` flow: resolve ref → kind/dir/id; read toml + md; `Table` reassembles the
readable whole (today's behaviour for spec/backlog/memory; new for slice/adr),
`Json` emits toml-as-data + md body.

`memory new` is `memory record` with `#[command(alias = "record")]` — identical
handler, two surface names.

### 5.5 Invariants, Assumptions & Edge Cases

- **Header suppressed on empty.** Rows present → header row; zero rows → empty
  string (preserves backlog virgin-repo → `""`; extends header to adr/backlog/
  memory).
- **Terminal-hide override.** `--all` OR any explicit `--status` disables the
  terminal-hide default (generalises backlog's existing rule to every kind).
- **`--json` vs `--format`.** `--json` is exactly `--format json`; if both given
  and consistent, fine; the coercion lives once in `Filter::from_args`.
- **Invalid regex** → a clean `anyhow` error, not a panic.
- **Multi-status semantics** — a row matches if its status ∈ the given set (OR
  within `--status`); `--tag` is OR within tags; the axes AND across each other.
- **slice divergence coupling** — extending `is_terminal_status` to include
  `superseded` also changes `is_divergent` (a superseded slice with incomplete
  phases no longer false-flags `⚠`). This is intended; existing divergence tests
  are reviewed and updated to assert the corrected behaviour.
- **backlog positional** — `[SUBSTR]` is retained as a deprecated alias: when
  `--filter` is absent and the positional present, the positional feeds `substr`;
  `--filter` is canonical. No break.
- **Memory exception** — `canonical_id` is not applied to memory; uid is its id.

## 6. Open Questions & Unknowns

- OQ-1: exact JSON field names per kind (e.g. slice `phases` as a nested
  `{completed,total,blocked}` object vs a rendered string). Resolve at build:
  faithful structured values, not the table's rendered cell strings.
- OQ-2: whether `show --format json` for slice/adr should include relationships
  (the `[relationships]` table) — lean yes (toml-as-data is faithful), confirm
  against the existing spec `show` json expectation when built.
- OQ-3 (deferred, not blocking): whether `-s`/`-t`/`-f`/`-r`/`-i` short flags
  collide with any kind-specific short flag (e.g. memory). Audited at build; a
  collision demotes the kind-specific flag to long-only, not the shared one.

## 7. Decisions, Rationale & Alternatives

- **D1 — C-hardened over a ListView trait (B), under the lifetime-churn metric.**
  The axis that grows unboundedly is kind-specific columns/flags; B unifies that
  (wrong) axis, making the shared render path a coupling magnet that churns on
  every per-kind display change and ossifies. B saves no flag churn vs C unless
  it *also* adopts flatten-args — so B = C + a forced god-type. C unifies the
  invariant axis (flags via flatten bundles; filter/id/json/format via policy
  fns) and keeps the variant axis local (zero shared-seam blast radius). C is
  also consistent with `entity.rs` D2. Alternative A (extend `meta.rs` only) was
  rejected: backlog/memory aren't `Meta`-shaped, so they'd stay bespoke — partial
  uniformity.
- **D2 — new `listing.rs` over widening `meta.rs`.** Cohesion: `meta.rs` is the
  numeric authored-toml reader; the kind-blind filter/format/render spine serves
  named (memory) and own-struct (backlog) kinds too — a distinct concern.
  `render_table` moves to `listing.rs` as generic layout.
- **D3 — id form prefixed everywhere (F-2), via `canonical_id(prefix,id)`.**
  Matches the citation convention + backlog; copy-paste-correct. Memory is
  conformant-by-exception (uid is canonical).
- **D4 — hide-terminal default + `--all`/explicit-`--status` reveal (F-3).**
  backlog's saner default, generalised. Per-kind terminal predicate.
- **D5 — `--filter/-f` flag over a positional substr (F-4).** Flags flatten
  cleanly into shared bundles; positionals do not. Matches spec-driver. backlog
  positional retained as deprecated alias.
- **D6 — `--format table|json` + `--json` sugar (F-6).** tsv deferred as a future
  `Format` arm (no contract change). JSON is a per-kind faithful mirror under a
  `{kind, rows}` envelope (D7).
- **D7 — per-kind faithful JSON over a normalized `extra`-bag envelope.** Agents
  get first-class typing on exactly the kind-specific data they want; an `extra`
  bag would bury it untyped.
- **D8 — `memory new` alias, keep `record` (F-7).** Uniform canonical verb, zero
  break; skills migrate at leisure.
- **D9 — pull the `regex` crate, include `--regexp/-r` + `-i` now.** The repo was
  deliberately regex-free (`memory.rs:125`); the dependency was decided
  explicitly (not slipped in). Small code; the spine was designed so it slots in
  as one more `CommonListArgs` field + one `retain` arm.

## 8. Risks & Mitigations

- R1 — **snapshot-test churn misread as regression.** Many `*_list` output tests
  change (prefixed ids, fewer rows, new headers). Mitigation: update them as part
  of the work with the *reason* in the commit; keep engine suites untouched as
  the behaviour-preservation proof.
- R2 — **slice divergence behaviour shift** (R4/§5.5). Mitigation: explicit
  test update asserting superseded no longer false-flags `⚠`; call it out in the
  audit.
- R3 — **`regex` dependency weight / lean-deps culture.** Mitigation: scoped to
  the filter arm; bounded blast radius; decided explicitly (D9).
- R4 — **short-flag collisions** (`-s -t -f -r -i -a`) with kind-specific flags.
  Mitigation: OQ-3 build-time audit; shared flag wins, kind-specific demotes to
  long-only.
- R5 — **C-indiscipline regression** — a future kind adds bespoke list flags
  instead of flattening the bundle. Mitigation: the bundle + format dispatch are
  the mandatory spine (Principle 2); a conformance test asserts each `list`
  variant flattens `CommonListArgs`.

## 9. Quality Engineering & Validation

- **Behaviour-preservation gate**: `entity.rs`, `registry.rs`, `meta.rs` reader
  suites green unchanged.
- **New unit tests** (pure, in `listing.rs`): `retain` matrix (substr ×
  multi-status × tag × terminal × all × regex), `canonical_id`, `json_envelope`
  shape, `Format`/`--json` coercion, invalid-regex error, empty→header-suppressed.
- **Per-kind tests**: prefixed-id rows; terminal-hide default + reveal; faithful
  JSON per kind; `slice show`/`adr show` (table + json); `memory new` ≡
  `memory record`; backlog `--filter` ≡ positional.
- **Conformance test**: every `list` subcommand parses the shared flags (R5).
- **Lint/format gate**: `just check` (plain `cargo clippy`, zero warnings),
  house string-assembly + no-`as` + BTree + `#[expect]` styles.
- **Closure intent** (from `slice-025.md`): `show` resolves for all five kinds;
  all `list`/`show` emit canonical prefixed ids; default `list` hides terminal
  with `--all`/`--status` revealing; shared filter base + shared renderer (human +
  JSON) back every kind; create-verb reconciled.

## 10. Review Notes

(Adversarial pass findings recorded here.)
