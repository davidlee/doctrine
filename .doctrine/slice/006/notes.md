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

## Deviation from plan text (accepted)

- Plan PHASE-01 EX-1 lists `today()` for meta.rs. **Not extracted** — `today()` was
  already single-sourced in `crate::clock` (the time seam) before this slice. Intent
  (one clock) already met. ADR scaffolds call `crate::clock::today()`, same as slice.
  Logged in phase-01.md Findings + the P01 commit.

## Locked decisions (don't relitigate — design §7)

- **D4** extract concrete fns only, **no `numeric_entity` trait/generic**.
- **meta.rs, not entity.rs** — presentation/reader ≠ kind-blind engine.
- ADR is a **local authored entity, no backend** (forgettable's ADR-005 = memory).
