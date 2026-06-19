# Design SL-112: Machine-check ADR-001 layering via a `syn` dependency-fitness test

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-111, ADR-001, ADR-013); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8), VT-1 (§9). -->

## 1. Design Problem

ADR-001 (`leaf ← engine ← command, no cycles`) is **review-only**. It has no
automated gate, and the 2026-06-19 architecture audit confirmed the drift the ADR
itself predicted: cycles between the relation engine and the command tier (broken
by SL-111). ADR-001 named two escalations once cycles recur — promoting the engine
to its own crate, and a fitness function. We are past the trigger.

The leverage is **durability**: once SL-111 makes the production graph acyclic, a
gate stops the drift re-growing the moment a future change reintroduces an upward
edge. Without it every structural fix decays back under review-only pressure.

This slice lands the **fitness test now**; the engine **crate split is deferred**
to a follow-on slice (User, 2026-06-19) — the layer map this slice records is the
de-risking artefact the later cut consumes (§8 R1).

## 2. Current State

- **Production graph is acyclic** post-SL-111. Verified: the remaining
  `relation → slice/adr/spec/…` upward edges are **all `#[cfg(test)]`-only**
  (`relation.rs:985+`, inside `mod tests` at `:967`) — out of the bins/lib graph
  ADR-001 governs. `just gate` runs `cargo clippy` bins/lib-only (no
  `--all-targets`), so the gate's altitude already excludes test code.
- **No automated enforcement.** ADR-001 § Verification records the rule as
  review-enforced *now*, the crate boundary *later*, and **explicitly rejects** a
  "homegrown module-graph unit test" as "brittle toil." This slice reopens that
  rejection (§7 D5).
- **The ADR's per-module tier table is stale** — it lists ~16 modules; `src/` now
  has ~63 top-level modules. The table is queried/derived data living in prose,
  which the storage rule forbids; it drifted exactly as that rule predicts.
- **No dependency tooling in the jail** (`cargo-deny`/`cargo-modules` absent);
  adding one is a flake change requiring User sign-off — out of scope.

## 3. Forces & Constraints

- **ADR-001** — the rule under enforcement; dependencies point downward only, no
  cycles. The gate is its first machine check.
- **Storage rule** (AGENTS.md) — queried/derived data is structured, never prose.
  The per-module tier assignment must be structured data, not an ADR prose table.
- **ADR-001's own brittleness objection** — a source-parsing test was rejected as
  brittle. A credible gate must *answer* that: `syn` (the canonical Rust AST
  parser) replaces fragile regex/brace-tracking, and the verdict logic is pure +
  unit-tested.
- **`just gate` integration** — `gate` runs `test-all` (`cargo test --workspace`).
  A `cargo test` rides that for free; no new gate wiring.
- **Pure/imperative split** (CLAUDE.md, slices-spec § Architecture) — the impure
  fs+`syn` walk is a thin shell; the decision logic is a pure function.
- **No parallel implementation / write less code** (CLAUDE.md) — one gate, one
  authoritative layer map, no second classifier.
- **ADR-013** — governance dependency routes through a Revision; the ADR-001
  amendment is written via a REV at reconcile, not hand-edited as the design step.
- **CHR-014** — source-reading tests that bake `env!("CARGO_MANIFEST_DIR")` go
  stale under a shared `CARGO_TARGET_DIR` across dispatch worktrees (§7 D4).

## 4. Guiding Principles

- **Gate the rule that is actually drifting.** Enforce ADR-001 rules 1 (downward)
  and 2 (acyclic); leave rule 3 (engine purity) to the convention the audit found
  well-honored (§7 D3).
- **Robust, not brittle.** Parse with `syn`, not regex — the gate's credibility is
  the whole point, and ADR-001 rejected the brittle version for cause.
- **Govern the production graph only.** Same altitude as `clippy`/`just gate`; the
  `#[cfg(test)]` upward edges are out of contract, not violations.
- **The map is canon, in one place, self-enforcing.** A single reviewed `const`
  table; a module absent from it (or absent from `src/`) fails the gate.
- **Pure core, thin shell.** All verdict logic in a pure fn, unit-tested with
  synthetic inputs — including a negative self-test so the gate cannot pass
  vacuously.

## 5. Proposed Design

### 5.1 System Model

A new integration test `tests/architecture_layering.rs` is the gate. It runs under
`cargo test` (hence `just gate` via `test-all`). It has two halves:

```
  src/**/*.rs ──syn parse, skip cfg(test)──▶ (modules, edges)   [impure shell]
                                                   │
                                                   ▼
        const LAYER_MAP ──▶ check_layers(modules, edges, map) ──▶ Vec<Violation>   [pure]
                                                   │
                              empty ⇒ pass    │    non-empty ⇒ panic! naming each edge/module
```

The shell reads source relative to **runtime CWD** (cargo sets the test process's
working directory to the package root per invocation — a runtime property), *not*
the compile-time `env!("CARGO_MANIFEST_DIR")`. A reused binary under a shared
target dir therefore still reads the worktree it was *run* in (§7 D4, sidesteps
CHR-014). A cheap pre-flight asserts `./src` and `./Cargo.toml` exist with a clear
"run from package root" message.

### 5.2 Interfaces & Contracts

```rust
// pure core — all logic here, no IO
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier { Leaf = 0, Engine = 1, Command = 2 }   // ordinal = altitude

const LAYER_MAP: &[(&str, Tier)] = &[
    // leaf
    ("clock", Tier::Leaf), ("root", Tier::Leaf), ("fsutil", Tier::Leaf),
    ("input", Tier::Leaf), ("git", Tier::Leaf), ("kinds", Tier::Leaf), /* … */
    // engine
    ("entity", Tier::Engine), ("registry", Tier::Engine), ("relation", Tier::Engine),
    ("meta", Tier::Engine), ("state", Tier::Engine), ("plan", Tier::Engine), /* … */
    // command
    ("slice", Tier::Command), ("adr", Tier::Command), ("memory", Tier::Command),
    ("boot", Tier::Command), ("mcp_server", Tier::Command), ("main", Tier::Command), /* … */
];

#[derive(Debug, PartialEq, Eq)]
enum Violation {
    Unclassified(String),                 // module in src/ but not in LAYER_MAP
    StaleEntry(String),                   // LAYER_MAP key with no src/ module
    UpwardEdge { from: String, to: String, from_tier: Tier, to_tier: Tier },
    Cycle(Vec<String>),                   // SCC with size > 1
}

/// Pure. No IO. The entire verdict.
fn check_layers(
    modules: &BTreeSet<String>,
    edges: &BTreeSet<(String, String)>,
    map: &[(&str, Tier)],
) -> Vec<Violation>;
```

Shell (impure, thin):

```rust
fn discover_modules(src: &Path) -> BTreeSet<String>;      // top-level .rs + dirs under src/
fn extract_edges(src: &Path) -> BTreeSet<(String,String)>; // syn walk, cfg(test)-skipped
```

### 5.3 Data, State & Ownership

- **`LAYER_MAP` owns the tier assignment.** One authority; ADR-001 carries the tier
  *definitions* + rule and points here (§7 D2). No prose per-module table survives.
- **Module = first path component under `src/`.** `relation.rs` → `relation`;
  `catalog/scan.rs` → `catalog`; `mcp_server/foo.rs` → `mcp_server`. Subdir files
  inherit the dir's tier; **intra-module edges are ignored** (matches ADR-001:
  tier is a property of a module, module-level).
- **Edge = (owning-module, target-module)** where target = first segment after
  `crate::`. Deduped; self-edges dropped.
- **No runtime state.** `LAYER_MAP` is `const`; the rest is per-run local.

### 5.4 Lifecycle, Operations & Dynamics

`check_layers`:

1. `modules` from `discover_modules`. Assert `LAYER_MAP` keys == `modules`
   (`Unclassified` for src-not-in-map, `StaleEntry` for map-not-in-src) — the
   forcing function in both directions.
2. For each `(s, t)` in `edges` with both classified: if `tier(t) > tier(s)` →
   `UpwardEdge`.
3. Tarjan SCC over `(modules, edges)`; every SCC of size > 1 → `Cycle` (catches
   same-tier cycles rules 1+2 would miss).
4. Return all violations. The `#[test]` panics with a formatted report listing each
   offending edge/module if non-empty.

`extract_edges`: per `src/**/*.rs`, `syn::parse_file`; walk items with a `Visit`
impl that **does not recurse into `#[cfg(test)]`-attributed items/mods**; collect
every `Path` whose leading segment is `crate` (covers `use` trees and inline path
exprs); map to `(owning_module, first_seg_after_crate)`.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — `LAYER_MAP` keys are exactly the discovered top-level modules. Drift
  either way is a violation, so the map cannot silently rot as the ADR table did.
- **INV-2** — the production graph stays a DAG with downward-only cross-tier edges.
  This is the property the gate exists to pin; green today (SL-111), red on
  regression.
- **ASM-1** — cargo sets the test process CWD to the package root. **Load-bearing**
  (§7 D4); the pre-flight probe converts a violated assumption into a clear failure
  rather than a mis-read tree. Confirmed empirically at execution (OQ-2 → resolved
  by probe).
- **ASM-2** — `use crate::X` / `crate::X::…` paths capture the cross-module edges
  that matter; a re-export laundering an edge through a third module is possible but
  not present and not worth modelling now (§8 R2).
- **Edge — `#[cfg(test)]` upward edges are not violations.** The `Visit` skip keeps
  `relation`'s test imports of `slice::SLICE_KIND` out of scope by construction.
- **Edge — `mod foo { … }` inline vs `mod foo;` file.** Both resolve to the same
  top-level module name; the walk keys on the *file's* top-level path component, so
  inline submodules inherit correctly.
- **Edge — paths like `crate::catalog::scan::f`** count as an edge to `catalog`
  (first segment only); deeper segments are intra-module and ignored.

## 6. Open Questions & Unknowns

- **OQ-1 — exact tier for the ambiguous ~20 modules** (`install`-as-utility wart
  ADR-001 flagged, plus `integrity`, `governance`, `value`, `lifecycle`, `conduct`,
  `links`, `dep_seq`, `projection`, `coverage*`, …). **Non-blocking**: resolved at
  execution by the gate itself — it will not go green until `LAYER_MAP` is
  self-consistent with the real edges, so authoring the map *is* the classification
  work, and a misclassification surfaces as a concrete `UpwardEdge`, not a judgement
  call. `install` is classified where its edges land; the wart is *recorded*, not
  *resolved* (slice Non-Goals).
- **OQ-2 — CHR-014 footgun.** *Resolved* (§5.1/D4): CWD-relative read + probe;
  `env!` dropped.
- **OQ-3 — rule 3 (engine purity).** *Deferred* (Q4-A, §7 D3); the impure-leaf
  refinement (tag `git`/`clock` impure, forbid engine→impure-leaf) is the named
  follow-up (§8 R3).

## 7. Decisions, Rationale & Alternatives

- **D1 — Fitness test now; engine crate split deferred to a follow-on slice.**
  *Alt A:* crate split now (compiler-enforced, strongest). Rejected for *this*
  slice — heaviest path: severing every engine→command edge incl. `relation`'s test
  constants, large mechanical churn, Med risk, and it needs the authoritative layer
  map anyway. *Alt B:* both at once. Rejected — couples a durable cheap win to a
  large risky move. **Chosen:** land the durable gate cheaply; the `LAYER_MAP` this
  slice authors is the crate split's de-risking input (§8 R1). Matches the slice's
  own follow-up note and ADR-001's "gate now, crate boundary when it settles."
- **D2 — Tier map as a central Rust `const LAYER_MAP`, not prose, per-module
  marker, or TOML.** *Alt — per-module `//! Tier:` marker:* best cohesion but no
  single survey point and 63 annotations; *Alt — TOML data file:* language-neutral
  but adds a file + parse + IO for no gain (readers here read Rust). *Alt — ADR
  prose table:* the status quo that drifted; forbidden by the storage rule.
  **Chosen:** one greppable, compile-checked, reviewed table the gate reads with no
  IO; ADR-001 holds the rule + tier definitions and points at it as the
  authoritative assignment. Surveyable *and* self-enforcing (INV-1).
- **D3 — Enforce ADR-001 rules 1 (downward) + 2 (acyclic) only; rule 3 (engine
  purity) deferred.** *Alt:* also gate purity (flag engine use of impure seams +
  `std::fs`/`process`/`SystemTime`). Rejected for scope — the audit found rule 3
  *well-honored* ("clock seam… date/uid pattern is real"), full purity detection is
  fuzzy with false-positive risk, and the directional/cyclic rule is the one that
  actually drifted. The cheap **impure-leaf refinement** (tag `git`/`clock` impure,
  forbid `engine→impure-leaf` in the same edge framework) is recorded as the follow-up
  (§8 R3), to be picked up "maybe later" (User, 2026-06-19).
- **D4 — Read `src/` relative to runtime CWD, not `env!("CARGO_MANIFEST_DIR")`.**
  *Alt:* `env!` (idiomatic). Rejected — `env!` bakes the path at compile time, so a
  reused test binary under a shared `CARGO_TARGET_DIR` reads a stale worktree
  (CHR-014). Cargo sets the test CWD to the package root per invocation (runtime),
  so a CWD-relative read tracks the worktree actually run. A pre-flight probe
  (ASM-1) makes the cargo guarantee a loud failure if ever violated. Sidesteps
  CHR-014 for this test without solving it globally.
- **D5 — Amend ADR-001 in place (reverse its "rejected" stance); the write routes
  through a REV at reconcile.** *Alt — follow-on ADR:* rejected — fragments the
  layering canon; this *is* ADR-001's own "Later — crate graph" plan maturing plus
  a reversal of its homegrown-test rejection (justified: the cycles arrived, so the
  cost/benefit flipped, and `syn` removes the brittleness that grounded the
  rejection). ADR-001 § Verification gains the fitness test as *now*-enforcement,
  its stale per-module prose table is replaced by tier definitions + a `LAYER_MAP`
  pointer, and the crate split stays the named future escalation. Per ADR-013 the
  governance write is a REV authored at `/reconcile`; this design records intent.
- **D6 — `syn` as a test-only dev-dependency.** *Alt:* regex line-scan, no dep.
  Rejected — regex cannot cleanly skip `#[cfg(test)]` scope (brace-tracking is the
  brittleness ADR-001 named) and catches `crate::` in strings/comments. `syn` is
  centralized in `[workspace.dependencies]` with a reason comment and added to this
  crate's `[dev-dependencies]` only — **zero shipped-binary weight**.

## 8. Risks & Mitigations

- **R1 (deferred crate split orphaned)** — landing only the test could leave the
  crate boundary indefinitely unmet. *Mit:* the slice's Follow-Ups name the crate
  split as a successor slice seeded by `LAYER_MAP`; ADR-001 keeps the crate boundary
  as the named escalation. The test is not a *replacement* for the boundary, it is
  the interim gate + the map that de-risks it.
- **R2 (edge laundering / dynamic reach)** — a re-export or macro could route an
  upward edge the `use crate::` walk misses (ASM-2). *Mit:* not present today;
  `syn` sees the literal `crate::` path in `use` *and* expression position, so only
  re-export laundering evades it, which the human/`/inquisition` layer still
  catches. Modelling it now is speculative scope.
- **R3 (rule 3 gap)** — engine purity stays review-only. *Mit:* audit found it
  well-honored; the impure-leaf refinement is a recorded cheap follow-up (§7 D3),
  not a silent omission.
- **R4 (false green from a parser miss)** — a bug in `extract_edges` could under-
  report edges and pass vacuously. *Mit:* the pure `check_layers` is unit-tested
  with synthetic inputs incl. a known upward edge and a 2-cycle (VT-2); and an EX
  check hand-introduces a real upward edge and asserts `just gate` fails naming it
  (§9). The gate is proven to *bite*, not merely to be green.
- **R5 (`LAYER_MAP` authoring churn)** — classifying ~63 modules is the bulk of the
  work and a classification can be wrong. *Mit:* INV-1 + the gate make every error a
  concrete failure (an `UpwardEdge` or an `Unclassified`), not a silent mistake; the
  map converges against the real graph rather than against judgement.

## 9. Quality Engineering & Validation

- **VT-1 (real-graph gate, primary).** `tests/architecture_layering.rs` builds the
  production graph and asserts zero violations. Green now (SL-111 left it acyclic);
  red on any future upward/cyclic edge or unclassified module. Runs under
  `just gate` (`test-all`).
- **VT-2 (bite-proof, pure unit tests).** `check_layers` over synthetic inputs:
  legal set → `[]`; injected upward edge → one `UpwardEdge`; injected 2-cycle →
  `Cycle`; src-only module → `Unclassified`; map-only key → `StaleEntry`. Guards
  R4.
- **VT-3 (cfg(test) exclusion).** A fixture asserting a `#[cfg(test)]`-scoped
  `crate::<command>` path produces **no** edge — pins the production-only contract
  (ASM, the `relation` test edges).
- **EX-1 (evidence the gate fails closed).** Hand-introduce a real upward edge
  (engine module `use`s a command module in non-test code) → `just gate` fails
  naming the edge; revert. Recorded, not committed.
- **EX-2** — `LAYER_MAP` keys == discovered modules (drop/add a module → red).
- **EX-3** — ADR-001 updated: fitness test recorded as now-enforcement, stale prose
  tier table replaced by definitions + `LAYER_MAP` pointer (written via REV at
  reconcile, D5).
- **EN-1** — SL-111 done (production graph acyclic). **EN-2** — `syn` dev-dep
  landed, `just gate` green.
- **`just gate`** green (clippy zero-warning, fmt, `test-all`).

## 10. Review Notes

### Internal adversarial pass (pending)

To run after this draft locks (design § Adversarial review): attack the
production-only scope (does skipping `#[cfg(test)]` hide a *real* upward edge that
only `cfg(test)` exercises but ships?), the CWD assumption (ASM-1 — confirm cargo's
guarantee empirically, not by docs), the `syn` path-collection completeness
(macro-generated paths, `crate::` in `macro_rules!` bodies), and the
`LAYER_MAP`-as-canon claim against the storage rule (is a Rust `const` "structured"
enough, or does canon want the map in `.doctrine`?).

### Doctrinal alignment

ADR-001 — the gate *implements* it; rules 1+2 enforced, rule 3 deferred with a
recorded follow-up (no silent narrowing). ADR-013 — the ADR-001 amendment routes
through a REV at reconcile (D5), not a hand-edit. Storage rule — the tier
assignment moves from prose (forbidden) to a reviewed `const` (D2). Pure/imperative
split honored (§5.1). No governance conflict surfaced; no `/consult` required.
