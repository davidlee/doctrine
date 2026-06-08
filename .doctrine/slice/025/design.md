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

Status vocabularies. Two distinct predicates ride these (§5.3): the slice
*divergence-terminal* set (`is_terminal_status`, `{done}` — **unchanged**) feeding
`is_divergent`, and the NEW per-kind *list hide-set* feeding `retain`. Inventories
(hide-set in brackets):

- `slice` — free string today; SL-025 amends `slices-spec.md` to the enforced set
  `{proposed, ready, started, audit, done, abandoned}` (`abandoned` replaces the
  out-of-spec `superseded` now on disk; 2 slices migrate). hide-set `{done, abandoned}`.
- `adr` — proposed/accepted/rejected/superseded/deprecated. hide-set `{rejected, superseded, deprecated}`.
- `spec` — draft/active/superseded. hide-set `{superseded}`.
- `memory` — active/draft/superseded/retracted/archived/quarantined (**SIX**). hide-set `{superseded, retracted, archived, quarantined}`.
- `backlog` — open/triaged/started/resolved/closed. hide-set `{resolved, closed}` (unchanged; `Status::is_terminal` already is this set).

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

Two new/changed seams, split by concern (C4-ish component view). Per ADR-001
(leaf ← engine ← command), the clap-facing arg bundle lives **command-side**; the
spine leaf is **clap-free** (A-3):

```
command layer (main.rs + per-kind run_list/run_show)
   CommonListArgs  (#[derive(clap::Args)] — flattened into every list variant)
        │  maps parsed args → listing::Filter + listing::Format
        │  supplies per-kind is_terminal + columns
        ▼
listing.rs  (NEW, pure leaf — NO clap)  ── the kind-blind read spine
   Filter, Format (plain enum + FromStr), retain<…>, render dispatch,
   json_envelope, canonical_id, validate_statuses, render_table (moved from meta)
        │
meta.rs  (kept, narrowed)  ── numeric authored-toml reader
   Meta, read_meta(s), sort_and_filter
        │
entity.rs (unchanged)  ── Kind { dir, prefix, scaffold }; prefix feeds canonical_id
```

`CommonListArgs` is one `#[derive(clap::Args)]` struct defined once in the command
layer and flattened into each kind's `list` variant — DRY at the flag surface
while keeping clap out of the leaf. `render_table` moves from `meta.rs` to
`listing.rs` (generic layout, used by every kind). `meta.rs` keeps exactly its
numeric-reader charter.

**`boot.rs` is a declared non-clap consumer.** The governance snapshot renders its
ADR and Memory sections via `adr::list_rows`/`memory::list_rows` (`boot.rs:124,127`).
Boot builds a `listing::Filter` directly from plain values — no clap — which is
itself the proof the leaf is genuinely clap-free (A-3). Boot adopts the new surface:
prefixed `ADR-` ids + header in the ADR section, the hide-terminal default in the
Memory section. Boot's snapshot tests change with the format (R6); it is listed in
the §9 gate as a *changing* consumer, not part of the unchanged engine gate.

**Dead-code removal on migration:** `meta::format_list` (last caller: adr) and the
filter half of `meta::sort_and_filter` (superseded by `retain`) are removed — repo
clippy denies `dead_code`. The sort-by-id `sort_and_filter` also provided is kept as
a small `meta`-side sort the numeric kinds call (§5.3 ordering).

### 5.2 Interfaces & Contracts

**Shared input contract** — one composable bundle, defined command-side
(`main.rs`), flattened into every `list` variant (illustrative; field-level clap
attrs elided):

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
    /// Output format. `value_parser` over `Format::from_str` — NOT `value_enum`,
    /// which would require `Format: clap::ValueEnum` and drag clap into the leaf (A-3).
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
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

**Shared filter contract** — pure, in `listing.rs` (clap-free). The command layer
builds a `Filter` from `CommonListArgs`; the leaf never sees clap:

```rust
pub(crate) struct Filter {
    pub substr: Option<String>,           // lowercased once
    pub regex: Option<regex_lite::Regex>, // pre-compiled (case flag baked in)
    pub status: Vec<String>,              // empty = no status constraint
    pub tags: Vec<String>,                // empty = no tag constraint
    pub all: bool,
}

/// Build a Filter + resolved Format from plain values (command layer passes the
/// parsed CommonListArgs fields in). Compiles the regex (clean error on a bad
/// pattern). `--json` forces Json regardless of `--format` (A-9 precedence).
pub(crate) fn build(/* substr, regexp, case_insensitive, status, tags, all,
                       format, json */) -> anyhow::Result<(Filter, Format)>;

/// Each kind projects a row to its filterable fields ONCE — one closure, not
/// five (A-1). `canonical` is the regex domain's leading field (uid for memory);
/// substr matches slug+title, regex matches canonical+slug+title — distinct
/// domains, both derivable from this projection.
pub(crate) struct FilterFields {
    pub canonical: String,   // SL-025 / ADR-001 / mem_… (regex domain)
    pub slug: String,
    pub title: String,
    pub status: String,
    pub tags: Vec<String>,
}

/// Keep a row iff: not (hidden by the kind's hide-set) AND substr-match
/// (slug+title) AND regex-match (canonical+slug+title) AND status-match AND
/// tag-match. Hide is suppressed when `all` OR any explicit `status` is given.
/// `key` projects each row; `is_hidden` is the kind's LIST hide-set predicate —
/// distinct from any divergence/lifecycle-terminal predicate (§5.3).
/// FILTER-ONLY: ordering is the caller's (per-kind) concern (§5.3).
pub(crate) fn retain<R>(
    rows: Vec<R>,
    f: &Filter,
    is_hidden: impl Fn(&str) -> bool,   // the kind's LIST hide-set, not divergence-terminal
    key: impl Fn(&R) -> FilterFields,
) -> Vec<R>;

/// Validate a stringly `--status` set against a kind's known statuses, with one
/// uniform error message (A-2 — recovers the correctness that the shared
/// Vec<String> bundle loses vs a typed clap enum; tab-completion is the only
/// residual cost, accepted). EVERY kind supplies a known-set: the enum kinds from
/// their variants, slice from the newly-enforced `slices-spec.md` vocabulary
/// `{proposed,ready,started,audit,done,abandoned}` (D10). This validates READ
/// (filter) input only — not stored-status writes/transitions.
pub(crate) fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()>;
```

One projection closure (not five) keeps the variant row types — `Meta`,
`BacklogItem`, memory's row — free of a shared supertype while serving both
match domains; it is filter-only, never render.

**Shared render contract** — pure, clap-free (`Format` is a plain enum + `FromStr`
so the command layer can wire `#[arg(value_parser)]` without clap in the leaf):

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format { Table, Json }   // + impl FromStr (table|json)

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
- **Two terminal predicates, never conflated.** (a) *divergence-terminal* —
  `slice::is_terminal_status` stays `{done}`, **untouched**; it feeds `is_divergent`
  only (an `abandoned` slice with incomplete phases must NOT false-flag `⚠`).
  (b) *list hide-set* — a NEW per-kind presentation predicate consumed only by
  `retain`: slice `{done, abandoned}`, adr `{rejected, superseded, deprecated}`,
  spec `{superseded}`, memory `{superseded, retracted, archived, quarantined}`,
  backlog `{resolved, closed}`. Memory active/draft stay visible. backlog's existing
  `Status::is_terminal` already IS its hide-set (reused, no new code).
- **Ordering is per-kind (variant axis), not in `retain`.** `retain` filters only;
  each kind orders its rows for render — slice/adr/spec by id, backlog by
  `(kind.ordinal, id)`, memory by `created` desc then uid. The sort-by-id half of
  `meta::sort_and_filter` survives as a thin `meta` sort the numeric kinds call; its
  status-filter half is removed (superseded by `retain`).
- **Slice status vocabulary (amended authority — D10).** `slices-spec.md`'s set
  becomes `{proposed, ready, started, audit, done, abandoned}`, enforced as the
  `validate_statuses` known-set for `slice list --status`. Write-time/transition
  enforcement stays deferred (lifecycle verb). The 2 live `superseded` slices
  migrate to `abandoned` as part of this slice.
- The id prefix is owned by `entity::Kind.prefix` (already present) and consumed
  by `canonical_id`. Memory is the exception: its uid *is* its canonical id, so
  it does not route through `canonical_id`.
- **`slice show` reassembly boundary (A-5)**: `slice show` reassembles the slice's
  *metadata + scope* (`slice-NNN.toml` as data + `slice-NNN.md` body) only — NOT
  `design.md`/`plan.*`/`notes.md`, which are distinct artifacts with their own
  (future) surfaces. Mirrors how `adr show` reassembles `adr-NNN.{toml,md}`.
- **spec JSON across subtypes (A-8)**: the table groups product/tech into labelled
  blocks; JSON emits a single `{kind:"spec", rows:[…]}` envelope where each row
  carries a `subtype` field (faithful), not two envelopes.

### 5.4 Lifecycle, Operations & Dynamics

`list` flow, every kind:
1. command layer parses `CommonListArgs` (flattened) + kind-specific flags.
2. `listing::build(...)` → `(Filter, Format)` (regex compiled, `--json` folded);
   `validate_statuses` against the kind's known set.
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
- **`--json` vs `--format` (A-9).** `--json` *forces* `Json` and wins over any
  `--format` value (so `--json --format table` → Json, no error); the precedence
  lives once in `listing::build`.
- **Invalid regex** → a clean `anyhow` error, not a panic.
- **Multi-status semantics** — a row matches if its status ∈ the given set (OR
  within `--status`); `--tag` is OR within tags; the axes AND across each other.
- **slice divergence is untouched** (corrects an inverted claim in the earlier
  draft). `is_terminal_status` stays `{done}`; the list hide-set is a *separate*
  predicate (§5.3). So `is_divergent` is unchanged and its tests stay green
  (behaviour-preservation, not a behaviour shift). Tracing the earlier draft's
  proposal — adding `superseded`/`abandoned` to `is_terminal_status` — would have
  *introduced* a false `⚠` on an abandoned-with-incomplete-phases slice
  (`terminal && completed<total`), the opposite of what it claimed. The split
  avoids it: an `abandoned` slice is hidden by default (hide-set) yet does not
  false-flag `⚠` (divergence-terminal excludes it).
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
  uniformity. *Churn caveat:* "a new shared flag → one edit" holds for a *full*-share
  flag in `CommonListArgs`; a flag shared by a *subset* costs one bundle-extraction
  touching its prior inline holders. C is still ≤ B here — B carries the identical
  subset cost *plus* a forced god-type.
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
- **D10 — amend `slices-spec.md` to `{proposed,ready,started,audit,done,abandoned}`,
  enforce as the `--status` filter known-set.** The spec already owned the set and
  marked it "v1: unenforced" (`slices-spec.md:92,219,235`); live disk had drifted to
  an out-of-spec `superseded` (2 slices). Uniform filter-validation needs a per-kind
  known-set, so slice must have one. `abandoned` (broader: superseded / dropped /
  obsoleted) replaces `superseded`; the 2 slices migrate. Enforcement is scoped to
  **read/filter input** (`validate_statuses`) — write-time/transition enforcement
  remains the deferred lifecycle verb's job, so the "no status-transition machinery"
  Non-Goal holds. Editing an evergreen spec + migrating live data are called out in
  the audit (R7).
- **D9 — pull `regex-lite`, include `--regexp/-r` + `-i` now.** The repo was
  deliberately regex-free (`memory.rs:125`); the dependency was decided
  explicitly (not slipped in). `regex-lite` (no `regex-automata`/`aho-corasick`)
  is chosen over full `regex` for compile time + binary size — id/slug/title
  matching needs nothing fancy. Small code; the spine slots it in as one
  `CommonListArgs` field + one `retain` arm.

## 8. Risks & Mitigations

- R1 — **snapshot-test churn misread as regression.** Many `*_list` output tests
  change (prefixed ids, fewer rows, new headers). Mitigation: update them as part
  of the work with the *reason* in the commit; keep engine suites untouched as
  the behaviour-preservation proof.
- R2 — **slice divergence (now a non-change).** An earlier draft coupled the list
  hide-set to `is_terminal_status` and mis-stated the effect. Resolved by splitting
  the predicates (§5.3/§5.5): `is_divergent` is untouched, its tests stay green. The
  split *is* the mitigation; no behaviour-shift risk remains.
- R3 — **regex dependency weight / lean-deps culture.** Mitigation: `regex-lite`
  (not full `regex`) — minimal compile/binary cost; scoped to the filter arm;
  decided explicitly (D9).
- R4 — **short-flag collisions** (`-s -t -f -r -i -a`) with kind-specific flags.
  Mitigation: OQ-3 build-time audit; shared flag wins, kind-specific demotes to
  long-only.
- R5 — **C-indiscipline regression** — a future kind adds bespoke list flags
  instead of flattening the bundle. Mitigation: the bundle + format dispatch are
  the mandatory spine (Principle 2); a **behavioural** conformance test (A-4 —
  clap exposes no structural "is-flattened" check) parses `<kind> list --filter x
  --json` for every kind and asserts success.
- R6 — **boot snapshot format churn.** The governance snapshot's ADR/Memory
  sections change (ADR-prefixed ids + header; memory hide-default). Mitigation: boot
  declared a consumer (§5.1, §9); its snapshot tests update with the reason in the
  commit; the heavier F-8 memory-section *trim* stays a separate follow-up.
- R7 — **spec amendment + live-data migration** (D10). Editing evergreen
  `slices-spec.md` and migrating 2 slices `superseded→abandoned`. Mitigation: both
  git-tracked + reviewed; the migration is a mechanical authored-toml edit; called
  out in the audit.

## 9. Quality Engineering & Validation

- **Behaviour-preservation gate**: `entity.rs`, `registry.rs`, `meta.rs` reader
  suites + `slice::is_divergent` tests green **unchanged**. `boot.rs` snapshot tests
  change *legitimately* (declared consumer, R6) — NOT part of the unchanged gate.
- **New unit tests** (pure, in `listing.rs`): `retain` matrix (substr ×
  multi-status × tag × hide-set × all × regex), `canonical_id`, `json_envelope`
  shape, `Format`/`--json` coercion, invalid-regex error, empty→header-suppressed.
- **Domain-distinction test (F-N9)**: a row whose `canonical` matches the regex but
  whose slug+title do NOT match the substr (and vice-versa) — proves the two match
  domains are wired independently (guards the A-1 regression).
- **Ordering-preservation test (F-N9)**: slice/adr/spec by id, backlog by
  `(kind.ordinal,id)`, memory `created`-desc+uid, asserted after `retain`.
- **Per-kind tests**: prefixed-id rows; hide-set default + reveal; faithful
  JSON per kind; `slice show`/`adr show` (table + json); `memory new` ≡
  `memory record`; backlog `--filter` ≡ positional; `slice list --status abandoned`
  accepted + `--status bogus` rejected (D10); 2 migrated slices read as `abandoned`.
- **Boot-consumer preservation test (F-N9/R6)**: boot renders via the new
  `list_rows`; assert the ADR section gains `ADR-` ids + header and the memory
  section applies the hide-default.
- **Conformance test**: every `list` subcommand parses the shared flags (R5).
- **Lint/format gate**: `just check` (plain `cargo clippy`, zero warnings),
  house string-assembly + no-`as` + BTree + `#[expect]` styles.
- **Closure intent** (from `slice-025.md`): `show` resolves for all five kinds;
  all `list`/`show` emit canonical prefixed ids; default `list` hides terminal
  with `--all`/`--status` revealing; shared filter base + shared renderer (human +
  JSON) back every kind; create-verb reconciled.

## 10. Review Notes

Internal adversarial pass (pre-`/plan`). All integrated into the body above.

- **A-1 (interface bug, fixed §5.2)** — a single `haystack` accessor cannot serve
  both substr (slug+title) and regex (canonical+slug+title) match domains, and
  five closures was a smell. Replaced with one `FilterFields` projection closure
  serving both domains.
- **A-2 (typed-status regression, fixed §5.2)** — the shared stringly
  `--status: Vec<String>` removes the clap enum validation backlog/memory have
  today. Added `validate_statuses(given, known)` for a uniform error; residual
  cost is lost shell tab-completion on status values (accepted).
- **A-3 (ADR-001 violation, fixed §5.1/§5.2)** — `clap::Args`/`ValueEnum` in the
  "pure leaf" `listing.rs` pulls a command-layer concern into a leaf.
  `CommonListArgs` now lives command-side; the leaf is clap-free (`Format` is a
  plain enum + `FromStr`; command wires `value_parser`).
- **A-4 (untestable claim, fixed §9/R5)** — "assert each variant flattens
  CommonListArgs" isn't structurally checkable; reworded to a behavioural
  parse-conformance test per kind.
- **A-5 (under-spec, fixed §5.3)** — `slice show` reassembles metadata + scope
  only, not design/plan/notes.
- **A-8 (under-spec, fixed §5.3)** — spec JSON is one envelope with a `subtype`
  field per row, not two envelopes.
- **A-9 (under-spec, fixed §5.5)** — `--json` forces Json and wins over
  `--format`; no conflict error.
- **A-7 (accepted, §5.5)** — `backlog list foo --filter bar` silently prefers
  `--filter`; documented precedence, not an error (low stakes).
- **OQ-3 / short-flag collisions** — `-s -t -f -r -i -a` audited at build against
  each kind's existing short flags; shared flag wins, kind-specific demotes to
  long-only.

### External adversarial pass (fresh session, pre-`/plan`) — integrated

Found NEW issues beyond A-1…A-9; all integrated above:

- **F-N1 (correctness, 🔴)** — `is_divergent` claim was *inverted* and the predicate
  conflated list-hide with divergence-terminal. Fixed: split `is_terminal_status`
  `{done}` (divergence, untouched) from the per-kind list hide-set (§5.3/§5.5; R2).
- **F-N2 (missed consumer, 🔴)** — `boot.rs` consumes `adr`/`memory::list_rows`.
  Fixed: boot declared a consumer, adopts the new surface (§5.1; §9; R6).
- **F-N3 (stale inventory, 🟠)** — memory has 6 statuses, not 4. Fixed (§2/§5.3);
  hide-set decided `{superseded, retracted, archived, quarantined}`.
- **F-N4 (under-spec, 🟠)** — ordering dropped from the contract. Fixed: `retain` is
  filter-only, ordering is a per-kind step (§5.2/§5.3).
- **F-N5 (missing authority, 🟠)** — slice had no enforceable vocabulary. Fixed:
  amend `slices-spec.md` (+`abandoned`, −`superseded`), enforce as the filter
  known-set, migrate 2 slices (D10; §5.3; R7).
- **F-N6 (🟡)** — `value_enum` would re-import clap to the leaf. Fixed:
  `value_parser`/`FromStr` (§5.2).
- **F-N7 (🟡)** — dead `meta::format_list` + `sort_and_filter` filter-half removed (§5.1).
- **F-N8 (🟡)** — D1 "one edit" softened for subset-share (§7).
- **F-N9 (🟡)** — added domain-distinction, ordering-preservation, and
  boot-consumer preservation tests (§9).
- *Non-finding noted:* OQ-3 short-flag collision risk is near-nil — every current
  `list` flag is long-only except `-p` (kept per-variant).

Residual open items carried to `/plan`: OQ-1 (per-kind JSON field shapes),
OQ-2 (`show --format json` relationships inclusion), OQ-3 (collision audit).
