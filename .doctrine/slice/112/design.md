# Design SL-112: Machine-check ADR-001 layering via a `syn` dependency-fitness gate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-111, ADR-001, ADR-013, CHR-015); doc-local refs bare — OQ-1 (§6),
     D1 (§7), R1 (§8), VT-1/F-1 (§9/§10). -->

## 1. Design Problem

ADR-001 (`leaf ← engine ← command, no cycles`) is **review-only**. The 2026-06-19
architecture audit confirmed the drift the ADR predicted: engine→command cycles
(broken by SL-111). ADR-001 named two escalations — an engine crate, and a fitness
function. We are past the trigger.

The leverage is **durability**: a gate stops the boundary re-eroding the moment a
future change reintroduces an upward edge. This slice lands the **fitness gate
now**; the engine **crate split is deferred** to a follow-on slice (User,
2026-06-19) — the layer map this slice authors is the de-risking artefact the
later cut consumes (§8 R1).

The design pass surfaced that the real graph is more tangled than SL-111's closure
implied (§2 / §10). That reshaped the gate from "assert the graph is clean" into
**"freeze the current boundary and ratchet it"** — a hard directional gate plus a
numeric tangle ratchet where a hard gate is not yet achievable.

## 2. Current State (measured, design probe 2026-06-19)

Empirical probe of the production `crate::` graph (regex preview; the authoritative
`syn` graph is PHASE-01). Key facts — several correct earlier assumptions:

- **The whole graph is NOT acyclic.** SL-111 made the **engine tier** acyclic and
  broke the engine→command *upward* cycles; it never made the whole graph acyclic.
  An earlier draft of this design wrongly claimed it did.
- **Engine core is essentially a clean DAG.** Among
  `entity/registry/relation/meta/state/plan`: only `meta→entity`, `state→plan`,
  `status→meta` — all downward, **zero back-edges**. The one real core cycle is
  **`conduct ↔ dtoml`** → captured as **CHR-015** (out of scope here; the tangle
  ratchet pins it from growing, §5).
- **Command tier is heavily cyclic** — a large intra-tier SCC (`adr↔governance`,
  `backlog↔integrity`, `plan`/`slice` neighbourhood, …) plus many 2-cycles. Normal
  for a CLI verb layer; **cohesion per verb is fine** (audit). This is *intra-tier*
  — not a layering-direction violation, and out of scope to *resolve* (Non-Goals).
- **Leaf tier is clean** — `{clock, root, fsutil, kinds, git}` import only leaves.
- **ADR-001's tier table is wrong, not merely stale.** It lists `input` as
  leaf/seam, but `input` imports `entity`+`meta` (engine) → `input` is **engine**
  (F-6). The prose table also predates ~47 modules.
- **`relation_graph` + `integrity` reach up into command modules in production**
  (`relation_graph` → `governance/adr/policy/standard/backlog/spec` inline calls;
  `integrity` → 11 command shells via the `*_KIND` view). Neither is imported by
  any engine-core module, so both classify cleanly as **command-tier**, making
  those reaches *intra-command* (legal), not upward (F-1, F-3).
- **`state → install`** is a genuine engine→command upward edge — ADR-001's own
  documented wart ("install … doubling as a utility provider"); Non-Goals say
  *classify, don't resolve* (F-2).
- **Doc-comment intra-doc links are a false-edge source** — `git → retrieve` was a
  rustdoc `///` link, not a code edge (F-5). The `syn` AST walk dodges these (doc
  comments are string attributes, not path nodes); a regex gate would not.
- **No dependency tooling in the jail** (`cargo-deny`/`cargo-modules`); adding one
  is a flake change needing User sign-off — out of scope.

## 3. Forces & Constraints

- **ADR-001** — the rule under enforcement. Rule 1 (downward only) is achievable
  and hard-gated; rule 2 (no cycles) is *comprehensively violated intra-command*
  today and unfixable in scope → enforced as a **non-increasing ratchet**, not
  abandoned (§7 D3). Rule 3 (engine purity) deferred (§7 D3b).
- **Storage rule** (AGENTS.md) — queried/derived data is structured, never prose.
  The per-module tier assignment must be a structured artefact, not an ADR table.
- **ADR-001's own brittleness objection** — a source-parsing test was rejected as
  "brittle toil." The gate must *answer* that: `syn` (canonical AST) over
  regex/brace-tracking, verdict logic pure + unit-tested (§7 D6).
- **`just gate` integration** — `gate` runs `test-all` (`cargo test --workspace`).
  A `cargo test` rides it for free; no new wiring.
- **Pure/imperative split** (CLAUDE.md) — impure fs+`syn` walk is a thin shell; the
  verdict is a pure function.
- **No parallel implementation / write less code** — one gate, one authoritative
  map + baselines; no second classifier.
- **ADR-013** — governance dependency routes through a Revision; the ADR-001
  amendment is authored via a REV at reconcile, not hand-edited as the design step.
- **CHR-014** — source-reading tests that bake `env!("CARGO_MANIFEST_DIR")` go
  stale under a shared `CARGO_TARGET_DIR` across worktrees (§7 D4).
- **Gate value is unproven until classification** — the worth of the gate is the
  count of genuine upward edges it will catch in future; unknown until the map
  exists, and classification is judgement-laden + gameable (§4, §7 D7, §8 R3).

## 4. Guiding Principles

- **Classify-first, then decide.** PHASE-01 produces the real `LAYER_MAP` + counts
  the upward baseline + measures per-tier tangle, *before* committing to build the
  gate. Small upward baseline + meaningful engine core → proceed; mostly-baseline →
  the gate is a fig leaf and we reconsider (§7 D7). The biggest risk becomes a
  cheap early checkpoint, not a bet.
- **Classify by what a module *knows*, not by what makes the gate green.** Each
  ambiguous module gets a one-line tier rationale authored *before* seeing green. A
  module that is "command" only because that legalises an edge is a smell (§8 R3).
- **Hard-gate what you can; ratchet what you cannot.** Cross-tier direction is
  hard-gated. Intra-tier cycles (unfixable in scope) are pinned by a numeric
  monotonic-down ratchet — frozen, never allowed to grow.
- **Govern the production graph only.** Same altitude as `clippy`/`just gate`;
  `#[cfg(test)]` edges and doc-comment links are out of contract by construction.
- **The map is canon, in one place, self-enforcing.** One reviewed `const` table; a
  module absent from it (or absent from `src/`) fails the gate.
- **Pure core, thin shell, prove it bites.** All verdict logic pure + unit-tested,
  including negative self-tests so the gate cannot pass vacuously.

## 5. Proposed Design

### 5.1 System Model

`tests/architecture_layering.rs` is the gate, run under `cargo test` (hence
`just gate` via `test-all`). Impure shell → pure verdict:

```
  src/**/*.rs ──syn parse; skip cfg(test); path-nodes only──▶ (modules, edges)   [shell]
                                                                   │
   LAYER_MAP, ACCEPTED_VIOLATIONS, TANGLE_BASELINE                 ▼
        └────────────────────────────────▶ check(...) ──▶ Vec<Violation>   [pure]
                                                  │
                       empty ⇒ pass    │    non-empty ⇒ panic! naming each item
```

Three assertions:
1. **Completeness (forcing)** — every discovered top-level module is in `LAYER_MAP`,
   and vice-versa.
2. **Cross-tier direction (hard)** — every edge `s→t` with `tier(t) > tier(s)` is a
   violation, *unless* listed in `ACCEPTED_VIOLATIONS`. The accepted set is frozen
   and may not grow.
3. **Per-tier tangle (numeric ratchet)** — for each tier, `tangle(tier) ≤
   TANGLE_BASELINE[tier]`, where `tangle = Σ(SCC_size − 1)` over non-trivial
   intra-tier SCCs (orientation-free; `0` ⇔ acyclic). Monotonic-down only.

The shell reads `src/` relative to **runtime CWD** (cargo sets the test process CWD
to the package root per invocation), *not* compile-time `env!` (§7 D4, sidesteps
CHR-014). A pre-flight asserts `./src` + `./Cargo.toml` exist with a clear
"run from package root" message.

### 5.2 Interfaces & Contracts

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier { Leaf = 0, Engine = 1, Command = 2 }   // ordinal = altitude

const LAYER_MAP: &[(&str, Tier)] = &[ /* authored PHASE-01; ~68 modules */
    ("clock", Tier::Leaf), ("root", Tier::Leaf), ("fsutil", Tier::Leaf),
    ("kinds", Tier::Leaf), ("git", Tier::Leaf),
    ("entity", Tier::Engine), ("registry", Tier::Engine), ("relation", Tier::Engine),
    ("meta", Tier::Engine), ("state", Tier::Engine), ("plan", Tier::Engine),
    ("input", Tier::Engine), /* F-6: was mis-listed leaf */ /* … */
    ("slice", Tier::Command), ("integrity", Tier::Command), /* F-3 */
    ("relation_graph", Tier::Command), /* F-1 */ ("install", Tier::Command), ("main", Tier::Command), /* … */
];

// Frozen baseline of pre-existing genuine upward edges. Small, enumerated, each
// annotated with a follow-up. MAY shrink, MUST NOT grow.
const ACCEPTED_VIOLATIONS: &[(&str, &str, &str)] = &[
    ("state", "install", "ADR-001 install-as-utility wart; resolve via helper extraction (follow-up)"),
    // … any others PHASE-01 surfaces, each with a reason + follow-up id
];

// Per-tier tangle ceiling, frozen at PHASE-01 measurement. MAY shrink, MUST NOT grow.
const TANGLE_BASELINE: &[(Tier, u32)] = &[
    (Tier::Leaf, 0), (Tier::Engine, /* conduct↔dtoml ⇒ 1, CHR-015 */ 1), (Tier::Command, /* measured */ 0),
];

#[derive(Debug, PartialEq, Eq)]
enum Violation {
    Unclassified(String),                 // src module not in LAYER_MAP
    StaleEntry(String),                   // LAYER_MAP key with no src module
    UpwardEdge { from: String, to: String, from_tier: Tier, to_tier: Tier },  // not in ACCEPTED
    StaleAccepted { from: String, to: String },   // ACCEPTED entry no longer an edge → prune it
    TangleGrew { tier: Tier, baseline: u32, actual: u32 },
}

/// Pure. No IO. The entire verdict.
fn check(
    modules: &BTreeSet<String>,
    edges: &BTreeSet<(String, String)>,
    map: &[(&str, Tier)],
    accepted: &[(&str, &str, &str)],
    tangle_baseline: &[(Tier, u32)],
) -> Vec<Violation>;
```

Shell (impure, thin):

```rust
fn discover_modules(src: &Path) -> BTreeSet<String>;        // top-level .rs + dirs under src/
fn extract_edges(src: &Path) -> BTreeSet<(String, String)>; // syn walk; cfg(test)-skipped; path nodes only
```

### 5.3 Data, State & Ownership

- **`LAYER_MAP` owns the tier assignment** (one authority; ADR-001 carries the rule
  + tier *definitions* and points here, §7 D2). No prose per-module table survives.
- **`ACCEPTED_VIOLATIONS` owns the frozen upward-edge baseline** — small, enumerated,
  each with a reason + follow-up. `StaleAccepted` forces pruning once an edge dies
  (the ratchet tightens automatically).
- **`TANGLE_BASELINE` owns the per-tier cycle ceiling** — frozen numbers; `TangleGrew`
  fires on any increase. Lowering them as cycles die is the intended maintenance.
- **Module = first path component under `src/`** (`catalog/scan.rs` → `catalog`);
  subdir files inherit; **intra-module edges ignored** (ADR-001: tier is module-level).
- **Edge = (owning-module, first-segment-after-`crate::`)**; deduped, self-edges
  dropped.
- **No runtime state**; `const` data + per-run locals.

### 5.4 Lifecycle, Operations & Dynamics

`check`:
1. Completeness: `LAYER_MAP` keys vs discovered `modules` → `Unclassified` /
   `StaleEntry`.
2. Direction: for each cross-tier edge `tier(t) > tier(s)`: if not in `accepted` →
   `UpwardEdge`. Any `accepted` entry not present in `edges` → `StaleAccepted`.
3. Tangle: for each tier, build the same-tier subgraph, Tarjan SCC, `tangle =
   Σ(size−1)` over non-trivial SCCs; if `> baseline` → `TangleGrew`.
4. Return all violations; the `#[test]` panics with a formatted report if non-empty.

`extract_edges`: per `src/**/*.rs`, `syn::parse_file`; a `Visit` impl that **does
not recurse into `#[cfg(test)]`-attributed items/mods** and collects only **path
nodes** (`ItemUse` trees + `ExprPath`/`TypePath`/`PatPath`) whose leading segment
is `crate` — *not* doc-comment or string content (F-5). Map each to
`(owning_module, first_seg_after_crate)`.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — `LAYER_MAP` keys == discovered modules (the map cannot silently rot as
  the ADR table did).
- **INV-2** — every cross-tier edge points down, modulo the frozen `ACCEPTED`
  baseline; the gate pins this going forward.
- **INV-3** — `tangle(tier)` never exceeds its frozen baseline (intra-tier cohesion
  may not worsen).
- **ASM-1** — cargo sets the test CWD to the package root (load-bearing; the
  pre-flight probe converts a violated assumption into a clear failure). Confirmed
  empirically PHASE-01.
- **ASM-2** — `crate::` path nodes capture the edges that matter; a re-export or
  macro-body edge could evade `syn` (§8 R2) — not present, not modelled now.
- **Edge — `#[cfg(test)]` upward edges / doc-link paths are not violations** (Visit
  skip + path-nodes-only handle both by construction: F-5, the `relation` test
  edges).
- **Edge — deep paths** (`crate::catalog::scan::f`) count as one edge to `catalog`.
- **Edge — `relation_graph`/`integrity` reaches** become intra-command (legal) under
  their command-tier classification — *not* baselined.

## 6. Open Questions & Unknowns

- **OQ-1 — exact tier for the ambiguous ~20** (`governance`, `value`, `lifecycle`,
  `conduct`, `links`, `dep_seq`, `projection`, `coverage*`, `dtoml`, …). Resolved in
  PHASE-01 by classifying *by what each knows*, with a rationale, then letting the
  gate confirm consistency. Not a downstream blocker.
- **OQ-2 — CHR-014 footgun.** *Resolved* (§5.1 / D4): CWD-relative read + probe.
- **OQ-3 — rule 3 (engine purity).** *Deferred* (§7 D3b); impure-leaf refinement is
  the named follow-up.
- **OQ-4 — final tangle metric.** `Σ(SCC_size−1)` proposed; PHASE-01 may add a
  secondary readout (largest-SCC / 2-cycle count) for diagnostics. Mechanism fixed,
  metric tunable.
- **OQ-5 — magnitude of the upward baseline.** *Unknown until PHASE-01* — and it is
  the go/no-go input (§7 D7, §8 R6).

## 7. Decisions, Rationale & Alternatives

- **D1 — Fitness gate now; engine crate split deferred.** *Alt:* crate split now
  (strongest, compiler-enforced) — rejected for this slice: heaviest, needs the
  authoritative map anyway, blocked by the very cycles below. *Alt:* both — rejected
  (couples a cheap durable win to a large risky move). **Chosen:** land the gate;
  `LAYER_MAP` de-risks the later cut (§8 R1).
- **D2 — Central Rust `const` map, not prose / per-module marker / TOML.** *Alts:*
  per-module `//! Tier:` (best cohesion, no survey point, 68 annotations); TOML
  (language-neutral, adds parse+IO for no gain); ADR prose table (the status quo
  that drifted; storage-rule-forbidden). **Chosen:** one greppable, compile-checked,
  reviewed table the gate reads with no IO; ADR-001 holds rule + definitions and
  points at it.
- **D3 — Enforce rule 1 *hard*; rule 2 as a *non-increasing tangle ratchet*.** The
  probe showed rule 2 is comprehensively violated intra-command and unfixable in
  scope. *Alt — gate global acyclicity (SCC size 1 everywhere):* unviable — you
  cannot baseline a ~33-node SCC as "debt." *Alt — ignore intra-tier cycles
  entirely:* rejected (false confidence; the blob festers — User). **Chosen:** hard
  directional gate + per-tier `Σ(SCC−1)` ratchet frozen at today's numbers,
  monotonic-down. Rule 2 is *enforced as "may not worsen,"* not abandoned.
- **D3b — Rule 3 (engine purity) deferred.** Audit found it well-honored; full
  purity detection is fuzzy. The impure-leaf refinement (tag `git`/`clock` impure,
  forbid engine→impure-leaf in the same framework) is the recorded follow-up.
- **D4 — Read `src/` relative to runtime CWD, not `env!`.** `env!` bakes the path at
  compile time → reused binary under shared target reads a stale worktree (CHR-014).
  Cargo's per-invocation CWD = package root tracks the worktree actually run; the
  probe (ASM-1) makes the guarantee loud if violated.
- **D5 — Amend ADR-001 in place; write via REV at reconcile.** *Alt — follow-on ADR:*
  rejected (fragments layering canon). This *is* ADR-001's own "Later — crate graph"
  plan maturing + a justified reversal of its homegrown-test rejection (cycles
  arrived; `syn` removes the brittleness that grounded it). The amendment: rule 1
  machine-enforced; rule 2 enforced as a ratchet with the command tangle openly
  recorded as unmet-and-tracked; stale prose table → tier definitions + `LAYER_MAP`
  pointer; crate split named as the future escalation; `input` reclassified engine.
- **D6 — `syn` as a test-only dev-dependency.** *Alt:* regex (no dep) — rejected:
  cannot cleanly skip `#[cfg(test)]`, and false-positives on `crate::` in
  doc-comments/strings (F-5) — the brittleness ADR-001 named. `syn` centralized in
  `[workspace.dependencies]` with a reason; `[dev-dependencies]` only → zero
  shipped-binary weight.
- **D7 — Classify-first spike as PHASE-01, gating gate construction.** The gate's
  value is unknown until the upward baseline is counted, and classification is
  gameable. PHASE-01 authors `LAYER_MAP` (with per-module rationale) + measures the
  upward baseline and per-tier tangle; a small baseline + meaningful engine core is
  the **go** condition for PHASE-02. Converts the top risk into a cheap checkpoint.
- **D8 — `ACCEPTED_VIOLATIONS` + `TANGLE_BASELINE` as frozen, prune-forced
  baselines.** Pre-existing debt is enumerated/counted and frozen, not silently
  tolerated: `StaleAccepted`/`StaleEntry`/`TangleGrew` force the baselines to tighten
  as debt dies and forbid growth. The ratchet, not a blanket waiver.

## 8. Risks & Mitigations

- **R1 (deferred crate split orphaned)** — landing only the gate could leave the
  boundary unmet. *Mit:* slice Follow-Ups name the crate split seeded by `LAYER_MAP`;
  ADR-001 keeps it as the named escalation. The gate is the interim ratchet + the
  de-risking map, not a replacement.
- **R2 (edge laundering / dynamic reach)** — a re-export or macro could route an
  upward edge `syn` misses (ASM-2). *Mit:* not present; `syn` sees literal `crate::`
  in use + expr/type position; re-export laundering still caught by review /
  `/inquisition`. Modelling now is speculative.
- **R3 (classify-to-green)** — tiers chosen to pass, not by what a module knows,
  yields a meaningless green. *Mit:* PHASE-01 authors a per-module rationale *before*
  green (§4, D7); a module classified solely to legalise an edge is flagged.
- **R4 (false green from a parser miss)** — `extract_edges` under-reporting passes
  vacuously. *Mit:* pure `check` unit-tested with synthetic upward edge + 2-cycle +
  unclassified + tangle-grew (VT-2); EX-1 hand-introduces a real upward edge and
  asserts `just gate` fails naming it. The gate is proven to *bite*.
- **R5 (ratchet rot / false confidence)** — baselines never shrink; the ADR reads
  "enforced" while the command blob festers. *Mit:* `ACCEPTED`/tangle entries cite
  follow-ups; `StaleAccepted` + monotonic-down force tightening; ADR-001 amendment
  states plainly that rule 2 is unmet-and-tracked, not satisfied.
- **R6 (gate value unknown)** — the upward baseline could be so large the gate is a
  fig leaf (OQ-5). *Mit:* PHASE-01 is the go/no-go (D7); a mostly-baseline result
  re-routes to `/consult` rather than auto-proceeding.

## 9. Quality Engineering & Validation

- **VT-1 (real-graph gate, primary).** `tests/architecture_layering.rs` asserts zero
  violations against the production graph. Green at PHASE-02 close (under the authored
  baselines); red on any future upward edge, accepted-set growth, tangle increase, or
  unclassified module. Runs under `just gate`.
- **VT-2 (bite-proof, pure unit tests).** `check` over synthetic inputs: legal → `[]`;
  upward edge (not accepted) → `UpwardEdge`; upward edge in `accepted` → `[]`;
  `accepted` entry absent from edges → `StaleAccepted`; intra-tier 2-cycle over
  baseline → `TangleGrew`; src-only → `Unclassified`; map-only → `StaleEntry`.
- **VT-3 (cfg(test) exclusion).** Fixture: a `#[cfg(test)]`-scoped `crate::<command>`
  path produces **no** edge (pins the production-only contract; the `relation` test
  edges).
- **VT-4 (doc-link exclusion).** Fixture: a `crate::X` reference inside a `///`
  intra-doc link produces **no** edge (F-5; the `git→retrieve` class).
- **EX-1** — hand-introduce a real upward edge (engine module `use`s a command module
  in non-test code) → `just gate` fails naming it; revert. Recorded, not committed.
- **EX-2** — `LAYER_MAP` keys == discovered modules (add/drop a module → red).
- **EX-3** — ADR-001 updated (rule 1 enforced; rule 2 ratcheted/tracked; table →
  definitions + `LAYER_MAP` pointer; `input` reclassified), written via REV at
  reconcile (D5).
- **EN-1** — PHASE-01 go condition met (small upward baseline, meaningful engine core).
  **EN-2** — `syn` dev-dep landed, `just gate` green.
- **Phases.** PHASE-01 classify-first spike (author `LAYER_MAP` + rationale; measure
  upward baseline + per-tier tangle; **go/no-go**). PHASE-02 build the gate (shell +
  pure `check` + VT-1..4, EX-1..2) against the PHASE-01 baselines. PHASE-03 ADR-001
  amendment via REV (reconcile).
- **`just gate`** green (clippy zero-warning, fmt, `test-all`).

## 10. Review Notes

### Internal adversarial pass (2026-06-19) — integrated

Empirical probe of the production `crate::` graph (regex preview, doc-links +
trailing test mods stripped). Findings, all integrated above:

- **F-1 — `relation_graph` has production upward reaches** (`governance/adr/policy/
  standard/backlog/spec`, inline calls). Not imported by engine-core → classified
  **command-tier**; reaches become intra-command (legal). Also confirms the `syn`
  walk must collect **expression-position** paths, not just `use` trees.
- **F-2 — `state → install`** is a real engine→command edge (ADR-001's wart). Non-Goals
  say classify-don't-resolve → seeded into `ACCEPTED_VIOLATIONS` with a follow-up.
- **F-3 — `integrity` reaches 11 command shells** (`*_KIND` view). Not imported by
  engine-core → **command-tier**; reaches become intra-command.
- **F-4 — the whole graph is not acyclic** (large intra-command SCC + many 2-cycles;
  engine core clean but `conduct↔dtoml` → CHR-015). Overturned the approved
  "no cycles (SCC size 1)" check → **hard directional gate + per-tier tangle ratchet**
  (D3); §2 corrected. Confirmed with the User as the only sane path; ratchet-by-count
  added at the User's prompt to keep rule 2 enforced-as-non-increasing rather than
  abandoned.
- **F-5 — doc-comment intra-doc links are false edges** (`git→retrieve`). `syn`
  path-nodes-only dodges them (VT-4); reinforces D6 over a regex gate.
- **F-6 — `input` is engine, not leaf** (imports `entity`+`meta`). ADR-001's table is
  *wrong*, not just stale → reclassified; amendment notes it.

Process: this pass also exposed that the **gate's value is unproven until
classification** and that classification is **gameable** → PHASE-01 classify-first
go/no-go (D7) + the per-module-rationale guard (R3).

### Doctrinal alignment

ADR-001 — the gate *implements* it: rule 1 hard-enforced; rule 2 enforced as a
non-increasing ratchet with the command tangle openly recorded as unmet-and-tracked
(no silent narrowing); rule 3 deferred with a named follow-up. ADR-013 — the ADR-001
amendment routes through a REV at reconcile (D5). Storage rule — tier assignment
moves from prose (forbidden) to a reviewed `const` (D2). Pure/imperative split
honored (§5.1). The rule-2 reframing is a governance reinterpretation confirmed with
the User (the ADR-001 authority); no unresolved governance conflict, no `/consult`
outstanding (the F-4 fork was raised and decided).

### External pass — pending

To run at the User's election after this draft locks: attack the production-only
scope (could a `cfg(test)`-only edge mask a shipping one?), the CWD guarantee
(ASM-1, confirm empirically), `syn` path-collection completeness (macro bodies),
the `LAYER_MAP`-as-canon claim vs the storage rule (is a Rust `const` "structured"
enough, or does canon want the map in `.doctrine`?), and whether the tangle ratchet
meaningfully constrains or just records.
