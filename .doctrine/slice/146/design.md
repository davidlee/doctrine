# Design — SL-146: Config coefficient CLI

## Decisions

### D1 — Module structure

```
src/commands/config.rs (new)  — command tier: arg parsing, dispatch, output, toml_edit writes
src/priority/config.rs (amend) — leaf tier: shared read_priority_table, expose clamp functions
src/commands/cli.rs (amend)    — register Command::Config, dispatch match arm
src/commands/mod.rs (amend)    — pub(crate) mod config;
```

ADR-001 boundary: `commands/config.rs` imports `priority::config` (leaf) and `toml_edit`. The leaf never imports the command tier. No new leaf module — the write path is command-tier, matching the `src/tag.rs` precedent (toml_edit called directly, no leaf write API).

**Imports**: `toml_edit::value`, `toml_edit::table`, `toml_edit::DocumentMut`, `toml_edit::Item`.

### D2 — CLI surface

```
doctrine config show   --priority [--json] [-p PATH]
doctrine config get    (--priority | --tag) <key> [--json] [-p PATH]
doctrine config set    (--priority | --tag) <key> <value> [-p PATH]
doctrine config unset  (--priority | --tag) <key> [-p PATH]
```

**`show` takes `--priority` only.** The whole `[priority]` section is always shown.

**`set`/`get`/`unset` require exactly one of `-P`/`--priority` or `-T`/`--tag`** (validated in
the handler, not clap — neither is `required`; the handler bails if both are absent or both
present):

| Flag | Section target | Key shape | Example |
|------|---------------|-----------|---------|
| `-P` / `--priority` | `[priority]` | `subsection.key` (2 segments) | `-P coefficients.value` |
| `-T` / `--tag` | `[priority].tag_coefficients` | bare tag name | `-T "area:cli"` |

`-T` is a convenience shortcut that prepends `tag_coefficients.` to the key before
path validation. `-T "area:cli"` is equivalent to `-P tag_coefficients."area:cli"`.

*(Future: `-K` / `--kind` for `kind_weights` shortcut — same pattern.)*

`-p`/`--path` is the standard doctrine project-root override, wired uniformly across
all verbs (`survey`, `next`, `explain`, `tag`, etc.). It is present on every subcommand
for consistency; `show`/`get` benefit when inspecting a non-current project's config.

### D2a — Path validation

The path parser is shared across `set`, `get`, and `unset`. It validates:

- **Path must have exactly 2 segments** (a 1-segment path like `coefficients` is section-level,
  rejected with `"expected a leaf key (2 dot-separated segments, e.g. coefficients.value)"`).
- **Segment 0** must be one of `coefficients`, `consequence`, `kind_weights`, `tag_coefficients`.
  Unknown subsections bail with `"unknown config subsection '{seg0}'"`.
- **Segment 1**:
  - For `coefficients`: `value` or `risk` map to a `CoefficientsCoeff` variant.
    Any other non-empty string produces `Unknown { subsection: "coefficients", key }`.
    `get` and `unset` proceed on unknown keys (raw value read / removed); `set` bails
    with "cannot set unknown key — add by hand-editing `doctrine.toml` or wait for a
    doctrine update that supports this key" (clamp policy is undetermined for unknown keys).
  - For `consequence`: `dep_coeff` or `ref_coeff` map to a `ConsequenceCoeff` variant;
    unknown keys follow the same `Unknown` pattern.
  - For `kind_weights` / `tag_coefficients`: any non-empty string is accepted (dynamic map keys).

```rust
/// Parsed and validated config path — either a static scalar or a dynamic map entry.
pub(crate) enum ConfigPath {
    CoefficientsCoeff { field: &'static str },
    ConsequenceCoeff { field: &'static str },
    Unknown { subsection: String, key: String },
    KindWeight { kind: String },
    TagCoefficient { tag: String },
}

/// Validate and parse a dotted config path (relative to section). Returns the structured path or an error.
pub(crate) fn parse_config_path(raw: &str) -> anyhow::Result<ConfigPath> { ... }
```

For `set` on map entries (`KindWeight`, `TagCoefficient`), the key is treated as an upsert — the
TOML key string is used verbatim (with TOML quoting rules applied by toml_edit on write).

Arg structs in `commands/config.rs`:

```rust
#[derive(clap::Subcommand)]
pub(crate) enum ConfigAction {
    Show(ConfigShowArgs),
    Set(ConfigSetArgs),
    Get(ConfigGetArgs),
    Unset(ConfigUnsetArgs),
}

#[derive(clap::Args)]
pub(crate) struct ConfigShowArgs {
    /// Target the [priority] section
    #[arg(short = 'P', long)] pub(crate) priority: bool,
    #[arg(long)] pub(crate) json: bool,
    /// Project root (default: auto-detect)
    #[arg(short = 'p', long)] pub(crate) path: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ConfigSetArgs {
    /// Target the [priority] section (subsection.key path)
    #[arg(short = 'P', long)] pub(crate) priority: bool,
    /// Target [priority].tag_coefficients (bare tag name)
    #[arg(short = 'T', long)] pub(crate) tag: bool,
    pub(crate) key: String,
    pub(crate) value: f64,
    #[arg(short = 'p', long)] pub(crate) path: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ConfigGetArgs {
    #[arg(short = 'P', long)] pub(crate) priority: bool,
    #[arg(short = 'T', long)] pub(crate) tag: bool,
    pub(crate) key: String,
    #[arg(long)] pub(crate) json: bool,
    #[arg(short = 'p', long)] pub(crate) path: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ConfigUnsetArgs {
    #[arg(short = 'P', long)] pub(crate) priority: bool,
    #[arg(short = 'T', long)] pub(crate) tag: bool,
    pub(crate) key: String,
    #[arg(short = 'p', long)] pub(crate) path: Option<PathBuf>,
}
```

### D3 — `config show` output format

Flattened dotted keys (relative to section), subsection header comments, inline annotations.

```
# coefficients
coefficients.value = 2.0
coefficients.risk = 1000000000.0   # clamped from 99e9

# consequence
consequence.dep_coeff = 0.5        # default
consequence.ref_coeff = 1.0
```

**Key enumeration** — the subsections and their leaf keys:

| Subsection | Keys |
|------------|------|
| `coefficients` | `value`, `risk` |
| `consequence` | `dep_coeff`, `ref_coeff` |
| `kind_weights` | iterate `effective.kind_weights` keys (from `load()`) |
| `tag_coefficients` | iterate `effective.tag_coefficients` keys (from `load()`) |

Rules:
- **Subsection headers** (`# coefficients`, `# consequence`, `# kind_weights`, `# tag_coefficients`) as whole-line comments grouping the TOML sub-structs. **Skip a subsection entirely if it has no keys** (empty kind_weights or tag_coefficients).
- **Quoted keys**: when a map key contains characters requiring TOML quoting (`:`, `.`, `#`, whitespace, `[`, `{`), the flattened output surrounds the key segment with double quotes, e.g. `tag_coefficients."area:risk" = 2.0`.
- **Inline annotations** aligned: compute the longest output key width, set annotation column to `max(56, max_key_width + 4)`. `# default` for absent keys that fell back to defaults; `# clamped from N` when the file value differs post-clamp.
- **Blank lines** between subsections; no blank line within a subsection.
- **TTY output**: subsection headers and inline annotations get colorized (dim/ANSI) to distinguish from keys and values.

**`--json` output**: flat object mapping each leaf key path to `{"effective": f64, "raw": f64|null, "annotation": "default"|"clamped"|null}`. Empty maps (kind_weights, tag_coefficients) produce no keys:

```json
{
  "coefficients.value": {"effective": 2.0, "raw": 2.0, "annotation": null},
  "coefficients.risk": {"effective": 1000000000.0, "raw": 99999999999.0, "annotation": "clamped"},
  "consequence.dep_coeff": {"effective": 0.5, "raw": null, "annotation": "default"}
}
```

### D4 — `config get` output

```
$ doctrine config get --priority coefficients.value
2.0
```

- Prints the effective (clamped & defaulted) value on stdout, bare.
- Optionally annotates `# default` / `# clamped from N` inline when the effective differs from the file value.
- For map entries (`kind_weights.X`, `tag_coefficients.X`): if the key is absent, returns `1.0` (the identity default per `kind_weight()` / `tag_coeff()`), annotated `# default`.
- For unknown static keys (matched by `ConfigPath::Unknown`): reads the raw f64 from
  the TOML table. If present, prints the bare value (no clamp annotation — the clamp
  policy is undetermined for unknown keys). If absent, prints nothing and exits non-zero
  with "key not found."
- `--json`: `{"path": "coefficients.value", "effective": 2.0, "raw": 2.0, "annotation": null}`.

### D5 — `config set` behaviour

1. Validate path via `parse_config_path` — returns `ConfigPath` or error.
2. Parse `doctrine.toml` as `DocumentMut` (hard error on malformed TOML).
3. **Clamp the user value**:
   - `ConsequenceCoeff { field: "dep_coeff" }` → `clamp_dep(value)`, fallback `0.5`.
   - `CoefficientsCoeff { field: "value" }` → `clamp_general(value, 1.0)`.
   - `CoefficientsCoeff { field: "risk" }` → `clamp_general(value, 2.0)`.
   - `ConsequenceCoeff { field: "ref_coeff" }` → `clamp_general(value, 1.0)`.
   - `KindWeight { .. }` / `TagCoefficient { .. }` → `clamp_general(value, 1.0)`.
   Clamp detection: compare `clamped != value` — see D7 7b for why this works
   for non-finite inputs.
4. **Walk path** via `entry().or_insert(table())` on the `DocumentMut` — creates intermediate tables
   if absent (including root `[priority]`, safe per CHR-019). For map entries, the final segment is
   the map key string.
5. **No-op guard**: before mutation, read the existing value from the `DocumentMut` via
   `item.as_value().and_then(|v| v.as_float())`. If the existing f64 == clamped, skip the write.
   A node that exists but is not a float (malformed prior edit) is treated as absent — the guard
   returns `false` and the write proceeds, overwriting the malformed entry.
6. Write `DocumentMut` back, echo `"config set: path = written_value"`
   (or `"config set: path = written_value (clamped from user_value)"` when clamping changed the value).

`set` does not support `--json` output — it is an imperative mutating verb, and its
exit code is the script interface (cf. `doctrine estimate set`, `doctrine tag add`).
The confirmation message is for the operator.

### D6 — `config unset` behaviour

1. Validate path via `parse_config_path` — returns `ConfigPath` or error (the parser already
   rejects section-level paths, so D2a covers step 4 below).
2. Parse `doctrine.toml` as `DocumentMut` (hard error on malformed TOML).
3. Walk path to parent table. If any intermediate segment absent → idempotent no-op (already absent).
4. Remove the leaf key from its parent table. Keep empty subtable if this was the last key.
5. Write back only if changed. Echo `"config unset: path (was old_value)"` or
   `"config unset: path (already absent)"`.

`unset` does not support `--json` output (same rationale as `set` — imperative verb,
exit code as script interface).

### D7 — `src/priority/config.rs` changes

The current `load()` inlines `read_to_string` + `text.parse()` in a single function.
The following changes extract shared helpers from that inline logic and refactor
`load()` to delegate to them, without changing its tolerant signature.

**7a. NEW — extract shared parse function**

```rust
/// Read doctrine.toml and extract the raw `[priority]` table.
/// Returns `Ok(None)` when the file is missing (NotFound only); hard error on
/// malformed TOML or other IO errors — callers choose whether to propagate.
pub(crate) fn read_priority_table(root: &Path) -> anyhow::Result<Option<toml::Table>> {
    let path = root.join("doctrine.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let value: toml::Value = text.parse()?;
    Ok(value.get("priority").and_then(|v| v.as_table()).cloned())
}
```

After extraction, `load(root)` will call `read_priority_table` internally (see 7c).
Config commands call `read_priority_table` directly — malformed TOML bubbles up
through `anyhow::Result`.

**7b. Clamp function visibility**

`clamp_general` and `clamp_dep` promoted to `pub(crate)` — the config commands use them to clamp user values before writing.

**Clamp detection**: the command compares `clamped != value` post-clamping and flags
a clamp when they differ. The clamp functions replace non-finite inputs with finite
fallbacks (e.g. `NaN` → field default, `Inf` → `COEFF_MAX`), so `clamped` is always
finite while `value` may be non-finite. Per IEEE 754, a finite f64 ≠ NaN is always
true (NaN ≠ everything, including itself), and finite ≠ Infinity is always true —
so a single `clamped != value` correctly detects clamping for all non-finite inputs
without a separate sentinel. No separate `(f64, bool)` return needed.

**7c. NEW — extract `load_from_table` helper**

To avoid double-reading the file in `config show`, extract the deserialise+clamp
step from the current inline `load()` into a `pub(crate)` function:

```rust
/// Deserialise a raw `[priority]` table into a clamped `PriorityConfig`.
/// Tolerant: deserialisation failure → defaults.
pub(crate) fn load_from_table(table: Option<toml::Table>) -> PriorityConfig {
    let Some(table) = table else { return PriorityConfig::default() };
    let cfg: PriorityConfig = match toml::Value::Table(table).try_into() {
        Ok(c) => c,
        Err(_) => PriorityConfig::default(),
    };
    clamp(cfg)
}
```

`load(root)` is REFACTORED to chain `read_priority_table(root)` → `load_from_table`,
retaining its tolerant signature. `config show` calls both directly:
`read_priority_table(root)?` for the raw table, `load_from_table(raw)` for effective.

**7d. No signature change to `load()`**

`load(root) -> PriorityConfig` stays tolerant. The scoring engine never hard-errors on config.

### D8 — `src/commands/cli.rs` changes

Add to `Command` enum:

```rust
/// Inspect and modify project-wide doctrine.toml config.
Config {
    #[command(subcommand)]
    action: crate::commands::config::ConfigAction,
},
```

Dispatch:

```rust
Command::Config { action } => match action {
    ConfigAction::Show(args) => crate::commands::config::run_config_show(&args),
    ConfigAction::Set(args) => crate::commands::config::run_config_set(&args),
    ConfigAction::Get(args) => crate::commands::config::run_config_get(&args),
    ConfigAction::Unset(args) => crate::commands::config::run_config_unset(&args),
},
```

### D9 — `src/commands/mod.rs` change

```rust
pub(crate) mod config;
```

No changes to `src/main.rs` — `mod commands` and `cli::dispatch` already handle all Command variants.

## Data flow — `config show`

```
config show
  │
  ├─ root::find(path)
  ├─ priority::config::read_priority_table(root)? → Option<toml::Table>  (one read+parse)
  └─ priority::config::load_from_table(&raw_table) → PriorityConfig     (deserialise+clamp, no second read)
       │
       ▼
  Diff raw vs effective per key:
    - key present in file, value == effective → no annotation
    - key present in file, value ≠ effective → "# clamped from N"
    - key absent in file                       → "# default"

  Raw value extraction helper:
    fn raw_f64_at_path(table: &toml::Table, path_segments: &[&str]) -> Option<f64>
    Walks the TOML table by path segments, returns the f64 at the leaf (or None if absent/malformed).
```

## Data flow — `config get`

```
config get key
  │
  ├─ parse_config_path(key)? → ConfigPath
  ├─ root::find(path)
  ├─ read_priority_table(root)? → Option<toml::Table>
  ├─ load_from_table(&raw)      → PriorityConfig (effective)
  │
  ▼
  For static keys (CoefficientsCoeff, ConsequenceCoeff):
    - extract raw f64 from raw_table (None if absent)
    - lookup effective value from PriorityConfig field
  For map keys (KindWeight, TagCoefficient):
    - raw: raw_f64_at_path(raw_table, ["kind_weights", kind])
    - effective: cfg.kind_weight(&kind) or cfg.tag_coeff(&tag)
  For unknown keys (ConfigPath::Unknown):
    - extract raw f64 from raw_table (prints bare value if present, "key not found" if absent)
    - no effective/clamp lookup (policy is undetermined)
  Print effective value, annotate if raw absent or clamped
```

## Edge cases

| Case | Behaviour |
|------|-----------|
| `doctrine.toml` missing | `read_priority_table` returns `Ok(None)` (read fails → None) |
| `doctrine.toml` malformed | `bail!("doctrine.toml: failed to parse: {e}")` |
| `[priority]` section absent | Raw table is `None`; all values shown as `# default` |
| `[priority]` partial (some subsections absent) | Present keys shown with file values; absent keys `# default` |
| `set` value == existing file value | No-op; echo "unchanged" |
| `set` value clamps | Write clamped value; echo "clamped to Y from X" |
| `set` NaN / Inf / negative / > COEFF_MAX | Clamp and write; echo clamped |
| `set` dep_coeff > 1 | Clamp to 1.0 |
| `set` dep_coeff ≤ 0 | Clamp to 0.0 |
| `set` ref_coeff < 0 | Clamp to 0.0 (ADR-015: non-negative) |
| `set` ref_coeff > COEFF_MAX | Clamp to COEFF_MAX |
| `set`/`get`/`unset` unknown subsection | Error: "unknown config subsection 'nonesuch'" |
| `set` unknown static key (via `ConfigPath::Unknown`) | Error: "cannot set unknown key — add by hand-editing doctrine.toml" |
| `get` unknown static key (via `ConfigPath::Unknown`) | Raw f64 from file if present; "key not found" if absent |
| `unset` unknown static key (via `ConfigPath::Unknown`) | Removes key from file; same as any leaf key |
| `set`/`get`/`unset` 1-segment path (no leaf) | Error: "expected a leaf key (2 dot-separated segments, e.g. coefficients.value)" |
| `get` absent scalar key | Returns effective default; `# default` annotation |
| `get` absent map key (`kind_weights.NO_SUCH`) | Returns `1.0` (identity default, matches `kind_weight()`); `# default` annotation |
| `get` present key | Returns effective value (raw == effective if no clamp) |
| `unset` existing leaf key | Removes key; echo "was N" |
| `unset` absent/tpyo leaf key | No-op; echo "already absent" (idempotent — intentional, the set-absent path is harmless) |
| `set`/`get`/`unset` without `--priority` or `--tag` | Error: "must specify --priority (-P) or --tag (-T)" |
| `set`/`get`/`unset` with both `--priority` and `--tag` | Error: "--priority (-P) and --tag (-T) are mutually exclusive" |
| `-T` key with tag name containing `.` | Prepend `tag_coefficients.` then parse — dots in the tag name are fine (e.g. `-T "area:cli"` → path `tag_coefficients.area:cli`) |
| `show --json` | Structured JSON with `effective`/`raw`/`annotation` per key |
| `get --json` | `{"path": "...", "effective": ..., "raw": ..., "annotation": ...}` |

## Non-goals (reaffirmed)

- No config for non-priority sections (`[dispatch]`, `[conduct]`, etc.) — `--priority` is the only section flag
- No batch/multi-key operations
- No `--dry-run` flag
- No diff/history tracking
- No `--all` flag for full-file display
- No validation beyond existing clamp policy

## Test plan

Unit tests in `commands/config.rs` (module-level, test fixtures via tempfiles):
- `parse_config_path` rejects unknown subsection `unknown.key`
- `parse_config_path` accepts unknown static key `coefficients.nonesuch` → `ConfigPath::Unknown`
- `parse_config_path` accepts unknown static key `consequence.unknown` → `ConfigPath::Unknown`
- `parse_config_path` rejects 1-segment path `coefficients`
- `parse_config_path` accepts `kind_weights` / `tag_coefficients` with any non-empty key
- `set`/`get`/`unset` with `-T` (tag shortcut), verify it targets `[priority].tag_coefficients`
- `set`/`get`/`unset` with `-P` and `-T` both → error
- `set`/`get`/`unset` with neither `-P` nor `-T` → error
- `show` with no `[priority]` → all defaults annotated
- `show` with full `[priority]` → exact file values, no annotations
- `show` with partial `[priority]` → present keys bare, absent keys `# default`
- `show` with clamped value → `# clamped from N` annotation
- `show` skips empty `kind_weights` / `tag_coefficients` subsections
- `show` quotes keys containing `:` (e.g. `area:risk`)
- `show --json` → correct JSON shape
- `set` simple scalar write, verify file contents
- `set` map entry write (kind_weights, tag_coefficients), verify file
- `set` coefficients.value 99e9 → clamped to COEFF_MAX, verify file + clamp message in output
- `set` value clamps, verify file + output message
- `set` dep_coeff uses `clamp_dep` (not general)
- `set` idempotent → file unchanged
- `set` creates missing `[priority]` → file has the section
- `set` overwrites malformed (non-float) prior entry
- `set` on `ConfigPath::Unknown` → error: "cannot set unknown key"
- `get` present scalar key → bare value
- `get` absent scalar key → bare default + `# default`
- `get` absent map key (`kind_weights.NO_SUCH`) → `1.0` + `# default`
- `get` on `ConfigPath::Unknown` with file value → reads raw f64 from file, no clamp annotation
- `get` on `ConfigPath::Unknown` with absent key → "key not found"
- `get --json` → correct JSON shape
- `unset` existing scalar key → removed from file
- `unset` existing map key → removed from file
- `unset` absent key → no-op, "already absent"
- `unset` on `ConfigPath::Unknown` → removes key from file, same as any leaf key
- `unset` removes the LAST key from a subsection → empty subsection header `[priority.coefficients]` remains in file output
- `unset` section path → error via `parse_config_path`
- Malformed `doctrine.toml` → hard error (all subcommands)

Unit tests in `priority/config.rs` (existing + one new):
- `read_priority_table` returns `Ok(None)` when file missing
- `read_priority_table` returns `Ok(Some(table))` when `[priority]` present
- `read_priority_table` errors on malformed TOML
- `load()` still tolerant (existing tests unchanged)

Integration/behaviour tests in `tests/`:
- `doctrine config show --priority` on the real `doctrine.toml` — golden output assertion

Scope verification criterion 11 ("Existing `survey`/`next`/`explain` output uses the new config
without restart") is inherited from `PriorityConfig::load(root)` — the config module writes
`doctrine.toml`, and every subsequent scoring-engine invocation reads it fresh via `load()`.
No new test needed in this module.
