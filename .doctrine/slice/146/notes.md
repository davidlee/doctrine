# Notes — SL-146: Config coefficient CLI

## Created

2026-06-23 by pi agent per user request "RFC-002 - looky here, i wanna slice a cli interface for setting project-wide toml config coefficients"

## Done

- Created **IMP-161** (improvement) — `Project-wide TOML config coefficient CLI — get/set priority coefficients, kind_weights, tag_coefficients`
- Created **SL-146** (slice) — `Config coefficient CLI — doctrine config get/set for priority coefficients, kind_weights, tag_coefficients`
- Linked IMP-161 → SL-146 (`slices`)
- Linked SL-146 → RFC-002 (`related`), ADR-015 (`governed_by`), SPEC-001 (`specs`)
- Soft-sequenced SL-146 after SL-136, SL-133, SL-134 (`after`)
- Advanced SL-146 lifecycle: `proposed → design`
- **Inquisition RV-147** — adversarial design review; 9 findings raised, all resolved. Design penance applied (see below).

## Research sources consulted

### Primary documents (the program & policy)

| Document | Key content | Relevance |
|----------|------------|-----------|
| [RFC-002](/.doctrine/state/dispatch/coordination-136/.doctrine/rfc/002/rfc-002.md) | Consumption surfaces program thesis — estimate, value, and scoring outward pipeline | The overarching program that this slice completes. Names IMP-134 (tagging→SL-136) and IMP-118 (scoring→SL-133) as dependencies; config CLI is the final un-surfaced read/write path |
| [RFC-002 TOML](/.doctrine/state/dispatch/coordination-136/.doctrine/rfc/002/rfc-002.toml) | Relations: related to SL-132/SL-133/SL-134/SL-135/SL-136/SL-137/SL-138 | Shows the slice ecosystem under this program; SL-146 fits as the next logical leaf |
| [ADR-015](https://github.com/davidlee/doctrine/blob/main/.doctrine/adr/015) | Multi-dimensional priority scoring policy | The durable policy that `[priority] coefficients/kind_weights/tag_coefficients` descend from. Governs this slice's config shape — the CLI writes what ADR-015 specifies |
| [SL-133 scope](/.doctrine/slice/133/slice-133.md) | Multi-dimensional priority scoring scope + terrain | Shows the full `PriorityConfig` schema, the two-pass formula, the pure/impure split. Config parsing is the leaf `src/priority/config.rs`; this slice's CLI writes the file that leaf reads |

### Code files (terrain survey)

| File | Key finding |
|------|------------|
| `src/priority/config.rs` | THE config reader — `PriorityConfig` struct, `load(root)` reads `doctrine.toml`'s `[priority]`, clamps silently. Uses its own reader, NOT `dtoml::parse`. Has `coefficients`, `kind_weights: BTreeMap`, `tag_coefficients: BTreeMap`, `consequence`. All serde defaulted. |
| `src/dtoml.rs` | THE shared `doctrine.toml` parser — the union of `[conduct]`, `[verification]`, `[estimation]`, `[value]`, `[dispatch]`. DOES NOT include `[priority]` — priority uses its own leaf reader per ADR-001. Important: the shared reader intentionally excludes priority. |
| `src/tag.rs` | `apply_tags_set` — root-level edit-preserving write via `toml_edit`. The direct precedent for how we'll edit `doctrine.toml`. Uses `doc.as_table_mut().insert()`. |
| `src/dispatch_config.rs` | `DispatchConfig` — another per-section config reader. Shows the pattern: `#[serde(default)]` struct, `#[serde(rename = "kebab-case")]` fields, `Default` impl. |
| `src/commands/facet.rs` | Precedent for a small CLI verb (`estimate set/clear`, `value set/clear`). Similar shape: one command module, subcommands for set/get/clear. |
| `src/catalog/scan.rs` | `read_facets` — reads the `[facet]` table from entity TOML files. Not directly relevant but shows the scan → facet projection pattern. |
| `src/commands/cli.rs` | Where new `Command::Config` would register. Shows the enum-based subcommand dispatch. |
| `src/main.rs` | Top-level subcommand wiring. Mod::20 shows `mod dispatch_config;` as a module declaration precedent. |

### Memories (gotchas & patterns)

| Memory | Key insight |
|--------|------------|
| [toml-edit root-insert-above-headers](mem_019ee9fd51d87aa38a2dfb31ad6c4eec) | **Critical**: `doc.as_table_mut().insert("key", value)` on a root key renders at the TOP of the document above all `[table]` headers. This is structural to TOML, not a toml_edit bug. For `doctrine.toml`, inserting `[priority]` at root level is safe — it lands before `[dispatch]`, which is fine for the config file. Not a corruption risk. |
| [Edit-preserving authored-TOML status transition](mem_019ea4e4f03c72e1b9a0ef55dcde956d) | The house pattern: read text → `DocumentMut::parse` → mutate → `fs::write(path, doc.to_string())`. Never reserialise. No-op guard if value unchanged. This is the exact pattern we reuse for `config set`. Note: the F-1 refuse (don't insert absent keys) was specific to scaffolded entity TOMLs — `doctrine.toml` has no scaffold guarantee so we use the more permissive CHR-019 pattern. |
| [Clippy bool/arg ceilings](mem_019e985028947ef2ad86d43997214aca) | Adding a bool param can trip `too_many_arguments` (>7 params) and `fn_params_excessive_bools` (>3 bools). Use an args struct like `RecordArgs`. For `config set`, the handler is simple enough (<7 params) but worth keeping in mind. |
| [TOML error classification](mem_019ea699e96c7613886480bf12992e5e) | `toml` crate has no stable error-kind enum — classify errors by span+message. Not directly relevant (we're writing, not parsing errors) but good to know for tests. |
| [Conformance surface parity](mem_019ea78bbab8737280d32bed6cbfa58c) | When adding a new CLI verb with uniform output surfaces, test ALL surfaces (human table + JSON). The `config show` command should have conformance tests for TOML output and JSON output. |

### Design precedent files

| File | What it shows |
|------|-------------|
| `doctrine.toml` (root) | Current state of `[priority.coefficients]` and `[priority.consequence]` — the values our CLI will read and write. No `kind_weights` or `tag_coefficients` section yet. |
| [SL-136 scope](/.doctrine/slice/136/slice-136.md) | Showed the `toml_edit` root-insert pattern was safe, and the `apply_tags_set` precedent. Also showed the REV obligation pattern for governance changes — not needed here since we're not changing spec. |
| ADR-001 (module layering) | Leaf ← engine ← command: `priority::config` is a leaf; `commands::config` is command-tier. The command calls `priority::config::load()` (already public) for reading, and writes `doctrine.toml` via toml_edit directly (no leaf write path exists yet). |

## Design decisions embedded in the scope

1. **Dot-separated path syntax**: `priority.coefficients.value` — mirrors TOML key path notation, idiomatic.
2. **Silent clamp (no hard error)**: Matches `PriorityConfig::load()` policy — write the clamped value, tell the user what was written. No rejection of out-of-range.
3. **Priority-only scope**: The `config` verb namespace is designed for future extension but only `[priority]` is wired. Other sections deferred.
4. **toml_edit for writes**: Reuses the SL-136/CHR-019 pattern. Inserting a missing `[priority]` section is safe per the root-inserts-above-headers guarantee.

## Open questions (for the design phase)

- Path syntax: dot-separated (`priority.coefficients.value`) is the initial choice — confirm during design.
- `config show` format: human TOML-like? JSON? Table with raw+effective columns? Scope says human-friendly TOML-like with both raw and effective values.
- `config show` scope: `[priority]` only for now; `--all` deferred.

## Related backlog items

- **IMP-161** (this slice's improvement parent): `/workspace/doctrine/.doctrine/backlog/improvement/161/`
- **IMP-133** (CLI UX review — still open): Could surface findings that inform` config` verb help text and discoverability
- **IMP-134** (tagging — SL-136 done): Built the `toml_edit` patterns we reuse
- **IMP-118** (priority scoring — SL-133 done): Defined the config schema we surface

## Commit status

Uncommitted. Created files:
- `.doctrine/backlog/improvement/161/backlog-161.toml`
- `.doctrine/backlog/improvement/161/backlog-161.md`
- `.doctrine/slice/146/slice-146.toml`
- `.doctrine/slice/146/slice-146.md`
- `.doctrine/slice/146/notes.md`

Also modified: RFC-002's `[[relation]]` table (gained an SL-146 entry via `doctrine link`).

**Gate**: No code was changed — these are only authored artefact files. `just check` not needed.

## Next step

Hand off to `/plan` — SL-146 design is locked and inquisited. The design phase completed:
1. Confirmed the path syntax (dot-separated, 2 segments)
2. Detailed the `config show` output format (flattened dotted keys, annotations, --json)
3. Designed the `toml_edit` write path (section-insert via `entry().or_insert()`, CHR-019 safe)
4. Planned the `commands/config.rs` module structure (four subcommands, shared path parser)
5. Locked down edge-case table and test plan
6. **Inquisition RV-147** applied 9 corrections (see below)

## Inquisition RV-147 — applied penance

2026-06-23 — adversarial design review against ADR-015, SPEC-001, scope, and terrain.
9 findings, all verified terminal. Corrections applied to `design.md`:

| Finding | Change |
|---------|--------|
| F-1 (blocker) | D7 `read_priority_table`/`load_from_table` now marked NEW — described as extractions from existing inline `load()`, not existing code |
| F-2 (major) | D2a path validation relaxed: unknown static keys → `ConfigPath::Unknown` variant. `get`/`unset` proceed; `set` bails. Scope extensibility preserved. |
| F-3 (major) | Edge-case table: added `set ref_coeff < 0` and `set ref_coeff > COEFF_MAX` rows. Test plan: added `set coefficients.value 99e9` test. |
| F-4 (minor) | Test plan: added note that scope verification 11 (survey/next/explain re-reads config) is inherited from `PriorityConfig::load()`. |
| F-5 (minor) | D7 7b clamp detection expanded with IEEE 754 explanation. D5 step 3 cross-references D7 7b. |
| F-6 (minor) | D5/D6: documented `--json` asymmetry rationale (imperative verbs, exit code as script interface, cf. `estimate set`/`tag add`). |
| F-7 (nit) | D2: added `--path` justification note (standard doctrine project-root override, wired uniformly). |
| F-8 (nit) | Test plan: added `unset` removes the LAST key from a subsection → empty subsection header remains test. |
| F-9 (minor) | Integration test: removed "optional" qualifier. Golden `config show --priority` test is required. |

Design is now doctrinally clean. Ready for `/plan`.
