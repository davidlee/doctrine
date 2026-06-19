# Design SL-112: Machine-check ADR-001 layering via a `syn` dependency-fitness gate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-111, ADR-001, ADR-013, CHR-015); doc-local refs bare — OQ-1 (§6),
     D1 (§7), R1 (§8), VT-1/F-1/C1 (§9/§10). -->

## 1. Design Problem

ADR-001 (`leaf ← engine ← command, no cycles`) is **review-only**. The 2026-06-19
audit confirmed the drift the ADR predicted: engine→command cycles (broken by
SL-111). ADR-001 named two escalations — an engine crate, and a fitness function —
but its accepted Verification section **explicitly rejects** "a homegrown
module-graph unit test" as brittle. This slice **overturns that rejection** (the
cycles arrived, so the cost/benefit flipped; `syn` removes the brittleness that
grounded it) and lands the **fitness gate now**; the engine **crate split is
deferred** to a follow-on slice the map de-risks (§8 R1).

The design + two adversarial passes (§10) reshaped the gate from "assert the graph
is clean" into **"freeze the boundary and ratchet it"**: a hard directional gate
over literal `crate::` edges, a forcing function against laundering umbrellas, and
a numeric intra-tier cycle ratchet where a hard gate is not yet achievable.

## 2. Current State (measured, design probe 2026-06-19)

Probe of the production `crate::` graph (regex preview + codex source-verification;
the authoritative `syn` graph is PHASE-01). Several facts correct prior assumptions:

- **The whole graph is NOT acyclic.** SL-111 made the **engine tier** acyclic and
  broke the engine→command *upward* cycles; it never made the whole graph acyclic.
- **Engine core is a clean DAG.** Among `entity/registry/relation/meta/state/plan`:
  only `meta→entity`, `state→plan`, `status→meta` — all downward, zero back-edges.
  The one core cycle, `conduct ↔ dtoml`, is **broken (CHR-015, done)** → the engine
  tangle baseline is **0**.
- **Command tier is heavily cyclic** — a ~32-module intra-tier SCC + many 2-cycles.
  Normal for a CLI verb layer; cohesion per verb is fine (audit). Intra-tier — not a
  direction violation, out of scope to *resolve* (Non-Goals).
- **Leaf tier is clean** — `{clock, root, fsutil, kinds, git}` import only leaves.
- **ADR-001's tier table is wrong, not merely stale** — `input` is listed leaf but
  imports `entity`+`meta` → engine (F-6); the table predates ~47 modules.
- **Some top-level modules are altitude-mixed umbrellas.** `catalog/scan.rs`
  dispatches into 9 command modules while `catalog/{hydrate,graph,diagnostic,mod}.rs`
  are pure (codex-verified); `priority` is similarly muddy (`priority/graph.rs`
  consumes the `relation_graph` all-kind seam while `priority/mod.rs` claims engine).
  `relation_graph`/`integrity` reach command-domain wholesale. A module's
  doc-comment self-claim is **not** authoritative — these are handled by
  **variable granularity** (§5.5 D9): mixed umbrellas are sub-classified, and the
  gate *forces* that (a `MixedUmbrella` violation), so most-knowing-wins cannot
  silently launder a pure sub-file's upward edge.
- **`state → install`** is a genuine engine→command upward edge — ADR-001's wart;
  Non-Goals say classify+baseline, don't resolve (F-2).
- **Doc-comment intra-doc links are a false-edge source** (`git→retrieve`, F-5) —
  the `syn` path-node walk dodges them; a regex gate would not.
- **No production `macro_rules!` and no production wildcard `use crate::*`**
  (codex-verified) — the parser's blind spots have no present breach; the gate is
  literal-`crate::`-path only by construction (§3, F-4).

## 3. Forces & Constraints

- **ADR-001** — under enforcement. Rule 1 (downward) hard-gated over literal
  `crate::` edges; rule 2 (no cycles) comprehensively violated intra-command and
  unfixable in scope → **non-increasing cyclic-edge ratchet**, not abandoned (§7 D3).
  Rule 3 (purity) deferred (D3b). The slice **overturns** ADR-001's current
  rejection of this test; the amendment is a **co-requisite for close** (D5, F-5).
- **Storage rule** (AGENTS.md) — structured canon lives in TOML under `slice/nnn/`
  or `adr/nnn/`, never prose or code. The layer map + baselines are an **authored
  companion TOML at `.doctrine/adr/001/layering.toml`** — a surface under the
  governing ADR's own directory; the gate parses it (§7 D2, F-3 / C3).
- **ADR-001's brittleness objection** — answered by `syn` (canonical AST); verdict
  logic pure + unit-tested (D6). Honestly scoped to **literal `crate::` path edges**;
  macro/re-export laundering disclosed as out of reach (F-4).
- **`just gate`** runs `test-all` — the gate rides it. **Pure/imperative split** —
  impure fs+`syn`+toml read is a thin shell; verdict pure. **No parallel impl** —
  one gate, one authored map. **ADR-013** — amendment via REV at reconcile.
- **CHR-014** — `env!` bakes a stale path under shared target dirs → read relative
  to runtime CWD (D4; cargo's package-root test CWD codex-confirmed).
- **Gate value unproven until classification** — its worth is the count of genuine
  upward edges it catches in future; unknown until the map exists, classification is
  judgement-laden + gameable (§4, D7, §8 R3).

## 4. Guiding Principles

- **Classify-first (PHASE-01 go/no-go).** Author the map + per-unit rationale,
  measure the upward baseline + per-tier cyclic-edge count, *before* building the
  gate. Small baseline + meaningful engine core → proceed; mostly-baseline →
  `/consult` (D7).
- **Classify by what a unit *knows*; descend to the cohesive unit.** Default unit =
  top-level module, tier = highest altitude of any non-test file (most-knowing-wins).
  Where a module's files span altitudes (mixed umbrella), **sub-classify** at
  submodule granularity — and the gate *forces* this, so coarse granularity cannot
  launder (D9).
- **Hard-gate what you can; ratchet what you cannot; force what would launder.**
  Direction hard-gated; intra-tier cycles pinned by a monotonic-down cyclic-edge
  count; mixedness flagged as a forcing-function violation.
- **Govern the production graph only, literal paths only.** `#[cfg(test)]` + doc
  links out by construction; macro/re-export laundering disclosed, not pretended away.
- **The map is canon — authored TOML beside its governing ADR, self-enforcing.**
- **Pure core, thin shell, prove it bites** — negative self-tests so the gate cannot
  pass vacuously.

## 5. Proposed Design

### 5.1 System Model

`tests/architecture_layering.rs` is the gate, under `cargo test` (hence
`just gate`). Impure shell → pure verdict:

```
  src/**/*.rs ─syn; skip cfg(test); literal crate:: path-nodes─▶ (units, edges)  ┐ [shell]
  .doctrine/adr/001/layering.toml ─toml─▶ (map, accepted, tangle_baseline)        ┘
                                                                   │
                                          check(units, edges, map, accepted, base) [pure]
                                                                   │
                              empty ⇒ pass    │    non-empty ⇒ panic! naming each item
```

Four assertions:
1. **Completeness** — discovered units == map keys.
2. **Cross-tier direction (hard)** — every edge `s→t` with `tier(t) > tier(s)` is a
   violation, unless in the frozen, enumerated `ACCEPTED_VIOLATIONS`.
3. **Mixed-umbrella forcing** — for any top-level module *not* sub-classified, if a
   non-test file under it reaches a tier **above** the module's assigned tier →
   `MixedUmbrella { module, file, reaches }` (sub-classify it, or baseline the
   edge). Kills most-knowing-wins laundering (C2).
4. **Per-tier cyclic-edge ratchet** — `cyclic_edges(tier) ≤ TANGLE_BASELINE[tier]`,
   where `cyclic_edges` = count of same-tier edges with both endpoints in one
   non-trivial SCC of the same-tier subgraph. Monotonic-down.

Source + TOML read relative to **runtime CWD** (package root, per cargo; D4); a
pre-flight asserts `./src`, `./Cargo.toml`, `./.doctrine/adr/001/layering.toml`
exist with a clear "run from package root" message.

### 5.2 Interfaces & Contracts

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier { Leaf = 0, Engine = 1, Command = 2 }   // ordinal = altitude

// Parsed from layering.toml. A "unit" is a top-level module, OR a `top::sub`
// submodule path for a sub-classified mixed umbrella.
struct LayerMap(BTreeMap<String, Tier>);           // unit → tier
struct Accepted(BTreeSet<(String, String)>);       // frozen upward-edge baseline
struct TangleBaseline(BTreeMap<Tier, u32>);        // frozen per-tier cyclic-edge ceiling

#[derive(Debug, PartialEq, Eq)]
enum Violation {
    Unclassified(String),
    StaleEntry(String),
    UpwardEdge { from: String, to: String, from_tier: Tier, to_tier: Tier },
    StaleAccepted { from: String, to: String },
    MixedUmbrella { module: String, file: String, reaches: Tier },
    TangleGrew { tier: Tier, baseline: u32, actual: u32 },
}

/// Pure. No IO. The entire verdict.
fn check(units: &Units, edges: &BTreeSet<(String,String)>,
         map: &LayerMap, accepted: &Accepted, base: &TangleBaseline) -> Vec<Violation>;
```

Shell: `discover_units` (top-level modules + the sub-classified submodules named in
the map), `extract_edges` (`syn` walk; keys each edge to the **finest classified
unit** owning the source file), `load_layering` (toml).

### 5.3 Data, State & Ownership

- **`.doctrine/adr/001/layering.toml` (authored, committed, under the governing ADR)
  owns the canon** — a `*.toml` under `adr/nnn/`, the storage rule's authored
  surface (F-3 / C3); a *companion* to `adr-001.toml`, so it does not perturb the
  strict `schema = "doctrine.adr"`. ADR-001's MD carries the rule + tier definitions
  and **points at this file**. Editing the map = a governance act, routed via REV.
  Shape:
  ```toml
  # tier = leaf | engine | command. Unit = top-level module, or `top::sub` for a
  # sub-classified mixed umbrella (design §5.5 / D9).
  [tiers]
  clock = "leaf"
  entity = "engine"
  input  = "engine"               # F-6: ADR-001 table wrongly said leaf
  "catalog::scan" = "command"     # mixed umbrella, sub-classified (C2/round-2)
  "catalog::hydrate" = "engine"
  "catalog::graph" = "engine"
  "catalog::diagnostic" = "engine"
  install = "command"
  relation_graph = "command"
  integrity = "command"
  # … every unit
  [[accepted_violation]]          # frozen; MAY shrink, MUST NOT grow
  from = "state"; to = "install"; reason = "ADR-001 install-as-utility wart"; follow_up = "<id>"
  [tangle_baseline]               # per-tier cyclic-edge ceiling; frozen at PHASE-01
  leaf = 0
  engine = 0                      # conduct↔dtoml broken (CHR-015)
  command = 0                     # measured PHASE-01
  ```
- **Default unit = first path component under `src/`**; sub-classified umbrellas add
  `top::sub` units. Edges keyed to the finest owning unit; deeper paths and
  intra-unit edges ignored.
- **No runtime state** — authored TOML + per-run locals.

### 5.4 Lifecycle, Operations & Dynamics

`check`: (1) completeness; (2) direction → `UpwardEdge`/`StaleAccepted`;
(3) per top-level module not sub-classified, each non-test file's max outbound tier
vs the module tier → `MixedUmbrella`; (4) per-tier same-tier subgraph, Tarjan SCC,
count edges inside non-trivial SCCs → `TangleGrew`. Panic with a formatted report
if non-empty.

`extract_edges`: per `src/**/*.rs`, `syn::parse_file`; a `Visit` that **skips
`#[cfg(test)]`** items/mods and collects only **path nodes** (`ItemUse` +
`ExprPath`/`TypePath`/`PatPath`) with leading `crate` — not doc-comment/string
content. Macro-body / re-export paths out of reach (F-4) — disclosed.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1/2/3** — map keys == units; cross-tier edges downward modulo frozen
  `ACCEPTED`; `cyclic_edges(tier) ≤ baseline`.
- **D9 variable granularity + mixed-umbrella forcing** — default top-level,
  most-knowing-wins; a module whose files span altitudes is *forced* to
  sub-classify (assertion 3). This closes the round-1 "umbrella launders a pure
  sub-file's upward edge" hole at its source rather than hand-waving it to unrelated
  slices. **Residual:** a *uniformly*-command umbrella with internal command↔command
  cycles still only ratchets, not splits — acceptable (intra-tier is out of scope).
- **Boundary-stability caveat (C2/round-2)** — the cyclic-edge count is meaningful
  only under a **stable module partition**: folding two units hides their edges as
  intra-unit; splitting one surfaces new counted edges. *Mit:* the partition *is*
  `layering.toml` — a fold/split is a reviewed canon edit (REV-routed) + visible
  directory surgery, not a silent dodge; `MixedUmbrella` catches folds that bury an
  upward reach. Disclosed as R8, not silently gameable.
- **ASM-1** — cargo sets the test CWD to the package root (codex-confirmed vs the
  cargo manual); the pre-flight probe makes a violation loud.
- **ASM-2** — literal `crate::` path nodes capture the edges that matter; macro /
  re-export laundering can evade `syn` (F-4) — no present breach, review-covered.
- **Edge — `#[cfg(test)]` + doc-link paths are not violations** (F-5).
- **Edge — doc-comment fixes** — `catalog/mod.rs:2`, `relation_graph.rs:2` claim
  engine but are command/mixed; corrected as part of the slice.

## 6. Open Questions & Unknowns

- **OQ-1 — exact tier for the ambiguous units** (`governance`, `value`, `lifecycle`,
  `conduct`, `links`, `dep_seq`, `projection`, `coverage*`, `priority::*`, …).
  PHASE-01 classifies most-knowing-wins with rationale; the gate confirms; the
  `MixedUmbrella` check surfaces any not-yet-sub-classified umbrella.
- **OQ-2 — CHR-014.** *Resolved* (D4); cargo CWD codex-confirmed.
- **OQ-3 — rule 3.** *Deferred* (D3b).
- **OQ-4 — tangle metric.** *Resolved*: per-tier cyclic-edge count (C1).
- **OQ-5 — upward-baseline magnitude.** *Unknown until PHASE-01* — the go/no-go input.
- **OQ-6 — canon home.** *Resolved*: `.doctrine/adr/001/layering.toml` (companion
  authored surface under the governing ADR; C3). No longer deferred to PHASE-02.

## 7. Decisions, Rationale & Alternatives

- **D1 — Fitness gate now; crate split deferred.** *Alts:* split now (heaviest,
  needs the map, blocked by the cycles); both (couples cheap win to risky move).
  **Chosen:** gate now; map de-risks the cut (R1).
- **D2 — Canon in `.doctrine/adr/001/layering.toml` (companion authored TOML), test
  parses it.** *Evolved C3→round-2:* an earlier "Rust `const`" was code-not-canon
  (storage rule); a *root* `layers.toml` was committed config but not a doctrine
  authored surface, and left the home contradictory. **Locked:** a `*.toml` under
  the governing ADR's `adr/001/` dir — storage-rule-authored, governance-routed,
  and clear of `adr-001.toml`'s strict schema. ADR-001 MD points at it.
- **D3 — Rule 1 *hard*; rule 2 as a non-increasing *cyclic-edge* ratchet.** *Metric
  fixed after C1:* count same-tier edges inside non-trivial SCCs (catches a new bad
  edge inside an existing blob, which `Σ(SCC−1)` missed). *Alts:* global acyclicity
  (can't baseline a 32-node SCC); ignore intra-tier (false confidence). Caveat:
  meaningful only under a stable partition (R8).
- **D3b — Rule 3 deferred** (impure-leaf refinement is the follow-up).
- **D4 — Runtime-CWD read, not `env!`** — sidesteps CHR-014; cargo CWD codex-confirmed.
- **D5 — Slice *overturns* ADR-001's rejection; amendment is a close co-requisite,
  via REV.** *Corrected after C5:* the design does not "implement" an ADR that still
  rejects this test — that is a live contradiction until amended, so the slice
  reverses the rejection and the amendment gates close (EX-3), not optional polish.
- **D6 — `syn` test-only dev-dep** (regex rejected: can't skip cfg(test), false-pos
  on doc/string `crate::`). Centralized in `[workspace.dependencies]`; dev-only.
- **D7 — Classify-first spike (PHASE-01), gating gate construction** — gate value
  unknown until the baseline is counted; classification gameable.
- **D8 — Frozen, prune-forced baselines** (`StaleAccepted`/`StaleEntry`/`TangleGrew`
  force tightening, forbid growth).
- **D9 — Variable granularity + mixed-umbrella forcing.** *Strengthened after C2:*
  default top-level most-knowing-wins, but a module whose files span altitudes is
  *forced* to sub-classify (assertion 3) — so coarse granularity cannot launder a
  pure sub-file's upward edge. *Alt — full per-file granularity:* rejected for
  appetite (~200 units); the forcing function gets the honesty without the bulk.
  *Rejected earlier claim:* "SL-115/116 decompose the umbrellas" was false (they
  split `main.rs`/`worktree.rs`, not `catalog`/`priority`) — dropped.

## 8. Risks & Mitigations

- **R1 (crate split orphaned)** — Follow-Ups name it, seeded by the map.
- **R2 (laundering / dynamic reach)** — macro/re-export/wildcard could evade `syn`.
  *Mit:* no present breach (codex-verified); review covers the residue; claim
  narrowed to "literal `crate::` path gate" (F-4).
- **R3 (classify-to-green)** — *Mit:* most-knowing-wins + the `MixedUmbrella` forcing
  function make "relabel command to hide" fail unless a command file genuinely
  exists; per-unit rationale before green; lying doc-comments fixed.
- **R4 (false green from a parser miss)** — *Mit:* pure `check` unit-tested with
  synthetic upward edge + intra-SCC new edge + mixed umbrella + tangle-grew (VT-2);
  EX-1 hand-introduces a real upward edge → gate fails naming it.
- **R5 (ratchet rot / overclaim)** — *Mit:* baselines cite follow-ups; monotonic-down
  + `StaleAccepted` force tightening; ADR amendment states rule 2 unmet-and-tracked;
  gate named "literal `crate::` path gate" (F-4).
- **R6 (gate value unknown)** — *Mit:* PHASE-01 go/no-go; mostly-baseline → `/consult`.
- **R7 (umbrella laundering)** — *Closed* by D9's `MixedUmbrella` forcing function:
  a mixed umbrella must sub-classify or fail. (Was a BLOCKER round-2; the
  unrelated-slice mitigation it relied on is dropped.)
- **R8 (boundary manipulation)** — folding units can hide cyclic edges; splitting
  inflates the count. *Mit:* the partition is reviewed canon (`layering.toml`,
  REV-routed) + visible directory diff; `MixedUmbrella` catches edge-burying folds.
  Disclosed; the ratchet assumes a stable partition.

## 9. Quality Engineering & Validation

- **VT-1 (real-graph gate, primary)** — zero violations against the production graph
  under the authored baselines; red on any future upward edge, accepted growth,
  cyclic-edge increase, new mixed umbrella, or unclassified unit. Under `just gate`.
- **VT-2 (bite-proof, pure unit tests)** — `check` over synthetic inputs: legal →
  `[]`; upward edge (not accepted) → `UpwardEdge`; accepted → `[]`; **new edge inside
  an existing same-tier SCC → `TangleGrew`** (C1 case); **a module file reaching above
  its module tier, not sub-classified → `MixedUmbrella`** (C2 case); accepted-absent →
  `StaleAccepted`; src-only → `Unclassified`; map-only → `StaleEntry`.
- **VT-3 (cfg(test) exclusion)** / **VT-4 (doc-link exclusion)** — fixtures → no edge.
- **EX-1** — hand-introduce a real upward edge in non-test code → `just gate` fails
  naming it; revert. **EX-2** — map keys == units (add/drop → red). **EX-3** — ADR-001
  amended (rejection overturned; rule 1 enforced; rule 2 ratcheted/tracked; table →
  definitions + `layering.toml` pointer; `input` reclassified), via REV — a close
  co-requisite (D5).
- **EN-1** — PHASE-01 go condition met. **EN-2** — `syn` dev-dep landed, gate green.
- **Phases.** PHASE-01 classify-first spike (author `layering.toml` + rationale;
  sub-classify mixed umbrellas; measure upward baseline + per-tier cyclic-edge count;
  **go/no-go**). PHASE-02 build the gate (shell + pure `check` + VT-1..4, EX-1..2).
  PHASE-03 ADR-001 amendment via REV (co-requisite).
- **`just gate`** green.

## 10. Review Notes

### Internal adversarial pass (2026-06-19) — integrated

Probe of the production graph. **F-1** `relation_graph` upward reaches → command
(confirms expr-position paths needed). **F-2** `state→install` baselined. **F-3**
`integrity` → command. **F-4** whole graph not acyclic → directional gate + cyclic
ratchet; §2 corrected. **F-5** doc-link false edges → `syn` dodges. **F-6** `input`
is engine. Process: gate value unproven until classification + gameable → PHASE-01
go/no-go (D7) + per-unit rationale (R3).

### External pass 1 — codex/GPT-5.5 (source-verified)

**C1** tangle metric `Σ(SCC−1)` missed new edges inside an existing SCC → per-tier
**cyclic-edge count** (D3/OQ-4). **C2** top-level granularity launders umbrellas →
most-knowing-wins + (round-2) the `MixedUmbrella` forcing function (D9). **C3**
`const`-as-canon violated the storage rule → authored TOML (D2). **C4** overclaim →
"literal `crate::` path gate" (F-4). **C5** ADR-001 currently rejects this test →
slice **overturns** it, amendment co-requisite (D5). ASM-1 (CWD) confirmed.

### External pass 2 — codex/GPT-5.5 (source-verified, on the revised mechanism)

- **C1-fix CONFIRMED** — the cyclic-edge metric counts a new intra-SCC edge and a
  bidirectional merge sanely; no paradox. Remaining caveat: gameable by
  module-boundary surgery (intra-unit edges ignored) → disclosed as **R8** +
  boundary-stability caveat (§5.5); the partition is reviewed canon.
- **C2 round-2 BLOCKER — the `catalog` umbrella hole was real and my SL-115/116
  mitigation was fiction** (they split `main.rs`/`worktree.rs`, not `catalog`).
  Source-verified: `catalog/scan.rs` reaches 9 command modules, `hydrate/graph/
  diagnostic/mod` pure. *Fix:* dropped the false mitigation; added the
  **`MixedUmbrella` forcing function** (assertion 3, D9) — a mixed umbrella must
  sub-classify (e.g. `catalog::scan`=command, `catalog::hydrate`=engine) or the gate
  fails. Closes R7 at its source. `priority` flagged as the other known mixed unit.
- **C3 round-2 MAJOR — `layers.toml` at repo root not a doctrine authored surface,
  and the design was self-contradictory** (§5.3 "owns canon" vs OQ-6/PHASE-02
  "unresolved home"). *Fix:* **locked** to `.doctrine/adr/001/layering.toml` (a
  `*.toml` under the governing ADR's dir = storage-rule authored surface, companion
  to the strict-schema `adr-001.toml`); OQ-6 resolved (D2).
- **C4 round-2 MINOR — slice text softer than design on the ADR co-requisite** →
  slice scope tightened to state the reversal is required for closure.
- Note: codex held the doc-comment self-claim authoritative; it is not — but the
  underlying umbrella-heterogeneity hole was real and is now closed by forcing.

### Doctrinal alignment

ADR-001 — the slice **overturns** its rejection and amends it (co-requisite,
EX-3/D5); rule 1 hard-gated (literal paths), rule 2 a non-increasing cyclic-edge
ratchet with the command tangle openly unmet-and-tracked, rule 3 deferred. ADR-013 —
amendment via REV. Storage rule — canon in an authored TOML under `adr/001/` (D2),
not prose, not code. Pure/imperative split honored. The F-4 rule-2 reframing was
decided by the User (ADR-001 authority); no `/consult` outstanding.
