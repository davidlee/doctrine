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

## Locked decisions (don't relitigate — design §7)

- **D4** extract concrete fns only, **no `numeric_entity` trait/generic**.
- **meta.rs, not entity.rs** — presentation/reader ≠ kind-blind engine.
- ADR is a **local authored entity, no backend** (forgettable's ADR-005 = memory).
