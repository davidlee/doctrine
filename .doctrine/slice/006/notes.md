# Notes SL-006: ADR support

Durable cross-phase facts harvested from phase sheets. Disposable scratch stays
in `state/.../phases/`; this survives.

## PHASE-01 outcomes (the shared substrate ADR rides)

- **`src/meta.rs` is the shared metadata-list module** (design D4's name, now real).
  Holds: `pub(crate) struct Meta { id, slug, title, status }` (fields `pub(crate)`),
  `sort_and_filter(Vec<Meta>, Option<&str>)`, `format_list(&[Meta])`,
  `read_meta(tree_root, stem, id)`, `read_metas(tree_root, stem)`.
- **`stem` param** = toml file stem (`"slice"` / `"adr"`) → `{stem}-NNN.toml`.
  Distinct from `entity::Kind.prefix` (`"SL"` / `"ADR"`). Separate argument; never
  conflate. `read_metas` reuses `entity::scan_ids` (numeric dirs only).
- **`read_metas` is unsorted** — yields `scan_ids` (readdir) order. `sort_and_filter`
  owns ordering. ADR `list` must call `sort_and_filter` (same as slice), not assume
  read order.
- **`Meta` fields are `pub(crate)`** so adr.rs (and its tests) can read/round-trip.

## PHASE-02 outcomes (the ADR kind surface P03 wires)

- **`src/adr.rs` exists** (commit `4d647b9`): `ADR_KIND` (Fresh, `prefix="ADR"`,
  `dir=".doctrine/adr"`, `scaffold=adr_scaffold`), `render_adr_toml(id,slug,title,
  date)`, `render_adr_md(canonical_ref, title)`, `adr_scaffold(ctx)->Fileset`
  (2 files + `NNN-slug` symlink — the `slice_scaffold` shape). All `fn`-private;
  P03 verbs (`run_new`/`run_list`) go in the same module.
- **`ADR_KIND` carries `#[expect(dead_code, reason=…PHASE-03…)]`** — it has no
  consumer until the verbs. **P03 must drop that attribute** the moment `run_new`
  references it, or clippy flips (an `expect` that no longer fires is itself a warning).
- **`adr.toml` template gained `schema="doctrine.adr"` + `version=1`** (slice.toml
  has neither). `Meta` ignores both → round-trip unaffected. They exist for P04's
  `toml_edit` mutation to assert it is editing a doctrine.adr.
- **`[relationships]` is four inert empty arrays** (`supersedes`/`superseded_by`/
  `related`/`tags`) — authored-but-inert; nothing reads them in v1.
- **Templates need no manifest entry** — `install.rs` `build_plan` loops
  `embedded_filenames()` (RustEmbed auto-embed of `install/`); slice.toml isn't
  listed either. Don't invent install config for adr.{toml,md}.

## P03 reuse seam (resolve before coding — DRY gate)

- **`resolve_title` is private to `slice.rs:236`** and the **slug-resolution policy**
  (`--slug` else `entity::derive_slug`, then empty-bail) is **inline in
  `slice::run_new:264-272`**. VT-2 says ADR `run_new` *reuses* both. Two are
  currently slice-local. Decision for P03: lift `resolve_title` (and ideally a small
  `resolve_slug(title, slug)` helper) to a shared home (`entity` or a thin shared
  module) and have both slice + adr call it — do **not** copy-paste (CLAUDE.md: no
  parallel implementation). `derive_slug` is already `entity::derive_slug` (shared).

## Deviation from plan text (accepted)

- Plan PHASE-01 EX-1 lists `today()` for meta.rs. **Not extracted** — `today()` was
  already single-sourced in `crate::clock` (the time seam) before this slice. Intent
  (one clock) already met. ADR scaffolds call `crate::clock::today()`, same as slice.
  Logged in phase-01.md Findings + the P01 commit.

## PHASE-03 outcomes (verbs landed; the seam P04 does NOT touch)

- **`adr::run_new` / `adr::run_list` exist** (commit `dc807a5`), thin CLI shells
  mirroring slice. `run_new` → `entity::materialise(&ADR_KIND, Fresh, …)` (monotonic
  id + race-retry inherited); `run_list` → `meta::read_metas(adr_root,"adr")` →
  `sort_and_filter` → `format_list`. `adr_root = root.join(ADR_DIR)`. `ADR_DIR` /
  `ADR_KIND` are still `const`-private in adr.rs; the `#[expect(dead_code)]` is gone.
- **The reuse seam resolved → `src/input.rs`** (the P03 DRY gate). New thin-shell
  module = CLI-input resolution for a `new` verb, symmetric to `meta.rs` (list
  *output*); `entity.rs` stays the kind-blind engine. Two `pub(crate)` fns:
  `resolve_title(Option<String>)` (moved verbatim from slice — arg|stdin, non-empty)
  and `resolve_slug(title, Option<String>)` (folds slice's old inline `--slug |
  derive_slug` + empty-bail). **Both `slice::run_new` and `adr::run_new` call them**
  — single-sourced, no copy. slice suite stayed green (behaviour-preservation).
- **`main.rs` has `Command::Adr { AdrCommand::{New,List} }`** + dispatch arm
  (mirror of `Command::Slice`). P04 extends `AdrCommand` with `Status`.
- **`memory::run_record` keeps a third "Title must not be empty"** check on a
  required `&str` (`memory.rs:551`, no Option/stdin) — left out of the input.rs lift
  (different signature). Future DRY candidate if memory gains an arg|stdin path.

## PHASE-05 outcomes (close-out: E2E + docs)

- **E2E test = `adr::tests::end_to_end_new_x2_list_status_accept_then_filtered_list`**
  (EX-1/VT-1). Drives the *real* verb cores in one tree — `run_new` x2 →
  `read_metas`/`sort_and_filter` list → `set_adr_status` accept → filtered list (only
  001). The earlier piecemeal tests flipped status by raw rewrite (the verb was P04);
  this is the first test exercising the actual status mutation through the list path.
  Kept in-process in the `adr.rs` test module — the project has no `tests/` dir; all
  suites are inline (idiom + DRY on the `adr_root` helper).
- **AGENTS.md (CLAUDE.md symlink) updated** (EX-2): `.doctrine/adr/nnn/` in the layout
  block; ADRs folded into the storage-rule **authored** tier (status in `adr-nnn.toml`).
  CLI surface lines (`adr new|list|status`) were already present from P03/P04.
- **Slice closed**: phase-05 → completed, `slice-006.toml` status → done.

## Locked decisions (don't relitigate — design §7)

- **D4** extract concrete fns only, **no `numeric_entity` trait/generic**.
- **meta.rs, not entity.rs** — presentation/reader ≠ kind-blind engine.
- ADR is a **local authored entity, no backend** (the storage-backend decision = memory).
