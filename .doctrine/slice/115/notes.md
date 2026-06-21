# Notes SL-115: Decompose main.rs: relocate orphan runners, extract cli arg modules

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

---

## 2026-06-20 — Plan complete, gated on SL-129

### Done

- **Design** authored, committed, and confirmed through 3 codex adversarial rounds (design §4/§5/§6 → round 2; §4 fold → round 3; exhaustive sweep → round 4).
- **Plan** authored: `plan.toml` + `plan.md`. 4 phases, sequencing forced by reachability (shells out before the match follows).
- **Phase sheets** materialised under `.doctrine/state/slice/115/phases/` — all 4 phases have their TOML + MD sheets.

### Commit ladder

```
92f31ac9 design(SL-115): fold codex round 2 — D1a sideways-call carve-out + V7 parse-regression
79c55de3 design(SL-115): fold codex round 3 — SpecReq→spec (second cycle-former); scaffold plan
50d02ea7 design(SL-115): fold codex round 4 — exhaustive cycle sweep clean; complete plan
```

### Key findings from codex adversarial audit

**B4 (round 2, blocker):** D1 body-relocation mints kind→kind edges the sink proof never covered — `MemoryCommand::Sync` → corpus via `MemoryCommand::run_*`. Nominal routing misroutes close cycles.

**B6 (round 3, blocker):** `SpecReqCommand` → requirement.rs (second cycle-former). `run_req_*` lives in `spec.rs`, not `requirement.rs`.

**Round 4 (exhaustive, clean):** Full per-kind table, `MemoryCommand::Sync`→corpus is the sole cycle-former, no third. Two factual nits fixed (Export mislabel, catalog orphans).

### Root fix: D1a rule

Route a kind's dispatch to the module its `run_*` body actually calls, not the nominal kind name. Body relocation from inert `main` into a command-tier module mints new production edges; nominal misroutes close cycles.

Two known instances:
- `MemoryCommand::Sync` → corpus → stays in `commands/` sink shell (PHASE-04), NOT `memory.rs`
- `SpecReqCommand` → spec.rs (own-module, zero edge), NOT `requirement.rs`

### V7 strengthened

`--help` snapshot + parse-regression covering `conflicts_with`, `value_parser`, `value_delimiter`, `requires` — these catch what a `--help` diff cannot.

### Per-batch gate (PHASE-03)

Gate runs after EACH domain batch in PHASE-03. Any tangle growth past `[tangle_baseline] command = 120` HALTS the batch for restructure — never an auto-accept.

### Gate: SL-129 must land first

PHASE-01 EN-2 gates on SL-129 landing first (`after` edge, R1). SL-129 consolidates `entity::id_path`; relocating the now-thinner resolvers afterwards keeps its 93-site inventory valid. **Do not flip SL-115 to ready/execute until SL-129 is in.**

When SL-129 is in: `/phase-plan PHASE-01` → `/execute`.

### Verification gate

No code moved yet (design + plan phase only). `just check` has not changed since last design commit. PHASE-01 builds the proof harness before any relocation.

### Durable items for memory

- **D1a rule** (dispatch routing by actual body calls, not nominal kind) — should be recorded as a `pattern` (or `design`?) memory for the dispatch layer.

---

## 2026-06-21 — PHASE-02 complete

### Done

- 5 remaining command modules populated: `commands/inspect.rs`, `commands/relation.rs`,
  `commands/dep_seq.rs`, `commands/supersede.rs`, `commands/validate.rs`
- Match arms in `main.rs` updated to qualified `commands::<module>::run_*()` paths
- All moved test modules pass in new homes with only `use super::*` path fixups
- `main.rs` reduced from ~7264 → ~5247 lines (~2100 lines removed)
- Verification: 2142 tests pass, `cargo clippy --bin doctrine` zero-warn, fmt clean
- PHASE-01 verification net (21 tests) — all pass; `tests/e2e_*` goldens untouched

### Notes

- `run_inspect` has no tests (golden-covered via e2e); its class-test reference in
  write_class_tests stayed in main.rs (uses `Commands::Inspect` variant, not the fn)
- All relocated modules carry `//! SL-129: uses entity::id_path` breadcrumb on line 2
- `commands/mod.rs` has `pub mod` for all 9 modules (dep_seq, facet, guard, inspect,
  map, relation, serve, supersede, validate)
- Work is on disk but UNCOMMITTED — 10 files modified, +1619/-1524 lines

---

## 2026-06-21 — Layering gate fix (non-module extractor target)

### Finding

`cargo test --test architecture_layering tests::architecture_layering_gate` failed
with `Unclassified("Command")`.

Root cause: `src/commands/guard.rs` carries `use crate::Command;`. The edge
extractor treats every `crate::<segment>` as a module reference, producing edge
`commands -> Command`. But `Command` is a private `enum` in `main.rs` — it is not
a module, and `main.rs` is deliberately excluded from `discover_units` (binary
entrypoint). The layering map has no entry for it, so `check` flags it as
unclassified.

### Fix

Filtered edges in `architecture_layering_gate` to drop targets that do not
correspond to an actual source module (`src/<name>.rs`, `src/<name>/mod.rs`,
or `src/<name>/<name>.rs` must exist). This is the same criterion `discover_units`
uses.

Reverted an earlier approach that filtered inside `extract_edges` itself — that
broke 5 synthetic unit tests that create temp dirs without the target module
files. The filter lives in the gate test only, where the real `src/` tree is
present.

### Verification

- `tests::architecture_layering_gate` passes
- All 18 layering tests pass (17 pass + 1 ignored)
- `cargo clippy` (no `--all-targets`) zero-warn
- `Command` is the **only** non-module target in the entire 394-edge graph (confirmed
  via `dump_real_graph`)

### Follow-up for audit/close

- The extractor cannot distinguish `crate::module_name` from `crate::TypeFromMain` —
  this limitation is latent for any crate-root type accessed across a module
  boundary. The filter-by-existence heuristic is correct for the current codebase.
- Architecturally cleaner: move `Command` to its own module (`src/command.rs` or
  `src/commands/command.rs`). This would eliminate the false positive at its source
  rather than filtering it. Consider as a future slice or a close-out
  recommendation.

---

## 2026-06-21 — PHASE-03 complete

### Done

- 23 kind enums + dispatch bodies relocated from `main.rs` to own-module or
  `commands/` shell, behind the uniform `pub(crate) fn dispatch(cmd, color)`
- 5 domain batches, each standalone gate-green
- `main.rs` reduced from ~5247 → ~2086 lines
- Only `Command` (top-level) and `ExportCommand` (top-level verb) enums remain

### D1a enforcement

- **MemoryCommand::Sync** → corpus — dispatch stays in residual match (NOT
  memory.rs). `corpus` already imports `memory`; routing Sync to `memory.rs`
  would close `corpus↔memory` 2-cycle → tangle 120→≥121. The residual arm
  calls `corpus::run_sync` directly in main.rs; destined for `commands/cli.rs`
  in PHASE-04.
- **SpecReqCommand** → spec.rs (own-module, NOT requirement.rs).
  `requirement.rs` has no CLI and doesn't import `spec`; `spec` already imports
  `requirement`. Routing to `requirement` would mint `requirement→spec` cycle.
  Folded into `spec.rs` — zero new edge.

### Batch ladder

```
A (7): Boot, Skills, Policy, Standard, Rfc, Rec, Knowledge
B (5): Revision+Change, Adr, Backlog, Slice, Worktree
C (4): Spec, SpecReq→spec (D1a), Coverage→commands/coverage, Map→commands/map
D (4): Dispatch+Candidate, Catalog+orphans, ConceptMap, Review
E (1): Memory+Sync (D1a carve-out — Sync stays residual)
```

### Verification

- 2142 tests pass, `cargo clippy --bin doctrine` zero-warn
- `tests::architecture_layering_gate`: **command = 120 unchanged**
- `tests/e2e_*` goldens untouched
- Grep: only `Command` + `ExportCommand` enums remain in main.rs

### Notes

- `CommonListArgs` stays at crate root (main.rs) — inert to the gate (design §4 F-B).
  The pre-filter handles its edge visibility from relocated kind enums.
- `ExportCommand` is a top-level verb, not nested; stays residual for PHASE-04
  (`commands/cli.rs`).
- Nested enums (`CandidateCommand`⊂Dispatch, `SyncCommand`⊂Memory,
  `RevisionChangeCommand`⊂Revision) rode their parents correctly.
- `FindRetrieveArgs` struct moved to `memory.rs` with `MemoryCommand`.
- No `use crate::commands::*` glob in `main.rs` — all qualified paths.
- `run_catalog_scan`/`run_catalog_graph` orphans relocated into `catalog.rs` with
  the Catalog fold (own-module, zero new edge).

### Durable items

- **Gate pre-filter pattern:** crate-root type references (`crate::TypeName`) cross
  the module boundary when enums relocate from main.rs. The edge extractor can't
  distinguish module names from type names. The source-file existence filter is
  the minimal correct fix — recorded as memory `mem_019ee7d35d0d`.
