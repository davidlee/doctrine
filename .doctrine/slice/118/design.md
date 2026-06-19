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
- `estimate set` positionals `[lower] [upper]` are **optional at clap**;
  `exact: Option<f64>` (`-x/--exact <N>` — an option taking exactly one value, **not
  a bool flag**, adversarial F2) with `conflicts_with_all = [lower, upper]`. The
  handler enforces the remaining rule — **exactly one mode present**: `exact`, or
  *both* positionals; neither (or one lone positional) is a parse-equal error.
  `-x N` ⇒ `lower = upper = N` (zero-width valid, SL-101 e4). `-x` with no argument
  is a clap error (VT-8).
- `<ID>` is positional-first, kind-agnostic (any entity with the facet seam).
- Output: `estimate set SL-118: lower=1 upper=3 (espresso_shots)` /
  `estimate cleared SL-118` / `no estimate on SL-118` (clear-absent no-op).

## 4. Architecture (O2 — new leaf `src/facet_write.rs`)

ADR-001 leaf: imports only `toml_edit` / `anyhow` / `std`; depends on no engine or
command module (verified by §9 layering test). Pure cores generic over
table-name + scalar fields — **one core serves both facets**. No clock: facet
writes do **not** bump `updated` (D1 reversed, §7) — so no `today` injection:

```rust
/// Mutate ONLY the named managed keys of the `[table]` facet, allocating the
/// table if absent and preserving every non-managed sibling key/sub-table.
/// Returns `true` iff the document changed. Pure: no disk, no clock.
/// Errors if `[table]` is present but not a standard table (e.g. `estimate = 7`
/// or `[[estimate]]`) — fail-loud, never silently overwrite a malformed shape.
fn set_facet(doc: &mut DocumentMut, table: &str, fields: &[(&str, f64)]) -> anyhow::Result<bool>

/// Remove the `[table]` facet if present. No-op (false) if absent. Pure.
fn clear_facet(doc: &mut DocumentMut, table: &str) -> bool

/// Shared read→parse→core→write-once-if-changed envelope (the ~5 lines reused by
/// both verbs and both facets).
fn edit_in_place(path: &Path, f: impl FnOnce(&mut DocumentMut) -> anyhow::Result<bool>) -> anyhow::Result<bool>

// IO wrappers
fn apply_set(path, table, fields) -> Result<bool>
fn apply_clear(path, table) -> Result<bool>
```

- **Mutate-in-place, not replace (adversarial F4):** `set_facet` inserts/overwrites
  only the managed keys (`lower`/`upper` or `value`); **every non-managed sibling
  key or sub-table in `[estimate]` is left verbatim** — this is the forward-compat
  guarantee (§6, VT-7). Alloc-if-absent creates the table carrying just the managed
  keys. A whole-table append is position-independent → corruption-safe.
- **Malformed-present fail-loud (F4):** if the `estimate`/`value` key exists but is
  not a standard table (`estimate = 7`, `[[estimate]]`), `set_facet` errors rather
  than clobbering — surfaces hand-edit corruption instead of papering over it
  (the house F-1 stance).
- Fields written via `toml_edit::value(f)` — never string-spliced (values are
  `f64`; no escaping needed).
- **No-op guard**: if the managed keys already equal target, write nothing
  (content + mtime hold) → `false`. (Value-equality: a hand-authored `lower = 2`
  (int) equal to a `2.0` set is a no-op — the int is left un-normalized; harmless,
  the read path normalizes.)

### Command tier (`main.rs`)

- Handlers `run_estimate_set/clear`, `run_value_set/clear`: resolve `<ID>` →
  toml path (factor the path-from-ref helper currently inline in
  `resolve_link_path`), then validate, then call the leaf.
- **Validation stays in the command tier** (the leaf must not depend on
  `estimate`/`value` — ADR-001): build `EstimateFacet`/`ValueFacet`, call
  `estimate::validate` / `value::validate`, *then* `facet_write::apply_set`. For
  the CLI to reject **exactly** what parse rejects, `validate` must be the
  *complete* rule (see §5, adversarial F1 — finiteness must move into `validate`).
- Register the Write class at `main.rs` (~L3047):
  `Command::Estimate { .. } | Command::Value { .. } => Write("estimate"/"value")`.

## 5. Validation reuse — one complete rule per facet (D2, adversarial F1+F5)

The pre-review claim "CLI rejects exactly what parse rejects" was **false**: clap's
`f64` parser accepts `inf`/`nan`, and `estimate::validate` (`src/estimate.rs:199`)
checks only `lower >= 0` / `upper >= lower` — **no finiteness**. Finiteness lives
only in the parse-path `toml_to_f64`. So `estimate set X inf inf` would pass the
command-tier validate and only be dropped later by the catalog read. Fix:

- **Move finiteness into `validate`.** `estimate::validate` gains finite checks on
  both bounds; `estimate::normalise` already calls `validate`, so the parse path is
  unaffected (belt-and-suspenders with `toml_to_f64`). `validate` is now the single
  complete rule, shared by CLI and parse.
- **`value::validate` must be wired, not parallel (F5).** Add pure
  `value::validate(&ValueFacet)` (finite) **and call it from `value::normalise`**
  before returning — exactly as `estimate::normalise` calls `estimate::validate`.
  Without that wiring it would be a second rule site (violating no-parallel-impl);
  with it, one rule serves CLI + parse.

## 6. Forward-compat — history-ready, not history-bearing (IDE-013)

A time-series of facet edits was raised and **deferred** (IDE-013): stashing
history in an unread `_extra` field writes data nothing reads under no REQ — the
unspec'd-residue debt pattern SL-104 is paying down, and a premature wire-shape
lock. It wants its own REQ in SPEC-020, routed like the confidence legitimization.

SL-118 ships history-**ready** for free: the edit-preserving writer (§4) touches
only the managed keys, so any future `[estimate]`/`[value]` sub-key (e.g. a
`history` array) **survives every `set`**. §9 VT-7 pins this on **both** facets.
`clear` removes the whole facet table (history with it) — a retention decision left
to IDE-013.

### Read-surface honesty (adversarial F6)

Facets are authorable on **any** kind, and that is **not** write-only data: the
catalog scan reads both facets kind-agnostically (`src/catalog/scan.rs:198`
`read_facets`), feeding the graph projection and map (SL-103) — the kind-agnostic
read surface SPEC-020 NF-002 defines. What is *slice-only* today is the **typed
per-entity reader** (`SliceDoc` carries `estimate`/`value`) and the `show` display
(IMP-112, slice-first). So an estimate on an ADR/RV is read by graph/map, not by
that entity's `show`, until a generic cross-kind facet read/show contract lands.
This slice does not narrow authoring to slices — but it does not *claim*
per-entity-show readability for non-slice kinds either (acceptance proof split
accordingly, §9).

## 7. Decisions

- **D1 — REVERSED (adversarial F3).** Facet writes do **not** bump `updated`.
  Empirical: `updated` is **not uniformly seeded** — the `review` (RV) kind has no
  `updated` field. A blunt "skip if absent" masks corruption on kinds that *do*
  seed it; a blunt "refuse if absent" wrongly rejects RV; distinguishing them
  couples the writer to per-kind seed metadata. Not worth it: the facet is a
  side-table, the catalog/graph read ignores `updated`, and git + the conventional
  commit already record provenance. `updated` stays the status seam's job. This also
  removes clock injection from the leaf entirely (pure, no `today`).
- **D2** — `value::validate` is added **and wired into `value::normalise`** (F5), so
  it is one rule, not a parallel one. `estimate::validate` gains finiteness (F1).
- **D3** — `<ID>` positional-first; bounds positional after it. **Yes.**
- **D4** — clear-when-absent is a friendly no-op (exit 0), not an error. **Yes.**
- **D5** — unresolved / non-existent `<ID>` → `bail` via `parse_canonical_ref` +
  existence check (never write a file for a non-entity).
- **D6 — `exact` is `Option<f64>`**, not a bool flag (F2); `conflicts_with` the
  positionals, handler enforces exactly-one-mode.

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
- **R2 — RESOLVED by D1 reversal.** No `updated` bump → no date inject, no
  per-kind seed-presence logic, no corruption-masking. The leaf is fully pure.
- **R3 — leaf-naming.** A new `facet_write.rs` rather than extending `dep_seq.rs`;
  accepted for cohesion + naming honesty (§O2). No shared code is duplicated (the
  cores are operation-specific; only the IO envelope is shared, and it lives here).

## 9. Verification

Leaf unit (pure, `facet_write`):
- **VT-1** `set_facet` allocates an absent `[estimate]` table (managed keys only).
- **VT-2** `set_facet` overwrites present managed keys in place.
- **VT-3** idempotent no-op: second identical `set` returns `false`, content+mtime
  hold.
- **VT-4** `clear_facet` removes the table; clear-absent returns `false`.
- **VT-5** golden round-trip: unrelated tables / comments / `[relationships]`
  preserved across `set` and `clear`.
- **VT-6** malformed-present fail-loud (F4): `set_facet` over `estimate = 7` and
  over `[[estimate]]` **errors**, file untouched.
- **VT-7** forward-compat (F7): a `set` over an `[estimate]` **and** a `[value]`
  table carrying an unknown sibling sub-key (`history`-shaped) **preserves** it
  (IDE-013 readiness).

Command tier:
- **VT-8** invalid matrix rejected with parse-equal verdicts: missing bound,
  negative lower, `upper < lower`, **non-finite `inf`/`nan` (F1 parity)**, `-x` +
  positionals conflict, **`-x` with no argument (clap, F2)**, one-lone-positional,
  neither mode supplied.
- **VT-9** `-x N` sets `lower == upper == N`.
- **VT-10** `value set` / `value clear` round-trip; `value::validate` rejects
  non-finite **via `normalise` too** (one rule, F5).
- **VT-11** round-trip via catalog scan: `set` → scan reads the normalized facet;
  `clear` → scan reads absent. (Proves the **catalog/graph** read only.)
- **VT-12** typed-reader round-trip on a **slice**: `estimate set SL-<n> …` →
  `slice show` / `SliceDoc` reads the facet back (the per-entity read surface
  catalog scan does not exercise, F7). Once IMP-112 lands this extends to display.

Architecture:
- **VT-13** layering `#[test]`: `facet_write` imports only `toml_edit`/`anyhow`/
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
