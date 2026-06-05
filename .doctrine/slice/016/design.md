# Design SL-016: Break slice↔state cycle: extract plan types

## 1. Design Problem

`src/` carries one import cycle: `slice ↔ state`. It is the only cycle in an
otherwise-acyclic module graph (2026-06-05 coupling assessment, recorded in
`slice-016.md` §Context). It also violates **ADR-001** (module layering:
dependencies point downward, no cycles).

Lift the authored phase-plan model (`Plan`, `PlanPhase`, `Plan::parse`) out of
`slice.rs` into a neutral `plan` module so the graph becomes a clean DAG. Pure
structural move — no behaviour change.

## 2. Current State

`slice.rs` defines the plan model and consumes `state`; `state.rs` reaches back
*up* into `slice` for the model's types:

- `slice → state` — calls `init_phases`, `set_phase_status`, `phase_rollup`;
  uses types `PhaseStatus`, `PhaseRollup`. (Command → engine — fine per ADR-001.)
- `state → slice` — **one line**, `state.rs:28` `use crate::slice::{Plan, PlanPhase}`.
  Types only. This single edge closes the cycle.

The original author flagged this as pre-acknowledged debt (`state.rs:22-27`):

> v1 debt (audit [watch]): the runtime layer reaches *up* into the slice-CLI
> module for its input model … lift `Plan` to a neutral home … if a second
> consumer of either appears.

`state` is now (effectively) that second consumer — it is the one consumer
besides the authoring shell, and the upward edge is what this slice discharges.

`Plan` today bundles two responsibilities in `slice.rs`:
- **pure** — `struct Plan { phases }`, `struct PlanPhase { id, name, objective }`,
  `Plan::parse` (private) + a `Raw` serde shim; rejects duplicate phase ids.
- **impure** — `read_plan(slice_root, id) -> Result<Plan>` reads
  `slice/NNN/plan.toml` off disk and calls `parse`.

## 3. Forces & Constraints

- **ADR-001** — `plan` must land in a tier the graph can depend on acyclically;
  impurity must not sink into the engine core.
- **Pure/imperative split** (`doc/slices-spec.md` §Architecture) — disk IO stays
  in the shell; the pure parser is an input to it.
- **Behaviour-preservation gate** — shared-machinery change; existing suites are
  the proof and must stay green with assertions unchanged.
- **Storage rule / immutability** — N/A (no authored-id or schema change).

## 4. Guiding Principles

Smallest move that makes the graph acyclic. Separate the pure parser (shared,
moves) from the disk read (shell-bound, stays). Leave every unrelated coupling
untouched.

## 5. Proposed Design

### 5.1 System Model

New module `src/plan.rs`, declared `mod plan;` in `main.rs`. It owns the pure
authored-plan model and parser. Zero `crate::` dependencies (only `serde`,
`toml`, `anyhow`) — a **leaf** in the engine tier (ADR-001).

Resulting edges:

```
slice → plan        (Plan/PlanPhase types, Plan::parse via read_plan)
slice → state       (unchanged: init_phases, set_phase_status, phase_rollup, …)
state → plan        (Plan/PlanPhase types — was: state → slice)
plan  → (nothing in-crate)
```

`state → slice` is removed. Graph is acyclic.

### 5.2 Interfaces & Contracts

`plan` module public surface (crate-visible):

```rust
pub(crate) struct Plan { pub phases: Vec<PlanPhase> }
pub(crate) struct PlanPhase { pub id: String, pub name: String, pub objective: String }
impl Plan {
    /// Parse + validate a `plan.toml` body. Rejects duplicate phase ids.
    pub(crate) fn parse(text: &str) -> anyhow::Result<Plan>;
}
```

Visibility change: `parse` was a private `fn`; it becomes `pub(crate)` so
`slice::read_plan` can call it across the module boundary. No other signature
changes. `read_plan` keeps its shape — `fn read_plan(slice_root: &Path, id: u32)
-> anyhow::Result<Plan>` — now returning `plan::Plan`.

`state` consumers are by-reference and unchanged: `init_phases(…, plan: &Plan, …)`
(`state.rs:172`), `render_phase_sheet(phase: &PlanPhase)` (`state.rs:333`).

### 5.3 Data, State & Ownership

No data-shape change. `plan` owns the type definitions; `slice` owns the disk
read (`read_plan`) and remains the only producer of a `Plan` (from `plan.toml`);
`state` is a pure consumer of `&Plan`. The `read`/`parse` split puts the one
impure step (disk) in the shell and the pure step (parse) in the engine leaf.

### 5.4 Lifecycle, Operations & Dynamics

Runtime flow unchanged: `slice::run_plan`/phase-materialisation → `read_plan`
(disk) → `plan::Plan::parse` (pure) → `state::init_phases(&plan)` → phase sheets.
Only the module boundary that `parse` sits behind moves.

### 5.5 Invariants, Assumptions & Edge Cases

- **Invariant preserved**: duplicate `PHASE-NN` ids rejected at parse
  (`Plan::parse`), per-id well-formedness still enforced at the FS boundary by
  `state::phase_stem`. Neither relocates.
- **Assumption**: nothing imports `slice`'s `Plan` by glob or re-export. Verified
  — only `state.rs:28` imports it; `main.rs` dispatches via `run_plan` and never
  names the type.
- **Edge**: `install::Plan` and `skills::Plan` are unrelated local types at
  different paths — no collision with `crate::plan::Plan`.

## 6. Open Questions & Unknowns

None open. The read/parse placement (the only decision) is resolved in §7 D1.

## 7. Decisions, Rationale & Alternatives

**D1 — parse moves, read stays.** `Plan::parse` (pure) → `plan`; `read_plan`
(disk IO) stays in `slice`. Rationale: ADR-001 keeps impurity out of the engine;
`state` needs only the types and `&Plan` (never reads), so the read has no reason
to move. *Alternative rejected*: move `read_plan` too — would put disk IO in the
engine leaf, violating the pure/imperative split for no benefit.

**D2 — module name `plan`, engine-tier leaf.** Neutral, matches the artifact it
models (`plan.toml`). Zero in-crate deps keeps it a leaf — lowest possible
coupling. *Alternative rejected*: fold types into `meta` — `meta` is
entity-metadata, a different concern; would conflate.

**D3 — `PhaseStatus` clap leak left in place.** The same debt comment names
`PhaseStatus: clap::ValueEnum` (arg-parser in the engine). It is **orthogonal** —
`state → slice` is solely the `Plan` import, so the clap leak forms no part of
the cycle. Out of scope (user decision, 2026-06-05); noted follow-up.

## 8. Risks & Mitigations

- **R1 — behaviour drift during the move.** *Mitigation*: pure relocation;
  existing suites stay green with assertions unchanged; `cargo build` proves the
  graph compiles acyclically.
- **R2 — missed consumer / hidden re-export.** *Mitigation*: verified the only
  non-`slice` consumer is `state.rs:28`; post-move `grep` gate asserts no module
  imports `slice` for plan types.
- **R3 — visibility widening.** `parse` private → `pub(crate)`. Bounded to the
  crate; acceptable, and required for the shell/engine split.

## 9. Quality Engineering & Validation

- **Behaviour-preservation**: full suite green, assertions unchanged. Test
  relocation is **selective** (adversarial finding A3): the two pure-parser tests
  — `slice.rs:711` (multi-phase parse) and `:727` (duplicate-id reject), both
  inline-TOML → `Plan::parse` — move to `plan.rs`. The scaffold-acceptance test
  `:734` calls slice-private `render_plan_toml`, so it is a slice-side *contract*
  test (renderer × parser) and **stays in `slice.rs`**, now calling
  `plan::Plan::parse` cross-module. `slice.rs:657`
  (`render_plan_toml_substitutes_ref_and_parses`) likewise stays. `state.rs` test
  helper `fn plan()` (`state.rs:420`) keeps working via the new import path.
- **Structural proof** (new, cheap): `cargo build` (acyclic compile) + a grep
  confirming `state.rs` no longer imports `crate::slice` and no module imports
  `slice`'s plan types. This is the closure evidence for breaking the cycle.
- **Gate**: `cargo clippy` zero warnings; `just check` green.
- No new behavioural tests — a pure move adds no behaviour to cover.

## 10. Review Notes

Internal adversarial pass (2026-06-05), claims verified against source:

- **A1 — `main.rs` blast radius (checked, clean).** `main.rs:432`/`:565` `Plan`
  are the clap `SliceCommand::Plan` *subcommand variant*, not the data type.
  main.rs never names the `Plan` type. Confirms §5.5 assumption.
- **A2 — `state.rs` residual coupling (checked, clean).** state.rs has many
  `slice` tokens, but all are doc comments, path string consts (`SLICE_DIR`,
  `STATE_SLICE_DIR`), `slice_id` params, or the `make_slice_dir` test helper —
  none are module deps. The only `crate::slice` edge is line 28; removing it
  fully severs the prod cycle.
- **A3 — test relocation was overstated (REAL; integrated into §9).** The
  scaffold-acceptance test (`slice.rs:734`) calls slice-private
  `render_plan_toml`, so it is not a pure-parser test. Corrected: only the two
  self-contained tests (`:711`, `:727`) move to `plan.rs`; the contract tests
  (`:734`, `:657`) stay in `slice.rs`.

No governance conflict surfaced; ADR-001 + pure/imperative split fully satisfied.
