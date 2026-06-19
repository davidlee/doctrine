# Design SL-118: Estimate & value facet authoring CLI verbs

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-101, SL-102, SL-104, SL-118, SPEC-020, ADR-001, IDE-013); doc-local refs
     bare — O1 (§3), D1 (§7), VT-1/F-1 (§9). -->

## 1. Design Problem

SL-101–103 built the estimate/value facets (model, parse/validate, unit
resolution, catalog/graph projection); SL-102 added the pure display formatters.
Across all of it the facet is **unauthorable from the CLI** — the only way to
attach an `[estimate]`/`[value]` table to an entity is hand-editing its `*.toml`.
No `doctrine` verb writes facet values; the authored-TOML write seams today are
edges (`link`/`needs`/`after`) and top-level status, both in `src/dep_seq.rs`.

This slice closes that gap: CLI verbs that author / edit / clear the `[estimate]`
and `[value]` facets, routed through the **existing** pure parse/validate
(SL-101) and an edit-preserving `toml_edit` write. Paired with IMP-112 (wire
display onto `show`), it is what makes the facets human-usable rather than a
contract proven only in tests.

## 2. Current State

- `estimate::parse_optional` / `estimate::validate` — pure, reusable; validation
  matrix (`lower >= 0`, `upper >= lower`, both finite, both required).
- `value::parse_optional` — pure; finite validation baked into `normalise`; **no
  standalone `validate`** (added here, §5).
- Both facets carry `#[serde(flatten)] _extra: BTreeMap` — unknown keys absorbed,
  ignored, round-tripped (NF-003 forward-compat).
- `src/dep_seq.rs` — "authored-TOML edit-preserving mutation leaf": `toml_edit`
  cores over `&mut DocumentMut`, `today` injected by the shell (no clock in core),
  IO wrappers do read→parse→core→write-once, no-op guard + F-1 refuse. Its cores
  act on **scaffold-seeded** keys (refuse if absent). **Facet tables are not
  seeded** — a third operation (alloc-if-absent) is needed.
- CLI verbs are `Command` enum variants in `main.rs` with `run_*` handlers; source
  refs resolve via `integrity::parse_canonical_ref` → `{stem}-{NNN}.toml` (the
  `resolve_link_path` shape). A Write/Read classification gate sits at `main.rs`
  (~L3047).

## 3. CLI Surface (O1 — subcommand group)

```
doctrine estimate set <ID> <lower> <upper>     # positional band
doctrine estimate set <ID> -x <N>              # point estimate (lower == upper)
doctrine estimate clear <ID>
doctrine value    set <ID> <magnitude>
doctrine value    clear <ID>
```

- `Command::Estimate` / `Command::Value` subcommand groups (each `Set`/`Clear`).
- `estimate set` positionals `[lower] [upper]` are **optional at clap**; `-x/--exact`
  is a flag. The handler enforces **exactly one mode**: both positionals XOR `-x`
  (clap cannot cleanly express both-or-neither-XOR-flag) — violation is a
  parse-equal error. `-x N` ⇒ `lower = upper = N` (zero-width valid, SL-101 e4).
- `<ID>` is positional-first, kind-agnostic (any entity with the facet seam).
- Output: `estimate set SL-118: lower=1 upper=3 (espresso_shots)` /
  `estimate cleared SL-118` / `no estimate on SL-118` (clear-absent no-op).

## 4. Architecture (O2 — new leaf `src/facet_write.rs`)

ADR-001 leaf: imports only `toml_edit` / `anyhow` / `std`; depends on no engine or
command module (verified by §9 layering test). Pure cores generic over
table-name + scalar fields — **one core serves both facets**:

```rust
/// Alloc-if-absent / replace the named facet table to exactly `fields`; bump
/// top-level `updated`. Returns `true` iff the document changed. Pure: no disk,
/// no clock — `today` injected. A whole-table append is position-independent, so
/// alloc-if-absent is corruption-safe (unlike a scalar tail-insert).
fn set_facet(doc: &mut DocumentMut, table: &str, fields: &[(&str, f64)], today: &str) -> bool

/// Remove the named facet table if present; bump `updated`. No-op (false) if
/// absent. Pure.
fn clear_facet(doc: &mut DocumentMut, table: &str, today: &str) -> bool

/// Shared read→parse→core→write-once-if-changed envelope (the ~5 lines reused by
/// both verbs and both facets).
fn edit_in_place(path: &Path, f: impl FnOnce(&mut DocumentMut) -> bool) -> anyhow::Result<bool>

// IO wrappers
fn apply_set(path, table, fields, today) -> Result<bool>
fn apply_clear(path, table, today) -> Result<bool>
```

- Fields written via `toml_edit::value(f)` — never string-spliced (sidesteps the
  splice-escape footgun; values are `f64` so no escaping needed anyway).
- **No-op guard**: if the table already equals target, write nothing (content +
  mtime hold) → `false`. `updated` bumps **only on change**. (Value-equality
  compare: a hand-authored `lower = 2` (int) equal to a `2.0` set is a no-op — the
  int is left un-normalized; harmless, the read path normalizes.)
- **`updated` safety (adversarial A-1):** facets are kind-agnostic, so the target
  may be a kind whose TOML lacks a top-level `updated` key. The bump
  **`insert`s only if `updated` is already present** (a seeded scalar — safe to
  replace); if absent it is **skipped, never inserted** (a tail-insert would land
  inside a trailing subtable = corruption, the F-1 hazard). The facet write still
  succeeds.
- `set_facet` inserts only the named keys — **sibling keys/sub-tables in
  `[estimate]` are left verbatim** (the forward-compat guarantee, §6).
- `[estimate]`/`[value]` author the table as an explicit (non-implicit) table.

### Command tier (`main.rs`)

- Handlers `run_estimate_set/clear`, `run_value_set/clear`: resolve `<ID>` →
  toml path (factor the path-from-ref helper currently inline in
  `resolve_link_path`), then validate, then call the leaf.
- **Validation stays in the command tier** (the leaf must not depend on
  `estimate`/`value` — ADR-001): build `EstimateFacet`/`ValueFacet`, call
  `estimate::validate` / new `value::validate`, *then* `facet_write::apply_set`.
  The CLI rejects exactly what parse rejects.
- Register the Write class at `main.rs` (~L3047):
  `Command::Estimate { .. } | Command::Value { .. } => Write("estimate"/"value")`.

## 5. Validation reuse (D2)

- estimate: build `EstimateFacet { lower, upper }`, finite-checked at f64 parse of
  the CLI args, then `estimate::validate` (the one matrix, no second impl).
- value: add a thin pure `value::validate(&ValueFacet) -> Result<()>` (finite) so
  the verb and the parse path share one rule (symmetry with estimate; currently
  finite lives only inside `normalise`).

## 6. Forward-compat — history-ready, not history-bearing (IDE-013)

A time-series of facet edits was raised and **deferred** (IDE-013): stashing
history in an unread `_extra` field writes data nothing reads under no REQ — the
unspec'd-residue debt pattern SL-104 is paying down, and a premature wire-shape
lock. It wants its own REQ in SPEC-020, routed like the confidence legitimization.

SL-118 ships history-**ready** for free: the edit-preserving writer (§4) touches
only the managed keys, so any future `[estimate]` sub-key (e.g. a `history` array)
**survives every `set`**. §9 VT-7 pins this. `clear` removes the whole facet table
(history with it) — a retention decision left to IDE-013.

## 7. Decisions

- **D1** — `set`/`clear` bump top-level `updated = today` (provenance; matches the
  status seam), behind the no-op guard so an unchanged write never bumps. **Yes.**
- **D2** — add thin `value::validate` for symmetry. **Yes.**
- **D3** — `<ID>` positional-first; bounds positional after it. **Yes.**
- **D4** — clear-when-absent is a friendly no-op (exit 0), not an error. **Yes.**
- **D5** — unresolved / non-existent `<ID>` → `bail` via `parse_canonical_ref` +
  existence check (never write a file for a non-entity).

## 8. Risks

- **R1 — clap mode ambiguity.** Optional positionals + `-x` cannot be a pure clap
  ArgGroup; the both-or-neither-XOR-flag rule is handler-enforced. Mitigation: a
  dedicated parse-equal error + VT covering each illegal combination.
- **R4 — leading-dash positionals (adversarial A-2).** `value` magnitude may be
  **negative** (value validates finite only, no range) — `value set SL-1 -5` has
  clap read `-5` as an unknown flag. Mitigation: `allow_hyphen_values = true` on the
  magnitude positional (`-- -5` also works). Estimate bounds are `>= 0`, so a
  negative there is rejected regardless — acceptable as a clap-level error; VT-8
  asserts rejection, not the message.
- **R2 — `updated` coupling.** Bumping `updated` couples facet-write to the date
  inject. Mitigation: `today` already threaded for the status seam; no-op guard
  prevents spurious bumps.
- **R3 — leaf-naming.** A new `facet_write.rs` rather than extending `dep_seq.rs`;
  accepted for cohesion + naming honesty (§O2). No shared code is duplicated (the
  cores are operation-specific; only the IO envelope is shared, and it lives here).

## 9. Verification

Leaf unit (pure, `facet_write`):
- **VT-1** `set_facet` allocates an absent `[estimate]` table.
- **VT-2** `set_facet` replaces present bounds.
- **VT-3** idempotent no-op: second identical `set` returns `false`, content+mtime
  hold.
- **VT-4** `clear_facet` removes the table; clear-absent returns `false`.
- **VT-5** golden round-trip: unrelated tables / comments / `[relationships]`
  preserved across `set` and `clear`.
- **VT-6** `updated` bumps only on change (no-op set does not bump); a target
  whose TOML lacks `updated` is written without it (A-1: no insert, no corruption).
- **VT-7** forward-compat: a `set` over an `[estimate]` table carrying an unknown
  sub-key (`history`-shaped) **preserves** that sub-key (IDE-013 readiness).

Command tier:
- **VT-8** invalid matrix rejected with parse-equal verdicts: missing bound,
  negative lower, `upper < lower`, non-finite, `-x` + positionals conflict, neither
  mode supplied.
- **VT-9** `-x N` sets `lower == upper == N`.
- **VT-10** `value set` / `value clear` round-trip; `value::validate` rejects
  non-finite.
- **VT-11** round-trip via catalog scan: `set` → scan reads the normalized facet;
  `clear` → scan reads absent.

Architecture:
- **VT-12** layering `#[test]`: `facet_write` imports only `toml_edit`/`anyhow`/
  `std` (ADR-001). (Folds into SL-112's fitness gate if landed; else a local
  source-scan.)

Dogfood:
- **VH-1** author an estimate + value on a live entity via the verb (not by hand),
  confirm `list`/scan read them back.

## 10. Out of Scope (Non-Goals)

- **Display / `show` wiring** — IMP-112 (formatters already exist).
- **Confidence authoring** — blocked on SL-104 legitimization; no `--*-confidence`.
- **Change history** — IDE-013 (this slice is history-*ready* only, §6).
- New validation semantics, aggregation, gating — none.
