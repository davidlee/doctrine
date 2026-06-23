# Plan rationale — SL-146: Config coefficient CLI

## Why these phases, in this order

### Dependency chain

```
PHASE-01 (foundation)
  └─ PHASE-02 (read surface: show + get)
       └─ PHASE-03 (write surface: set)
            └─ PHASE-04 (delete surface: unset)
```

Each phase builds on the previous. The foundation phase is mandatory before any
subcommand can ship — it delivers `read_priority_table`, `load_from_table`,
clamp visibility, `parse_config_path`, CLI registration, and arg structs.

### PHASE-01: Foundation — priority/config refactor + module skeleton

The design's D7 refactors `src/priority/config.rs` to extract two shared
helpers (`read_priority_table`, `load_from_table`) from `load()`'s inline logic.
This is the critical enabler: every subcommand needs to read `doctrine.toml`
and resolve effective values. Without extraction, each subcommand would
re-implement `load()`'s internal loop, creating duplication across four commands.

`clamp_general` and `clamp_dep` are promoted to `pub(crate)` — the `set`
command calls them to clamp user values before writing. The design's D7b
confirms a simple `clamped != value` comparison correctly detects clamping
for all inputs (NaN, Inf, finite).

`parse_config_path` is the shared path validator for set/get/unset. It
validates segment count, known subsections, and classifies static vs dynamic
keys. Shipping this early (with tests) gives all three subsequent phases a
single, well-tested validation surface — no per-subcommand path parsing.

The CLI registration (D8/D9) adds `Command::Config` to the enum and
`mod config` to `commands/mod.rs` with stub handlers that compile but don't
execute. This isolates the boilerplate from the subcommand logic.

### PHASE-02: config show + config get — read-only surface

Both `show` and `get` are pure read operations. They share the same data
flow: read doctrine.toml once, extract raw table, compute effective via
`load_from_table`, diff raw vs effective for annotations. No mutation, no
toml_edit. This makes them the lowest-risk phase to ship first after the
foundation.

Grouping them together avoids the overhead of setting up the read pipeline
twice. The output formatting for `show` (subsection headers, annotations,
key quoting, JSON) is the bulk of the work; `get` is a thin wrapper that
reuses the same resolution logic for a single key.

`show` is also a useful debugging tool for the phases that follow — operators
can inspect current config before/after `set` and `unset` operations.

### PHASE-03: config set — write surface

The most complex single subcommand. Introduces toml_edit writes:
`DocumentMut::parse`, `entry().or_insert()` for missing sections, value
clamping before write, no-op guard (compare existing f64), and
edit-preserving serialisation back to disk.

The SL-136 `apply_tags_set` precedent proves the toml_edit pattern works for
root-level section insertion (`entry().or_insert()`) and edit-preserving
round-trips. CHR-019 / RV-129 confirmed this is safe for `[priority]`
creation.

Clamp logic reuses the now-`pub(crate)` `clamp_general`/`clamp_dep` from
PHASE-01, dispatched by `ConfigPath` variant per D5 step 3. The no-op guard
(D5 step 5) prevents unnecessary file writes and preserves mtime semantics.

Error surface: unknown static keys are refused for `set` (design D5 edge
case table) because the engine has no clamp policy for unknown keys — the
operator must hand-edit.

### PHASE-04: config unset — delete surface

A smaller mutation operation than `set`. Walks the DocumentMut to the parent
table, removes the leaf key, handles the "already absent" idempotent case.
Reuses the toml_edit infrastructure proven in PHASE-03.

The design D6 edge case table confirms that removing the last key from a
subsection leaves the empty table header (standard toml_edit behaviour) —
acceptable because `load()` tolerates empty tables.

`unset` on `ConfigPath::Unknown` is supported (unlike `set`) — removing an
unknown key is safe because no clamp policy is needed for deletion. The
design's edge case table explicitly allows this.

## Why show and get are grouped, but set and unset are separated

- **show + get**: both read-only, zero file mutation risk. Can be tested
  extensively on arbitrary doctrine.toml fixtures without side effects.
- **set**: the highest-complexity subcommand (clamping, upsert, no-op guard,
  malformed overwrite). Deserves its own phase to isolate the toml_edit
  write path from the simpler delete path.
- **unset**: depends on the toml_edit proficiency gained in PHASE-03 but
  is otherwise independent of `set`'s clamping logic. A natural capstone.

## Verification approach

Each phase verifies its own criteria before the next begins. Existing
`priority/config.rs` tests must pass unchanged at every phase boundary
(D7d guarantee: `load()` signature stays tolerant).

Tests use tempfile fixtures for unit tests (no mutable impact on the
project's own `doctrine.toml`). An E2E golden test in PHASE-02 captures
the real project config output.
