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
