# Design: Risk facet CLI verb (SL-134)

## 1. Current vs target behaviour

**Current:** Risk items scaffold `[facet]` with `likelihood`/`impact`/`origin`/
`controls` seeded empty. No CLI verb exists to set or clear them post-creation —
users hand-edit the TOML.

**Target:** `doctrine risk set <ID> --likelihood <LEVEL> --impact <LEVEL> …` and
`doctrine risk clear <ID>` — mirrors the `estimate`/`value` facet verb pattern.

## 2. Architecture

### 2.1 Pure/impure split

- **Pure:** `facet_write::set_facet_mixed` — edit-preserving TOML mutation
  (allocates table, overwrites managed keys, no-op guard, forward-compat).
  `RiskLevel` enum validation is handled by clap's `ValueEnum` parse (invalid
  tokens rejected before the handler runs) — no separate pure `validate` needed.
- **Impure:** `run_risk_set` / `run_risk_clear` — root resolution, file I/O,
  kind gate, echo. The thin shell calls the pure cores.

### 2.2 FacetField — mixed-type payload for facet_write

`facet_write::set_facet` accepts only `&[(&str, f64)]` (float-only). Risk needs
string and array fields too. Extend the shared leaf with one new public core:

```rust
// src/facet_write.rs

pub(crate) enum FacetField {
    Str { key: &'static str, value: String },
    Arr { key: &'static str, values: Vec<String> },
}

pub(crate) fn set_facet_mixed(
    doc: &mut toml_edit::DocumentMut,
    table: &str,
    fields: &[FacetField],
) -> anyhow::Result<bool>;
```

`FacetField` carries typed payload so the command layer never touches
`toml_edit` directly. The existing `set_facet` and its `apply_set` wrapper are
unchanged — this is additive, not a rewire.

**No-op guard per type:**
- `Str`: read back via `Value::as_str()`, compare string-equal
- `Arr`: read back via `as_array()` → iterate `.get(i).and_then(Value::as_str)`,
  element-wise ordered compare. Non-string array elements (integer, nested
  table) are treated as "not equal" → triggers overwrite, matching the
  integer-tolerance pattern from `set_facet`'s float guard.

**Silent correction of malformed managed keys:** Both `set_facet` and
`set_facet_mixed` overwrite managed keys when their current value has the wrong
type (scalar where array expected, integer where string expected) — never bail
on a malformed managed key inside a valid table. This is consistent with the
existing `set_facet` behaviour for float fields.

**Behaviour-preservation gate:** All existing `set_facet` callers and VT tests
stay unchanged. `set_facet_mixed` is additive; the shared leaf compiles and
passes the full existing suite as-is.

### 2.3 Kind gate

`doctrine risk set` / `doctrine risk clear` refuse non-risk items. A small
`read_kind` helper in the command layer reads the `kind` field from the entity
TOML via `toml_edit` and matches against `ItemKind::ALL` (visibility bumped to
`pub(crate)` in `backlog.rs` — one token change, no logic change).

- Non-risk backlog item → `"RSK-008: risk set requires a risk item, got issue"`
- Non-backlog entity (no `kind` field) → `"no 'kind' field — not a backlog item"`

## 3. Command surface

### 3.1 `doctrine risk set`

```
Usage: doctrine risk set [OPTIONS] <ID>

Arguments:
  <ID>  Canonical entity ref (e.g. RSK-008)

Options:
  --likelihood <LEVEL>  likelihood level [possible values: low, medium, high, critical]
  --impact <LEVEL>      impact level [possible values: low, medium, high, critical]
  --origin <ORIGIN>     risk origin — free text description of the source
  --controls <CTRL>     controls — each occurrence replaces the entire list (not additive)
  -p, --path <PATH>     explicit project root (default: auto-detect)
```

The `controls` field annotation:

```rust
#[arg(long, long_help = "Controls — each occurrence replaces the entire list (not additive)")]
pub(crate) controls: Vec<String>,
```

Rules:
- At least one of `--likelihood` / `--impact` required — error if neither supplied
- `--origin` optional — sets the free-text `origin` field
- `--controls` repeatable — `--controls A --controls B` sets `controls = ["A", "B"]`
  (replaces the whole list; `long_help` states this explicitly)
- Levels auto-complete from `RiskLevel::ValueEnum` variants
- Kind gate enforced (see §2.3)

### 3.2 `doctrine risk clear`

```
Usage: doctrine risk clear [OPTIONS] <ID>

Arguments:
  <ID>  Canonical entity ref (e.g. RSK-008)

Options:
  -p, --path <PATH>  explicit project root (default: auto-detect)
```

Removes the entire `[facet]` table (likelihood, impact, origin, controls — all
gone). No-op if absent. Kind gate enforced.

### 3.3 Echo

Matches `estimate`/`value` pattern:

- Set changed: `risk set: RSK-008 likelihood=medium impact=high origin="supply chain" controls=["ci-check"]`
- Set unchanged: `risk unchanged: RSK-008 likelihood=medium impact=high`
  (includes all supplied fields — origin and controls also named if supplied but unchanged)
- Clear: `risk cleared: RSK-008`
- Clear no-op: `no risk facet to clear: RSK-008`

## 4. CLI wiring

```rust
// src/commands/cli.rs

pub(crate) enum RiskAction {
    Set(RiskSetArgs),
    Clear(RiskClearArgs),
}

// In Command enum:
/// Set or clear the [facet] on a risk item
Risk {
    #[command(subcommand)]
    action: RiskAction,
},
```

`dispatch()` arm delegates to `run_risk_set`/`run_risk_clear`.
`write_class` classifies as `Write("risk")`.

## 5. Code impact

| File | Change |
|------|--------|
| `src/backlog.rs` | `ALL`: `const` → `pub(crate) const` (one token) |
| `src/facet_write.rs` | Add `FacetField`, `set_facet_mixed`, `apply_set_mixed`; VT-13–VT-17 |
| `src/commands/facet.rs` | Add `RiskSetArgs`, `RiskClearArgs`, `read_kind`, `run_risk_set`, `run_risk_clear`; VT-1–VT-12 |
| `src/commands/cli.rs` | Add `RiskAction` enum, `Risk` variant in `Command`, dispatch arm |
| `src/commands/guard.rs` | `Command::Risk { .. } => Write("risk")` |

Files NOT changed: `estimate.rs`, `value.rs`, all other modules.

## 6. Verification

| VT | Where | Description |
|----|-------|-------------|
| VT-1 | facet.rs | `set` with `--likelihood low --impact medium` writes both to `[facet]` |
| VT-2 | facet.rs | `set` with `--likelihood` only — partial write, impact unchanged |
| VT-3 | facet.rs | `set` with neither axis → error |
| VT-4 | facet.rs | `set` on non-risk item → kind-gate error |
| VT-5 | facet.rs | `set` on non-backlog entity → "no kind field" error |
| VT-6 | facet.rs | `clear` removes `[facet]` table |
| VT-7 | facet.rs | `clear` on absent facet → no-op echo |
| VT-8 | facet.rs | `set` idempotent — same values → no-op echo |
| VT-9 | facet.rs | `set` with `--origin` writes origin string |
| VT-10 | facet.rs | `set` with `--controls A --controls B` writes `["A", "B"]` |
| VT-11 | facet.rs | `set` preserves non-managed facet keys (origin survives likelihood-only set) |
| VT-12 | — | Invalid level token → clap rejection (clap intrinsic; not tested — `RiskLevel::ValueEnum` parse is clap's contract) |
| VT-18 | facet.rs | `set` on risk item with absent `[facet]` table — allocates table and writes fields |
| VT-13 | facet_write.rs | `set_facet_mixed` allocates absent `[facet]` with Str/Arr fields |
| VT-14 | facet_write.rs | `set_facet_mixed` overwrites present Str/Arr managed keys |
| VT-15 | facet_write.rs | `set_facet_mixed` no-op on identical Str values |
| VT-16 | facet_write.rs | `set_facet_mixed` no-op on identical Arr values (element-wise) |
| VT-17 | facet_write.rs | `set_facet_mixed` forward-compat: preserves unknown sibling keys |

## 7. Design decisions

- **D1 — Extend facet_write rather than command-layer TOML manipulation.**
  `facet_write` is the designated edit-preserving TOML mutation surface for
  facets. Adding `FacetField` there keeps the command layer thin and prevents
  `toml_edit` from leaking into command code. The behaviour-preservation gate
  applies at the shared-leaf level.

- **D2 — `RiskLevel` validation via clap's `ValueEnum`, not a pure `validate`
  function.** Unlike `estimate` (which needs custom validation for
  `lower >= 0`, `upper >= lower`, finite checks), risk levels are a closed enum
  with no cross-field constraints. Clap handles parsing and produces clear
  auto-generated help. No separate pure validate module needed.

- **D3 — `--controls` replaces the entire list (declared in help text).**
  Controls are typically 1-5 items; re-specifying the full list on `set` is
  low-friction. An additive `risk controls add/remove` subcommand would be a
  separate improvement. Help text explicitly states the replace semantics.

- **D4 — Both axes partially settable (at-least-one constraint).**
  `exposure()` already treats a missing axis as zero, and partial assessment is
  a legitimate editing state (assess likelihood first, impact later). The
  at-least-one guard prevents accidental empty `set` without adding a
  no-op-in-disguise path.

- **D5 — `FacetField` starts with `Str` and `Arr` only (no `Float`).**
  Risk has no float fields. The existing `set_facet(&[(&str, f64)])` already
  handles the float-only path for estimate/value. Adding a `Float` variant now
  would create parallel vocabulary — add it later if a facet needs it.

- **D6 — `ItemKind::ALL` visibility bump.** `ALL` is in declaration order
  (Issue, Improvement, Chore, Risk, Idea), not priority order. Modules consuming
  it must use `ordinal()` for priority grouping — the `ALL` array is for
  iteration only. The existing `from_prefix` method already iterates `ALL`
  internally and serves as precedent for this pattern being safe.

- **D7 — Kind gate reads `kind` via `toml_edit`, not full `BacklogItem` parse.**
  Cheaper, no need to parse the full entity. The `read_kind` helper iterates
  `ItemKind::ALL` (visibility bumped to `pub(crate)`) so the mapping stays
  single-source.

- **D8 — Empty `--origin` is indistinguishable from cleared.** The scaffold
  seeds `origin = ""`, so setting `--origin ""` is a no-op (same as the seeded
  value). The Str no-op guard handles this transparently — no special case needed.

## 8. Risks & open questions

- **The `ALL` visibility bump is a one-token change** but opens a surface that
  was previously module-private. The field is `const`, so it cannot be mutated;
  it surfaces the declaration-order array for iteration. Minimal risk.
- **No `origin` or `controls` clear-via-set-path.** To clear just origin while
  keeping likelihood/impact, users must re-`set` without `--origin`. Origin is a
  free-text field with no explicit "clear" flag — matching how `estimate set`
  has no partial-clear flag (clear removes the whole facet). Acceptable for the
  first iteration.
