# Notes SL-020: Backlog entity v1: work-intake items (one kind + item_kind facet)

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — model + scaffold (commit `5e069ec`)

Decisions worth surviving the phase sheet (audit-harvest candidates):

- **`dead_code` bridge is module-scoped + has a fulfillment trap.** `src/backlog.rs`
  is production-dead until the verbs land, so one `#![expect(dead_code, reason)]`
  rides the whole module (the `retrieve.rs` PHASE-01 precedent), not per-item attrs.
  Trap: under `cargo test` the tests make most items live → a module
  `expect(dead_code)` would be **unfulfilled** → `warnings = "deny"` turns that into
  a hard error. The inert `KIND_PRECEDENCE` const (referenced *nowhere*, including
  tests) stays dead in BOTH the lib and test builds and keeps the expectation
  fulfilled. **Retire the expect only when the last verb (PHASE-04/05) consumes the
  model and nothing is left dead** — and drop `KIND_PRECEDENCE`'s "keeps it
  fulfilled" rationale at the same time (the const itself stays as canon).
- **str→enum reuses the serde derive (single source).** `parse_enum` /
  `optional_enum` drive the closed enums through `serde::de::value::StrDeserializer`
  (the `IntoDeserializer` idiom). `as_str` is the *render* mirror only — no second
  hand-written string→variant table. Unknown tokens get serde's "unknown variant"
  message for free. The `"" → None` seam lives in `validate`, never a serde derive
  (`""` is no enum variant — a direct `Option<Resolution>` derive would reject it).
- **Two toml templates, a `{{kind}}` token (not a literal `kind` per template).**
  `backlog.toml` serves the four plain kinds; `backlog-risk.toml` adds `[facet]`.
  `render_backlog_toml` picks via `ItemKind::has_facet()` and substitutes
  `item_kind.as_str()`. Keeps each template literal (spec template-per-variant
  precedent) without a 5th near-duplicate.
- **Const `Kind.scaffold` is a non-capturing closure** `|c| backlog_scaffold(K, c)`
  (design §5.1) — coerces to the `fn` pointer in const context; five closures vs
  spec.rs's two named wrappers.
- **R6 gate held:** `git diff src/entity.rs` empty across PHASE-01 — the five
  backlog `Kind`s are pure `Fresh` callers, zero engine change. This is the load-
  bearing premise of the whole slice; keep it green every phase.

## PHASE-02 — `backlog new <kind>` + install wiring

Decisions worth surviving the phase sheet (audit-harvest candidates):

- **`new` is a pure mirror of `adr`/`spec` `run_new`.** `backlog::run_new(path,
  item_kind, title, slug)` — resolve title/slug, `clock::today()`, `materialise(
  item_kind.kind(), &LocalFs, …, &Fresh, …)`, print `Created XXX-NNN: <dir>` via
  `writeln!(io::stdout())` (NOT `println!` — the `print_stdout` clippy denial).
  Added `ItemKind::canonical_id(id)` (mirror `SpecSubtype::canonical_id`) for the
  print; it makes `prefix()` live in the lib build. CLI: a `Backlog` `Command`
  variant + a one-arm `BacklogCommand::New`. **R6 gate held** — `git diff
  src/entity.rs` still empty.
- **Authored-entity wiring trap closed, both surfaces** (`mem.pattern.install.
  authored-entity-wiring`): `.doctrine/backlog` → `install/manifest.toml`
  `[dirs].create`; `!.doctrine/backlog/` → the **repo** `.gitignore` (the dogfood
  blanket-`.doctrine/*` model). The memory's load-bearing nuance held exactly: the
  installer's *denylist* model (client repos, `[gitignore].entries`) takes NO
  negation — only this repo's blanket model does. Manifest seeds only the
  `.doctrine/backlog` **parent**; the five per-kind dirs are engine-created lazily
  on first `new` (PHASE-03 missing-dir tolerance). Negation takes **no inline `#`**
  (`mem.gitignore.no-inline-comments`).
- **The git-addable proof shells real `git`.** `created_backlog_item_is_git_addable`
  inits a temp repo, writes the two canonical gitignore lines, runs the real `new`,
  then asserts `git check-ignore -q <item>` exits **1** (not-ignored = negation
  live) AND `git add <item>` succeeds. Faithful R5 wiring proof, not a string match.
- **`#![expect(dead_code)]` STILL fulfilled, unchanged.** `new` makes `kind/prefix/
  canonical_id/as_str/has_facet` + the renders live, but `Status::is_terminal`,
  `Resolution`/`RiskLevel` + their `as_str`, `from_prefix`, the `Raw*`/`validate`
  parse tier, `BacklogItem`, and `KIND_PRECEDENCE` stay dead in the lib build;
  `KIND_PRECEDENCE` stays dead in BOTH builds and keeps the expectation green.
  **Retire the module expect at PHASE-05** when `edit` consumes the last of it.

## PHASE-03 — `backlog list` (survey + visibility matrix)

Decisions worth surviving the phase sheet (audit-harvest candidates):

- **A NEW full-entity reader, NOT `meta::read_metas`.** `read_kind(root, kind)`
  rides `entity::scan_ids` (numeric dirs only; **missing dir → empty Vec**, the C2
  total-function tolerance) and parses+`validate`s each `backlog-NNN.toml` into a
  full `BacklogItem` — `read_metas` yields only the 4 `meta::Meta` keys (no
  `kind`/`status`-as-enum/`tags`). `read_all` iterates `ItemKind::ALL` and extends.
  `meta.rs`/`entity.rs` UNCHANGED (R6/EX-3 held — `git diff` empty across the pair).
- **Compute/print split mirrors `adr::{list_rows,run_list}`.** `list_rows(root,
  &ListFilter) -> Result<String>` is the testable half (read→`select`→sort→
  `format_rows`); `run_list` prints it via `write!` (no extra newline —
  `render_table` carries its own). Tests assert the rendered String, never capture
  stdout.
- **Filters bundled in a private `ListFilter`** (`kind/status/tag/substr/all`) so
  the verb stays at 6 args / 1 bool — under the clippy arg+bool ceilings
  (`mem.pattern.lint.cli-handler-args-struct`) — and the compute half is one
  testable argument. All axes AND together.
- **Visibility folded into `select` (one predicate, no special branches).** `Some(
  status) ⇒ status == s` (an explicit `--status` reveals a terminal state);
  else `all || !is_terminal`. Promoted is a *resolution* on a terminal *status*,
  so it falls out by the terminal rule with NO promoted-specific code (design D5).
  `is_terminal` goes live here.
- **Sort key `(kind.ordinal(), id)` — declaration order, NOT `KIND_PRECEDENCE`.**
  Added `ItemKind::ALL` (decl order, the single source for both the cross-kind read
  and `ordinal`) + `const fn ordinal`. A deterministic GROUPING, explicitly not a
  priority claim (R7). `KIND_PRECEDENCE` (risk-first, the future resolver's) stays
  referenced-nowhere. DRY: test `ALL_KINDS` retired in favour of `ItemKind::ALL`.
- **`#![expect(dead_code)]` STILL fulfilled.** `list` makes `is_terminal`,
  `ItemKind::ALL`/`ordinal`, and the `BacklogItem` list fields live; `from_prefix`,
  `Resolution`/`RiskLevel::as_str`, and `resolution`/`facet`/`created`/`updated`
  stay dead → expectation still met. **Retire at PHASE-05.**
- **Test fixture: a direct-toml `write_item`** (the `meta::tests::write_meta_toml`
  precedent) seeds an arbitrary `status`/`resolution` without the unbuilt PHASE-05
  `edit` verb, exercising the real `scan_ids`+`validate` reader path.

## PHASE-04 — `backlog show <ID>` (commit `<pending>`)

Decisions worth surviving the phase sheet (audit-harvest candidates):

- **`show` is TOML-only — NO prose body (`format_show(&BacklogItem)`).** Unlike
  `spec::run_show` (reads `.md` + members + interactions), backlog show is a pure fn
  of the validated item, which carries no prose field. Design §5.4 enumerates
  identity+facet+timestamps+relations and never lists the `.md`; "cannot go stale" =
  derived from the item's own structured state. So show opens exactly ONE file (the
  toml). The `.md` body stays scaffold-only, surfaced by neither verb in v1.
- **`read_item(root, kind, id)` factored OUT of `read_kind`'s loop** (DRY) — the
  single-id read shared by the `list` cross-kind loop and `show`. One parse path; a
  missing file is a hard error (never implicit create, §5.5). No parse re-fork.
- **`parse_ref` upper-cases the prefix; deliberately NOT shared with
  `spec::resolve_spec_ref`.** Both do `rsplit_once('-')` + prefix-lookup + `u32` tail,
  but backlog tolerates case (`iss-7` → Issue) per §5.5 whereas spec refs are always
  canonical. A shared helper would impose one case-policy on both — kept separate,
  the divergence is commented at both sites (R3 / the "load-bearing prefix" note).
- **Relations render per-axis, non-empty only (D-PHASE04-2).** Mirror `spec::render`'s
  `if !is_empty` gating. "Seam always present even when empty" (§5.5 invariant) is a
  STORAGE rule, not a render rule — an item with no outbound relations shows no
  `relationships:` block. OUTBOUND only; the inbound/reverse view is the deferred
  registry surface's (ADR-004 / PRD-011), never computed by `show` — the VT-3 negative.
- **Unassessed-risk render = a bare `[facet]` header.** A seeded risk has
  `facet=Some` with all-`None` axes + empty `controls`, so the block is gated on
  `item.facet` (kind-bearing) and prints `[facet]` with no axes — a truthful
  "assessable, unassessed" signal, consistent with `new`/`list` treating risk as
  facet-bearing. Per-axis lines appear once assessed (`likelihood`/`impact`/`origin`/
  `controls`). Accepted as-is (within §5.4 latitude); flag for audit if a reviewer
  wants the header suppressed until assessed.
- **`#![expect(dead_code)]` STILL fulfilled.** `show` makes the LAST broad surface
  live — `from_prefix`, `Resolution::as_str`, `RiskLevel::as_str`, and the
  `resolution`/`facet`/`created`/`updated` fields all consumed by `format_show`. Only
  `KIND_PRECEDENCE` (referenced nowhere) stays dead → the module expect remains
  fulfilled in both builds. **Retire the module expect at PHASE-05** (`edit`); nothing
  else is left to consume except the `validate_transition`/`edit` surface PHASE-05 adds.
- **R6 gate held:** `git diff src/entity.rs src/meta.rs` empty across PHASE-04 —
  backlog-local + a `Show` CLI arm only; the engine and shared `meta` path untouched.
