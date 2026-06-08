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

## PHASE-02 — adr migration + CommonListArgs + show seam (commit 165e576)

First kind on the spine; establishes the two seams every later phase reuses.

### The reusable command-side seam (`src/main.rs`)
- `CommonListArgs` (`#[derive(clap::Args)]`, `pub(crate)`, main.rs ~46) — the
  mandatory shared list-flag bundle. Fields: `filter: Option<String>` (-f),
  `regexp: Option<String>` (-r), `case_insensitive: bool` (-i), `status:
  Vec<String>` (-s, `value_delimiter=','`), `tag: Vec<String>` (-t), `all: bool`
  (-a), `format: Format` (`value_parser = Format::from_str`, `default_value_t`),
  `json: bool`. `--format` uses `value_parser`/`FromStr` (NOT `ValueEnum`) so clap
  stays out of the leaf (A-3). Needs `use std::str::FromStr;` in main.rs.
- `CommonListArgs::into_list_args(self) -> listing::ListArgs` — the lowering seam
  (clap → clap-free leaf). PHASE-03+: `#[command(flatten)] list: CommonListArgs`
  in the kind's `List` variant; dispatch calls `kind::run_list(path,
  list.into_list_args())`.
- Show wiring pattern (no shared struct — show args are small): a `Show { reference:
  String, format: Format (value_parser), json: bool, path }` variant; dispatch
  resolves `if json { Format::Json } else { format }` and calls
  `kind::run_show(path, &reference, fmt)`.

### The per-kind list_rows recipe (model: `src/adr.rs:list_rows`)
`fn list_rows(root, args: ListArgs) -> Result<String>`:
1. `validate_statuses(&args.status, KIND_STATUSES)?` (A-2) — every kind supplies a
   `&[&str]` known-set. adr's lives in `ADR_STATUSES`, lockstep-guarded against the
   `AdrStatus` variants by a drift-canary test.
2. `let (filter, format) = listing::build(args)?;`
3. read rows (existing reader: `meta::read_metas` for numeric kinds).
4. `let mut rows = listing::retain(rows, &filter, is_hidden, key);` — `is_hidden` is
   the kind's LIST hide-set fn; `key: &Row -> listing::FilterFields`.
5. **kind sorts** (`rows.sort_by_key(|m| m.id)` for adr — ordering is per-kind).
6. branch on `format`: Table → assemble grid (header row + `canonical_id` ids) →
   `listing::render_table`; Json → faithful row structs → `listing::json_envelope`.
- **Empty-list contract (§5.5)**: the table grid includes a header row, so guard
  `if rows.is_empty() { return String::new(); }` BEFORE calling render_table — else
  a no-row list prints a bare header. adr does this in its `render_table` helper.
- adr's tag axis is sourced from `[relationships].tags` (faithful, read-only — adr
  has no tag write verb); projected in `key`.

### The show recipe (model: `src/adr.rs:run_show`/`read_adr`/`format_show`/`show_json`)
- `parse_ref(&str) -> Result<u32>`: strips `ADR-`/`adr-` prefix (optional,
  case-spec'd) then parses; bare/padded ids work.
- `read_adr(adr_root, id) -> (AdrDoc, String)`: a fuller deserialize struct
  (`AdrDoc`: the four list fields + created/updated + a `Relationships` substruct
  with `#[serde(default)]` on every axis) read as data, plus the `.md` body verbatim.
- Table = `format_show`: `Vec<String>` parts + `concat()` (house lint: no
  `push_str(&format!)`), identity line, flat fields, non-empty relationship axes,
  then the prose body. Json = `show_json`: `serde_json::json!({kind, adr: doc,
  body})` pretty-printed (OQ-2: relationships included — toml-as-data is faithful).

### Boot consumer (declared changing consumer, R6 — `src/boot.rs:124`)
- Boot builds `listing::ListArgs { status: vec!["accepted".into()], ..default }`
  directly (no clap) — itself the proof the leaf is clap-free. The explicit status
  also reveals accepted past the hide-set, which is the boot intent.
- Boot snapshot test (`regenerate_projects_accepted_adrs_and_memory_pointers`)
  updated: asserts `ADR-001  accepted` (prefixed) + a header line (padding-agnostic:
  `lines().any(|l| l.starts_with("id") && l.contains("status"))`).

### meta.rs narrowing (EX-4)
- `meta::format_list` DELETED (adr was its last caller) + its two tests. The
  numeric-kind grid now lives per-kind on the spine. `meta::sort_and_filter` KEPT
  (slice.rs:417, spec.rs:916 still call it) — the surviving sort-by-id helper; its
  status-filter half is now dead-but-harmless until those kinds migrate (PHASE-03/04).

### Decisions / gotchas (durable)
- adr `--status` went single (`Option<String>`) → multi (`Vec<String>` via `-s`),
  the uniform surface. One value still works; the known-set = `AdrStatus` variants.
- listing.rs's self-clearing `#![expect(dead_code)]` RETIRED (build + canonical_id
  consumed). No per-symbol expect was needed — every spine symbol now has a caller
  via adr. PHASE-03+ should NOT need to reintroduce one.
- `Format::from_str` as a `value_parser` needs `std::str::FromStr` in scope at the
  call site (main.rs imports it).
- doc-comment lint: a line wrapping onto `{a, b, c}` trips
  `clippy::doc_lazy_continuation` (reads as a list item). Keep brace-sets on one line
  or rephrase.

### Gate
- 593 bin unit tests pass (was 581: +14 adr list/show/validate, −2 meta format_list).
  `just check` clean (clippy zero warnings, fmt). e2e suites green. Behaviour-
  preservation suites (entity/registry/meta readers/is_divergent) green **unchanged**.
- Manual CLI smoke confirmed: list (default/-f/--json), show (table), -s bogus error.

## PHASE-03 — slice list/show on the spine + status vocab (commits 0817896, 547eb76)

Second kind on the spine. Adds slice's variant axis (phase rollup + two markers)
and the vocabulary-drift mechanism.

### slice list (`src/slice.rs`)
- Same `list_rows(root, ListArgs)` recipe as adr: validate_statuses(SLICE_STATUSES)
  → listing::build → read_metas → retain(metas, &filter, is_hidden, key) → sort_by
  id → join phase rollup → branch Table/Json. **Rollup join is AFTER retain** — the
  filter projects `Meta` alone (the impure `state::phase_rollup` read runs only for
  the survivors). adr has no such join; this is slice's variant axis.
- Hide-set `is_hidden`: `{done, abandoned}` (terminal presentation). DISTINCT from
  `is_terminal_status` `{done}` (divergence). DISTINCT from the vocab. Three sets,
  never conflated — see the doc-comments tying them together.
- JSON `SliceRow.phases` is STRUCTURED: `{completed, total, blocked}` or `null`
  (untracked). NOT the rendered `4/6 !1 ?1` cell (OQ-1). The `?`/`⚠` markers are
  table-display-only — absent from JSON. **PHASE-06 conformance contract.**
- Table grid (`render_table`, renamed from `format_slice_rows`): header + per row
  `canonical_id`, `decorated_status`, `phases_cell`, slug, title. Empty → "".

### The two markers (independent predicates, same column)
- `is_drifted(status)` = `!SLICE_STATUSES.contains(status)` → trailing `?`.
  §5.5 vocabulary-drift invariant: an out-of-vocab STORED status is never hidden
  (hide-set lists known terminals only) and is flagged `?`.
- `is_divergent(status, rollup)` (UNCHANGED) → trailing ` ⚠`.
- `decorated_status(status, rollup)` composes both: `{status}{?}{ ⚠}` (drift hugs
  the token, divergence trails). Both can appear: `bogus? ⚠`. Computed in ONE place.

### slice show (A-5 — metadata + scope ONLY)
- The adr quartet, ported: `parse_ref` (SL-/sl-/bare), `SliceDoc` (Meta fields +
  created/updated + `Relationships{specs,requirements,supersedes}` all
  `#[serde(default)]`), `read_slice` (toml-as-data + slice-NNN.md body ONLY),
  `format_show`/`show_json`. JSON envelope key is `slice` (cf adr's `adr`).
- **NEVER folds design.md/plan.*/notes.md** — proved by
  `show_does_not_fold_in_design_plan_or_notes` (writes sibling files with secret
  markers, asserts neither table nor json leaks them). The reassembly opens only
  `slice-NNN.{toml,md}`.

### Vocabulary (D10) — `SLICE_STATUSES`
- `&["proposed","ready","started","audit","done","abandoned"]`. Slice has NO status
  enum → this `&[&str]` is the sole authority (no drift-canary against variants;
  instead `slice_statuses_matches_the_spec_vocabulary` pins it to slices-spec).
  Guards `--status` READ input only (writes deferred to the lifecycle verb).
- slices-spec.md amended: added `abandoned` (with a definition: terminal-but-not-
  delivered, distinct from done), updated the lifecycle set + the "unenforced" note
  (read-filter is now enforced; writes stay manual). `superseded` was never in the
  slice vocab — it was the ADR value mistakenly stored on SL-002.

### Data migration (C-3)
- SL-002 `superseded → abandoned` on `.doctrine/slice/002/slice-002.toml` ONLY
  (hand edit, single field; the historical `# superseded by SL-003` comment kept as
  prose history). The `002-entity-engine` symlink alias untouched (it is an alias,
  not a 2nd entity).

### main.rs wiring
- `SliceCommand::List` flattens `CommonListArgs` (bespoke `--status: Option<String>`
  dropped); dispatch `slice::run_list(path, list.into_list_args())`.
- `SliceCommand::Show { reference, format, json, path }` added, mirroring adr.

### meta narrowing
- slice STOPPED calling `meta::sort_and_filter` (it sorts via `sort_by_key(id)`).
  The fn STAYS — spec.rs:916 still calls it (PHASE-04 migrates spec). Its
  status-filter half is dead-but-harmless until then.

### Gate
- 614 bin unit tests pass (was 593: +23 slice list/show/vocab/drift/decorated,
  −2 old format_slice_rows tests renamed/expanded). `just check` clean (clippy zero
  warnings, fmt). e2e suites green. Behaviour-preservation (entity/registry/meta
  readers/is_divergent/is_terminal_status) green **UNCHANGED**.
- Manual CLI smoke: default list hides SL-002, `--status abandoned` reveals it as
  `abandoned`, `-s bogus` errors with the vocab list, `show SL-002` table +
  `show 25 --json` envelope all correct; SL-025 shows `2/6` (rollup join works).

## PHASE-04 — spec + backlog on the spine (commits 95e8bba, a37cf36, 1b99b5b)

The last two show-having kinds. spec adds the per-subtype block axis + the
single-envelope-with-subtype JSON shape (A-8); backlog is the reference shape +
the deprecated positional alias (A-7).

### spec list (`src/spec.rs`)
- `list_rows(root, ListArgs)` on the spine, BUT the per-subtype grouping makes it
  unlike adr/slice: it calls `subtype_rows(root, subtype, &filter)` ONCE PER
  SUBTYPE (product, then tech) with the SAME `Filter` (one `build`). Each
  `subtype_rows` = read_metas → `retain(metas,&filter,is_hidden,key)` → sort_by id
  → join `member_count` (the variant-axis join, AFTER retain — the slice rollup
  precedent). `key(subtype, m)` prefixes the id with the subtype's `Kind.prefix`
  (PRD/SPEC) — load-bearing so PRD-001 and SPEC-001 never collide in the shared
  envelope.
- **Table** = two labelled blocks (`format_spec_rows`: `subtype.label()\n` + grid),
  prefixed ids, empty block suppressed, whole-empty → "".
- **Json = ONE `{kind:"spec", rows:[...]}` envelope** spanning BOTH subtypes
  (A-8 — NOT two envelopes). `SpecRow{id (prefixed), subtype (&'static str
  "product"/"tech"), status, slug, members (usize COUNT, structured not rendered)}`.
  **PHASE-06 conformance contract.**
- Hide-set `is_hidden` = `{superseded}`. `SPEC_STATUSES` =
  `{draft,active,deprecated,superseded}` (the SpecStatus variants) + drift-canary
  `spec_statuses_matches_the_variants`. spec has a CLOSED enum → a stored status is
  always in-vocab → NO `?` drift marker (slice-only).

### spec show --json (`src/spec.rs`)
- `run_show` gained `format: Format`; `show_json` added. Faithful: serializes the
  `Spec` (added `Serialize` to `Spec`/`Source`/`Member`/`Interaction`; SpecStatus/
  SpecSubtype/C4Level already had it) + `id` (prefixed) + `body` verbatim +
  `members` (each `{label, order, requirement:{id,slug,title,kind,status}}` — the
  requirement projected by hand via `serde_json::json!` because `Requirement` stays
  Deserialize-only) + `interactions`. EX-2 boundary preserved (no cross-corpus scan).

### backlog list (`src/backlog.rs`)
- `list_rows(root, kind: Option<ItemKind>, args: ListArgs)` — `--kind` is the ONE
  kind-specific axis (kept beside the flatten, applied via `items.retain(...)`
  AFTER the shared `retain`). read_all → `retain(items,&filter,is_hidden,key)` →
  kind-filter → sort `(kind.ordinal, id)` → Table/Json.
- `key(i)` = prefixed `i.kind.canonical_id(i.id)` (already prefixed pre-SL-025) +
  slug/title + `status.as_str()` + `i.tags` (backlog has real tags, unlike spec).
- **Hide-set REUSES `Status::is_terminal`** via a stringly bridge
  `is_hidden(status:&str) = parse_enum::<Status>(status).is_ok_and(Status::is_terminal)`
  — `{resolved,closed}`, NO new predicate (design §5.3). `BACKLOG_STATUSES` =
  the 5 Status variants + drift-canary.
- **Json** = `{kind:"backlog", rows:[...]}`, `BacklogRow{id (prefixed), kind, status,
  resolution (Option, null when absent), slug, title}` — flat (facet/relationships
  ride show, not list). **PHASE-06 conformance contract.**
- `select`/`ListFilter` DELETED (replaced by the spine + the inline kind-filter).

### backlog show --json (`src/backlog.rs`)
- `run_show` gained `format`; `show_json` projects the full item by hand
  (`serde_json::json!`) — flat identity + resolution + the risk `[facet]` (risk
  only, null otherwise) + outbound relationships. BacklogItem fields are private &
  its substructs aren't Serialize, so hand-projection (not a derive).

### A-7 — the deprecated positional (`src/main.rs` dispatch)
- `BacklogCommand::List` flattens `CommonListArgs`, KEEPS the positional
  `substr: Option<String>`, keeps `--kind` + `-p`. The **precedence lives in the
  dispatch** (not list_rows): `if list.filter.is_none() { list.filter = substr; }`
  — `--filter` WINS, the positional folds in only when --filter is absent. Proven
  by `tests/e2e_backlog_filter_alias.rs` (binary-level — the fold is unreachable
  from a unit test).

### main.rs wiring
- `SpecCommand::List` flattens CommonListArgs (dropped bespoke `--status:
  Option<String>`); `SpecCommand::Show` gained `format`/`json`. backlog ditto +
  the positional fold. Dispatch folds `json → Format::Json` for both shows.

### Dead-code removal (the planned §5.1 narrowing, completed)
- **`meta::sort_and_filter` REMOVED.** spec.rs:916 was its last NON-TEST caller; once
  spec migrated, `cargo build` (bins/lib) flagged it dead (repo denies dead_code).
  The adr.rs callers at 594/602/925/938 were all `#[cfg(test)]` — they used it as a
  sort/filter helper. Repointed: those two adr tests now assert via the real
  `list_rows` path (the production surface) + a local `sort_by_key` for the
  read_metas round-trip. The meta.rs `sort_and_filter_orders_by_id_and_filters_status`
  test was deleted with the fn. NO numeric kind calls it now → the "surviving sort
  half" rationale in design §5.3 is moot (all kinds sort via `sort_by_key`).

### Gate
- 625 bin unit tests pass (was 614: +6 spec list/json/hide/regex/canary/show-json,
  +6 backlog regex/json/canary/is_hidden/show-json, −1 meta sort_and_filter test).
  +1 e2e (`e2e_backlog_filter_alias`, A-7). `just check` clean (clippy zero
  warnings, fmt). All e2e suites green. Behaviour-preservation
  (entity/registry/meta readers/is_divergent/is_terminal_status) green **UNCHANGED**.
- Manual CLI smoke: spec list (two labelled blocks, prefixed ids, #members), spec
  list --json (ONE envelope, subtype per row), -s bogus (uniform error); backlog
  list (prefixed, ordinal+id sort), positional `auth` filters, `auth --filter
  token` → token wins (A-7), backlog show --json faithful.

## PHASE-05 — memory + boot consumer (commit 7283121)

The LAST kind on the spine — the exception kind — plus boot's last section.

### memory list (`src/memory.rs`)
- `list_rows(root, type_f: Option<MemoryType>, args: ListArgs)` on the spine:
  validate_statuses(MEMORY_STATUSES) → listing::build → collect_all → retain(rows,
  &filter, is_hidden, key) → `--type` retain → `sort_default` → Table/Json.
- **THE uid exception (§5.3/§5.5)**: `key(m).canonical = m.uid` directly — NOT
  `canonical_id`. The regex domain is the full `mem_<32hex>`. memory has no slug, so
  `key` plays the slug role in the substr/regex domains. JSON `uid` is NOT prefixed.
- **Hide-set** `{superseded, retracted, archived, quarantined}` via
  `is_hidden(status:&str) = Status::parse(s).is_ok_and(Status::is_hidden)`. active +
  draft VISIBLE (the SL-005 list showed ALL six — this is the BEHAVIOUR CHANGE).
  `--all`/explicit `--status` reveals (the uniform rule, in `retain`).
- **Re-gridded onto `render_table`** (handover-recommended, §5.2 shared-renderer
  closure intent). The private `format_list` width-printer DELETED. Columns
  `uid type status trust key title` (EX-1: trust is the new column vs SL-005's
  uid/type/status/key/title). Header row 0; empty → "". **F-A10 scrub preserved**:
  `key` + `title` cells `scrub_line`d (newline-injection guard) — the re-grid kept it.
- `MEMORY_STATUSES` = the 6 Status variants + drift-canary
  `memory_statuses_matches_the_variants` (adr/spec/backlog precedent). Closed enum →
  NO `?` drift marker (slice-only).
- **`--type` is the one kind-specific axis** (kept beside the flatten — backlog
  `--kind` precedent), applied via `rows.retain` AFTER the shared retain.
- **JSON** = `{kind:"memory", rows:[{uid, type, status, trust, key|null, title}]}`.
  `MemoryRow` serde struct (`#[serde(rename="type")]`); `trust`/`title` scrubbed.
  **PHASE-06 conformance contract.**

### select_rows KEPT (retrieve dep) + sort extracted
- `retrieve.rs:672` still calls `select_rows(collect_all, type_f, status_f, None)`
  for its typed filter + sort — NOT changed. Extracted the created-desc+uid
  comparator into `sort_default(&mut [Memory])`, shared by both `select_rows` and
  `list_rows` (DRY — one comparator, the §5.2 ordering contract).

### memory show --json (`src/memory.rs`)
- `run_show` gained `format: Format`; dispatch folds `json → Format::Json`.
  `show_json(m, body)` hand-projects (private fields, closed enums via `as_str`) the
  full entity under `{kind:"memory", memory:{uid,key,type,status,title,summary,
  created,updated,scope{...},anchor{kind,commit,checkout_state_id,ref,verified_sha},
  verification_state,reviewed,review_by,trust_level,severity,weight}, body}`. Table
  path UNCHANGED (the SL-005 nonce-framed `render_show`; nonce only minted for Table).
  **PHASE-06 conformance contract.**

### memory new alias (`src/main.rs`)
- `#[command(visible_alias = "new")]` on `MemoryCommand::Record` — two surface
  names, ONE handler (`run_record`), identical entity. `memory record` keeps working.
  Proven by `tests/e2e_memory_new_alias.rs` (the alias is unreachable from a unit
  test — backlog A-7 precedent): both verbs, identical args → same normalised toml.

### boot active-only (`src/boot.rs`, C-4)
- boot's memory section calls `list_rows(root, None, ListArgs{ status:
  vec!["active"], ..default })` — an EXPLICIT active-only predicate, DECOUPLED from
  the CLI list default (which keeps draft). boot is an agent-context PRODUCER; draft
  is unreviewed → must not leak. Mirrors the ADR section's `status:["accepted"]`.
- Boot snapshot test `regenerate_projects_accepted_adrs_and_memory_pointers`
  unchanged (its memory is active). NEW test
  `boot_memory_section_is_active_only_decoupled_from_the_list_default`: a fixture
  spanning active + draft + each of the 4 hidden states → asserts boot shows
  active-ONLY, while `list_rows` default hides the 4 terminal but KEEPS draft (VT-3 —
  the two distinct visibility rules).

### main.rs wiring
- `MemoryCommand::List` flattens `CommonListArgs` (dropped bespoke single
  `--status`/`--tag`; KEEPS `--type`); dispatch `run_list(path, memory_type,
  list.into_list_args())`. `Show` gained `format`/`json`. `Record` gained the
  visible_alias.
- **OQ-3 short-flag check**: memory's `--type`/`--tag` were LONG-ONLY pre-SL-025, so
  NO collision with the shared `-t` (tag) or any of `-f/-r/-i/-s/-a`. `-p` stays the
  per-variant root locator. The shared `-t` is now tag (Vec, OR); the old memory
  `--tag` was single `Option<String>`. No demotion needed for memory.

### Gate
- 634 bin unit tests pass (was 625: +9 — drift-canary, is_hidden, key-uid-exception,
  hide default+reveal+all, bad-status reject, list json, --type filter, show_json;
  the 3 format_list tests migrated to format_rows; the 4 list_rows tests retargeted
  to the new signature). +1 e2e (`e2e_memory_new_alias`). `just check` clean (plain
  clippy zero warnings, fmt). All e2e green. Behaviour-preservation
  (entity 24 / registry 25 / slice::tests 44 incl is_divergent + is_terminal_status)
  green **UNCHANGED** — memory list default change is a DECLARED consumer change, not
  the engine gate.
- Manual CLI smoke: memory new (alias), list (type/trust cols + header, draft
  visible), list --json (uid not prefixed), -s bogus (six-status error), show --json
  (faithful + body), --type fact, -f substr on title.
