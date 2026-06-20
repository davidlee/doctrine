# SL-129 Design: Corpus-wide entity id→path helper

## Summary

Add `stem: &'static str` to `entity::Kind`; add `entity::id_path` (+ `Ext` enum)
and `entity::rel_path` helpers; remove now-duplicate `stem` from `integrity::KindRef`
and `governance::GovKind`; replace all ~85 production `format!` sites (plus ~8 test-only) that hand-roll
the `<dir>/<NNN>/<stem>-<NNN>.{toml,md}` formula with the data-driven helper.

## Data model

### `entity::Kind` gains `stem`

```rust
#[derive(serde::Serialize)]
pub(crate) struct Kind {
    pub dir: &'static str,
    pub prefix: &'static str,
    #[serde(skip)]                // not serialized — JSON catalog shape unchanged (RV-117 F-2)
    pub stem: &'static str,      // "slice", "revision", "adr", etc.
    pub scaffold: fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset>,
}
```

Sub-kinds (DESIGN_KIND, PLAN_KIND, NOTES_KIND) get `stem: ""`. The shared
`make_file_name` helper guards both `id_path` and `rel_path`.

### New helpers

```rust
pub(crate) enum Ext { Toml, Md }

/// Shared filename computation — one guard against stem-less kinds (RV-117 F-3).
fn make_file_name(kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    debug_assert!(!kind.stem.is_empty(),
        "{}: stem-less kind {}", module_path!(), kind.prefix);
    let n = format!("{id:03}");
    let file = match ext {
        Ext::Toml => format!("{}-{n}.toml", kind.stem),
        Ext::Md   => format!("{}-{n}.md", kind.stem),
    };
    PathBuf::from(&n).join(file)
}

pub(crate) fn id_path(root: &Path, kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    root.join(kind.dir).join(make_file_name(kind, id, ext))
}

pub(crate) fn rel_path(kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    make_file_name(kind, id, ext)
}
```

### `integrity::KindRef` loses `stem`

```rust
pub(crate) struct KindRef {
    pub(crate) kind: &'static entity::Kind,   // stem now at kind.stem
    pub(crate) state_dir: Option<&'static str>,
}
```

All KINDS rows drop the `stem:` literal — they reference `kind.stem` through the
existing `kind` field.

### `governance::GovKind` loses `stem`

```rust
pub(crate) struct GovKind {
    pub kind: entity::Kind,                    // stem now at kind.stem
    pub statuses: &'static [&'static str],
    pub hidden: bool,
}
```

~10 sites using `g.stem` become `g.kind.stem` (lines 73, 87, 217, 246, 356,
359, 369, 375, 404, 444). The 4 GovKind constructors (ADR_KIND, POLICY_KIND,
STANDARD_KIND, RFC_KIND) drop their `stem:` field. 3 of those sites are
path-construction format! calls (lines 215, 220, 398) — replaced upstream in
Pattern A. The remaining ~7 are serde-key/error-format uses — automatically
fixed by the compiler after GovKind::stem removal.

## Replacement patterns

**Pattern A — `id_path` (root in scope):** `root.join(kind.dir).join(&name).join(format!("slice-{name}.toml"))`
→ `entity::id_path(root, kind, id, Ext::Toml)`. ~50 sites.

**Pattern B — `rel_path` (scaffold closures):** `PathBuf::from(format!("{name}/{BACKLOG_STEM}-{name}.toml"))`
→ `entity::rel_path(kind, id, Ext::Toml)`. ~20 sites.

**Pattern C — `format!("{}-{name}.toml", kref.stem)`:** becomes
`entity::id_path(root, kref.kind, id, Ext::Toml)` — `kref.stem` → `kref.kind.stem`
via the field removal. ~23 sites.

## Compile-order commits

1. **Add + seed:** Add `stem` to `Kind`, add `Ext`/`id_path`/`rel_path`, add `stem:`
   to all 36 Kind initializers (30 production + 6 test). Compiles (KindRef/GovKind unchanged).
2. **Remove + replace:** Remove `KindRef::stem`, remove `GovKind::stem`,
   update all KINDS rows, update `g.stem` → `g.kind.stem`, replace all ~85 production format!
   sites with helper calls.
3. **Gate:** `just check` — full project gate (build, all tests, clippy, fmt).
    Baseline recorded before PHASE-01; unchanged after PHASE-02.

## Verification

- Behaviour-preservation: every replacement produces the identical path
  (`id_path` is pure over Kind data).
- `just check` green before (baseline) and after.
- `git diff --stat` confirms only intended files touched.

## Decided choices

- **stem on Kind** (not KindRef) — fundamental identity field alongside dir/prefix.
- **GovKind.stem removed** — source of truth is kind.stem (option 1, user-confirmed).
- **Sub-kinds get empty stem** — minimal sentinel; shared `make_file_name` guard
  on both helpers (RV-117 F-3).
- **Test assertion full-path strings left alone** — readability of failure output.
- **`meta.rs` internals excluded** — `read_meta`/`read_id` already abstracted behind
  a utility with a stem parameter; their callers pass a kind-tree root, not a project
  root, so `id_path` would double-join `kind.dir`.
- **`BACKLOG_STEM` constant left in place** — harmless, not worth removing.
- **~85 production + ~8 test-only sites** (scope doc inventory superseded by this design count — RV-117 F-4).

## Adversarial review resolution

| Finding | Severity | Disposition |
|---------|----------|-------------|
| F1: id_path/rel_path duplication | nit | Accepted — internal helper refactored at implementation time |
| F2: debug_assert only in debug | nit | Accepted — acceptable for single-author tool |
| F5: main.rs sites are test-only | minor | Accepted — still replaced, count corrected |
| F6: meta.rs internals excluded | minor | Accepted — already abstracted |
| F7: meta.rs uses kind-root, not project-root | minor | Flagged — callers must not switch to id_path |
| F8: g.stem → g.kind.stem JSON output same | nit | Confirmed — runtime value, not field name |
| F9: BACKLOG_STEM constant | nit | Accepted — kept as dead harmless |
| RV-117 F-1: verification gate understated | major | Fixed — "full just check" replaces "7 tests" |
| RV-117 F-2: Kind serialization drifts catalog JSON | major | Fixed — `#[serde(skip)]` on `stem` |
| RV-117 F-3: rel_path lacks stem-less guard | minor | Fixed — shared `make_file_name` guards both |
| RV-117 F-4: stale scope inventory | minor | Fixed — slice-129.md reconciled with design counts |
