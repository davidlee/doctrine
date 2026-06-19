# Design SL-112: Machine-check ADR-001 layering via a `syn` dependency-fitness gate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-111, SL-115, SL-116, ADR-001, ADR-013, CHR-015); doc-local refs bare —
     OQ-1 (§6), D1 (§7), R1 (§8), VT-1/F-1 (§9/§10). -->

## 1. Design Problem

ADR-001 (`leaf ← engine ← command, no cycles`) is **review-only**. The 2026-06-19
architecture audit confirmed the drift the ADR predicted: engine→command cycles
(broken by SL-111). ADR-001 named two escalations — an engine crate, and a fitness
function — but its accepted Verification section **explicitly rejects** "a homegrown
module-graph unit test" as brittle, deferring to the crate boundary. This slice
**overturns that rejection** (justified: the cycles arrived, so the cost/benefit
flipped, and `syn` removes the brittleness that grounded it) and lands the **fitness
gate now**; the engine **crate split is deferred** to a follow-on slice — the layer
map this slice authors de-risks the later cut (§8 R1).

The design + adversarial passes (§10) reshaped the gate from "assert the graph is
clean" into **"freeze the current boundary and ratchet it"**: a hard directional
gate over literal `crate::` edges, plus a numeric intra-tier cycle ratchet where a
hard gate is not yet achievable.

## 2. Current State (measured, design probe 2026-06-19)

Empirical probe of the production `crate::` graph (regex preview; the authoritative
`syn` graph is PHASE-01). Several facts correct earlier assumptions:

- **The whole graph is NOT acyclic.** SL-111 made the **engine tier** acyclic and
  broke the engine→command *upward* cycles; it never made the whole graph acyclic.
- **Engine core is essentially a clean DAG.** Among
  `entity/registry/relation/meta/state/plan`: only `meta→entity`, `state→plan`,
  `status→meta` — all downward, **zero back-edges**. The one real core cycle is
  **`conduct ↔ dtoml`** → captured as **CHR-015** (out of scope; the ratchet pins it).
- **Command tier is heavily cyclic** — a ~32-module intra-tier SCC + many 2-cycles.
  Normal for a CLI verb layer; **cohesion per verb is fine** (audit). Intra-tier —
  not a direction violation, and out of scope to *resolve* (Non-Goals).
- **Leaf tier is clean** — `{clock, root, fsutil, kinds, git}` import only leaves.
- **ADR-001's tier table is wrong, not merely stale** — it lists `input` as leaf,
  but `input` imports `entity`+`meta` (engine) → `input` is **engine** (F-6). The
  prose table also predates ~47 modules.
- **Umbrella modules carry mixed altitudes.** `catalog/mod.rs:2` and
  `relation_graph.rs:2` *claim* engine-tier in their doc-comments, yet
  `catalog/scan.rs` dispatches into `slice/governance/spec/knowledge/review/rec/
  revision/backlog` and `relation_graph` reaches `governance/adr/policy/standard/
  backlog/spec`; `integrity` imports 11 command shells (the `*_KIND` view). A
  module's self-claim is not authoritative — these operate at the **command**
  altitude (they *know* command-domain). The doc-comment claims are corrected
  (§5.5); the modules classify command-tier (F-1, F-3, F-2).
- **`state → install`** is a genuine engine→command upward edge — ADR-001's own
  wart; Non-Goals say *classify+baseline, don't resolve* (F-2).
- **Doc-comment intra-doc links are a false-edge source** — `git → retrieve` was a
  rustdoc `///` link, not a code edge (F-5). The `syn` path-node walk dodges these;
  a regex gate would not.
- **No production `macro_rules!` and no production wildcard `use crate::*`** exist
  (codex-verified, §10) — so the parser's known blind spots have **no present
  breach**, but the gate is literal-`crate::`-path only by construction (§3, F-4).

## 3. Forces & Constraints

- **ADR-001** — the rule under enforcement. Rule 1 (downward) is hard-gated over
  literal `crate::` edges; rule 2 (no cycles) is comprehensively violated
  intra-command and unfixable in scope → enforced as a **non-increasing ratchet**,
  not abandoned (§7 D3). Rule 3 (engine purity) deferred (§7 D3b). The slice
  **overturns** ADR-001's current rejection of this test; the amendment is a
  **co-requisite**, not a downstream nicety (§7 D5, F-5).
- **Storage rule** (AGENTS.md) — structured/queried canon lives in TOML, never
  prose or buried in code. The layer map + baselines are **authored TOML**, the
  test reads them (§7 D2, F-3) — not the prose ADR table, not a code `const`.
- **ADR-001's brittleness objection** — answered by `syn` (canonical AST) over
  regex, verdict logic pure + unit-tested (§7 D6). The gate is honestly scoped to
  **literal `crate::` path edges**; macro-body / re-export laundering is out of
  reach and remains a review concern (F-4).
- **`just gate`** runs `test-all` (`cargo test --workspace`) — the gate rides it.
- **Pure/imperative split** — impure fs+`syn`+TOML read is a thin shell; verdict pure.
- **No parallel implementation** — one gate, one authored map + baselines.
- **ADR-013** — the ADR-001 amendment is authored via a REV at reconcile.
- **CHR-014** — `env!("CARGO_MANIFEST_DIR")` bakes a stale path under shared target
  dirs → read relative to runtime CWD (§7 D4; cargo's documented test CWD =
  package root, codex-confirmed §10).
- **Gate value unproven until classification** — its worth is the count of genuine
  upward edges it catches in future; unknown until the map exists, and
  classification is judgement-laden + gameable (§4, §7 D7, §8 R3).

## 4. Guiding Principles

- **Classify-first, then decide (PHASE-01 go/no-go).** Author the real layer map +
  per-module rationale, measure the upward baseline + per-tier cyclic-edge count,
  *before* building the gate. Small upward baseline + meaningful engine core →
  proceed; mostly-baseline → `/consult`, not a fig leaf (§7 D7).
- **Classify by what a module *knows*, most-knowing-wins.** A top-level module's
  tier = the **highest altitude any of its non-test files operates at** (§7 D9). An
  umbrella with one command file *is* command — honest, not laundered. A module's
  own doc-comment is not authoritative; fix the ones that lie.
- **Hard-gate what you can; ratchet what you cannot.** Cross-tier direction is
  hard-gated. Intra-tier cycles (unfixable in scope) are pinned by a numeric
  monotonic-down ratchet on **cyclic-edge count** (§7 D3) — frozen, never grown.
- **Govern the production graph only, literal paths only.** Same altitude as
  `clippy`; `#[cfg(test)]` edges + doc-comment links out of contract by
  construction; macro/re-export laundering named as out-of-reach, not pretended away.
- **The map is canon — authored TOML, in one place, self-enforcing.** A module
  absent from it (or absent from `src/`) fails the gate.
- **Pure core, thin shell, prove it bites.** Verdict logic pure + unit-tested with
  negative self-tests so the gate cannot pass vacuously.

## 5. Proposed Design

### 5.1 System Model

`tests/architecture_layering.rs` is the gate, run under `cargo test` (hence
`just gate`). Impure shell → pure verdict:

```
  src/**/*.rs ─syn; skip cfg(test); literal crate:: path-nodes─▶ (modules, edges)  ┐ [shell]
  layers.toml ─toml parse────────────────────▶ (LAYER_MAP, ACCEPTED, TANGLE_BASE)  ┘
                                                                   │
                                          check(modules, edges, map, accepted, base) [pure]
                                                                   │
                              empty ⇒ pass    │    non-empty ⇒ panic! naming each item
```

Three assertions:
1. **Completeness (forcing)** — discovered top-level modules == `LAYER_MAP` keys.
2. **Cross-tier direction (hard)** — every edge `s→t` with `tier(t) > tier(s)` is a
   violation, unless in the frozen, enumerated `ACCEPTED_VIOLATIONS`.
3. **Per-tier cyclic-edge ratchet** — for each tier, `cyclic_edges(tier) ≤
   TANGLE_BASELINE[tier]`, where `cyclic_edges` = count of same-tier edges whose
   both endpoints lie in one non-trivial SCC of the same-tier subgraph.
   Monotonic-down.

Source + TOML read relative to **runtime CWD** (package root, per cargo), not
`env!` (§7 D4). A pre-flight asserts `./src` + `./Cargo.toml` + `./layers.toml`
exist with a clear "run from package root" message.

### 5.2 Interfaces & Contracts

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier { Leaf = 0, Engine = 1, Command = 2 }   // ordinal = altitude

// Parsed from authored layers.toml (§5.3); shapes the pure check input.
struct LayerMap(BTreeMap<String, Tier>);
struct Accepted(BTreeSet<(String, String)>);       // frozen upward-edge baseline
struct TangleBaseline(BTreeMap<Tier, u32>);        // frozen per-tier cyclic-edge ceiling

#[derive(Debug, PartialEq, Eq)]
enum Violation {
    Unclassified(String),                 // src module not in map
    StaleEntry(String),                   // map key with no src module
    UpwardEdge { from: String, to: String, from_tier: Tier, to_tier: Tier },  // not accepted
    StaleAccepted { from: String, to: String },     // accepted entry no longer an edge → prune
    TangleGrew { tier: Tier, baseline: u32, actual: u32 },
}

/// Pure. No IO. The entire verdict.
fn check(
    modules: &BTreeSet<String>,
    edges: &BTreeSet<(String, String)>,
    map: &LayerMap,
    accepted: &Accepted,
    base: &TangleBaseline,
) -> Vec<Violation>;
```

Shell (impure, thin): `discover_modules(&Path) -> BTreeSet<String>`;
`extract_edges(&Path) -> BTreeSet<(String,String)>` (`syn` walk); `load_layers(&Path)
-> (LayerMap, Accepted, TangleBaseline)` (toml).

### 5.3 Data, State & Ownership

- **`layers.toml` (authored, repo root, committed/reviewed) owns the canon** —
  beside `clippy.toml`/`doctrine.toml`; structured per the storage rule (F-3), not
  an ADR prose table, not a code `const`. ADR-001 carries the rule + tier
  *definitions* and points at this file. Shape:
  ```toml
  # tier = leaf | engine | command. A module's tier = highest altitude any of its
  # non-test files operates at (most-knowing-wins; design §5.5 / D9).
  [tiers]
  clock = "leaf"
  entity = "engine"
  input = "engine"          # F-6: ADR-001 table wrongly said leaf
  catalog = "command"       # F-2: umbrella; scan.rs knows command-domain
  relation_graph = "command"
  integrity = "command"
  install = "command"
  # … every top-level module
  [[accepted_violation]]    # frozen; MAY shrink, MUST NOT grow
  from = "state"; to = "install"; reason = "ADR-001 install-as-utility wart"; follow_up = "<backlog id>"
  [tangle_baseline]         # per-tier cyclic-edge ceiling; frozen at PHASE-01
  leaf = 0
  engine = 1                # conduct↔dtoml, CHR-015
  command = 0               # measured PHASE-01
  ```
- **Module = first path component under `src/`** (`catalog/scan.rs` → `catalog`);
  subdir files inherit; intra-module edges ignored.
- **Edge = (owning-module, first-segment-after-`crate::`)**; deduped; self-edges dropped.
- **No runtime state** — authored TOML + per-run locals.

### 5.4 Lifecycle, Operations & Dynamics

`check`:
1. Completeness: map keys vs discovered modules → `Unclassified` / `StaleEntry`.
2. Direction: each cross-tier edge `tier(t) > tier(s)`: if not in `accepted` →
   `UpwardEdge`. Any `accepted` not in `edges` → `StaleAccepted`.
3. Ratchet: per tier, build same-tier subgraph, Tarjan SCC, count edges with both
   endpoints in one non-trivial SCC; if `> baseline` → `TangleGrew`.
4. Return all; the `#[test]` panics with a formatted report if non-empty.

`extract_edges`: per `src/**/*.rs`, `syn::parse_file`; a `Visit` that **skips
`#[cfg(test)]`** items/mods and collects only **path nodes** (`ItemUse` trees +
`ExprPath`/`TypePath`/`PatPath`) with leading segment `crate` — not doc-comment or
string content. Macro-body / re-export paths are out of reach (F-4) — disclosed,
not silently dropped.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — `LAYER_MAP` keys == discovered modules.
- **INV-2** — every cross-tier edge points down, modulo the frozen `ACCEPTED` baseline.
- **INV-3** — `cyclic_edges(tier)` never exceeds its frozen baseline.
- **D9 most-knowing-wins** — a top-level module's tier is the highest altitude any
  non-test file under it operates at. Consequence + **named limitation (F-2)**: a
  mostly-engine umbrella with a single command file is painted command, so an
  *upward edge from its pure parts* would read as intra-command and escape the
  gate. Mitigated by the umbrella-decomposition slices (SL-115 main, SL-116
  worktree) that split these, and by the ratchet; honestly disclosed, not hidden.
- **ASM-1** — cargo sets the test CWD to the package root (codex-confirmed against
  the cargo manual, §10); the pre-flight probe makes a violation a clear failure.
- **ASM-2** — literal `crate::` path nodes capture the edges that matter; macro /
  re-export laundering can evade `syn` (F-4) — no present breach (no production
  `macro_rules!` / wildcard imports), review-covered.
- **Edge — `#[cfg(test)]` + doc-link paths are not violations** (Visit skip +
  path-nodes-only; F-5, the `relation` test edges).
- **Edge — doc-comment fixes** — `catalog/mod.rs:2`, `relation_graph.rs:2` claim
  engine but are command-tier; their doc-comments are corrected as part of the slice
  (a self-claim that contradicts the authoritative tier is a defect).

## 6. Open Questions & Unknowns

- **OQ-1 — exact tier for the ambiguous ~20** (`governance`, `value`, `lifecycle`,
  `conduct`, `links`, `dep_seq`, `projection`, `coverage*`, `dtoml`, …). Resolved in
  PHASE-01 by classifying *most-knowing-wins* with a rationale; the gate confirms
  consistency. Not a downstream blocker.
- **OQ-2 — CHR-014 footgun.** *Resolved* (§5.1 / D4): CWD-relative read + probe;
  cargo's CWD guarantee codex-confirmed.
- **OQ-3 — rule 3 (engine purity).** *Deferred* (§7 D3b).
- **OQ-4 — tangle metric.** *Resolved*: per-tier **cyclic-edge count** (edges inside
  non-trivial same-tier SCCs), replacing `Σ(SCC−1)` which missed new edges inside an
  existing blob (F-1). PHASE-01 may add largest-SCC as a diagnostic readout.
- **OQ-5 — magnitude of the upward baseline.** *Unknown until PHASE-01* — the
  go/no-go input (§7 D7, §8 R6).
- **OQ-6 — exact `layers.toml` home.** Proposed repo root (beside `clippy.toml`);
  a `[layering]` table in `doctrine.toml` is the alternative. Settle in PHASE-02.

## 7. Decisions, Rationale & Alternatives

- **D1 — Fitness gate now; engine crate split deferred.** *Alts:* crate split now
  (strongest, heaviest, needs the map anyway, blocked by the cycles); both (couples
  a cheap win to a risky move). **Chosen:** land the gate; `LAYER_MAP` de-risks the
  later cut (§8 R1).
- **D2 — Layer map + baselines in authored TOML (`layers.toml`), test reads it.**
  *Reversed from an earlier "Rust `const`" draft after F-3:* a `const` in test code
  is implementation, not authored canon; the storage rule puts structured canon in
  TOML. *Alts:* per-module `//! Tier:` marker (cohesive, no survey point, 68
  annotations); ADR prose table (drifted, forbidden). **Chosen:** one reviewed TOML
  the gate parses; ADR-001 holds the rule + definitions and points at it.
- **D3 — Rule 1 *hard*; rule 2 as a *non-increasing cyclic-edge ratchet*.** The
  probe showed rule 2 comprehensively violated intra-command, unfixable in scope.
  *Alts:* global acyclicity (unviable — can't baseline a 32-node SCC); ignore
  intra-tier cycles (false confidence — the blob festers). **Chosen (metric fixed
  after F-1):** count same-tier edges inside non-trivial SCCs, per tier, frozen,
  monotonic-down — catches a *new* bad edge inside an existing blob (which
  `Σ(SCC−1)` missed).
- **D3b — Rule 3 deferred.** Audit found it well-honored; impure-leaf refinement is
  the named follow-up.
- **D4 — Read relative to runtime CWD, not `env!`.** Sidesteps CHR-014; cargo's
  package-root CWD codex-confirmed; the probe makes the guarantee loud.
- **D5 — Slice *overturns* ADR-001's rejection; amendment is a co-requisite, via
  REV at reconcile.** *Corrected after F-5:* the design does not "implement" ADR-001
  while ADR-001 still rejects this test — that is a live contradiction until amended.
  So the slice states plainly it **reverses** the rejection (cycles arrived; `syn`
  de-brittles), and the amendment (rule 1 enforced; rule 2 ratcheted + command
  tangle recorded unmet-and-tracked; table → definitions + `layers.toml` pointer;
  `input` reclassified) is required to close the slice, not optional polish. *Alt —
  follow-on ADR:* rejected (fragments layering canon).
- **D6 — `syn` as a test-only dev-dependency.** *Alt:* regex (no dep) — rejected:
  cannot skip `#[cfg(test)]` cleanly, false-positives on doc-comment/string
  `crate::` (F-5). Centralized in `[workspace.dependencies]`; `[dev-dependencies]`
  only → zero shipped-binary weight.
- **D7 — Classify-first spike as PHASE-01, gating gate construction.** Gate value
  is unknown until the upward baseline is counted; classification is gameable.
  PHASE-01 authors the map + rationale + measures baselines; small baseline +
  meaningful engine core is the **go** condition. Converts the top risk into a cheap
  checkpoint.
- **D8 — `ACCEPTED_VIOLATIONS` + `TANGLE_BASELINE` as frozen, prune-forced
  baselines.** Debt enumerated/counted + frozen, not silently tolerated:
  `StaleAccepted`/`StaleEntry`/`TangleGrew` force tightening, forbid growth.
- **D9 — Top-level-module granularity, most-knowing-wins.** *Alt — per-file/
  submodule granularity (airtight, no umbrella laundering):* rejected for appetite —
  ~200 units, bigger map, heavier PHASE-01; the umbrellas are *already* slated for
  decomposition (SL-115/116). **Chosen:** top-level unit; tier = highest altitude of
  any non-test file (so an umbrella with a command file is command — honest, not
  laundered); the residual hole (a mostly-engine umbrella's pure-part upward edge)
  is named (§5.5) and mitigated by the decomposition slices + ratchet, not hidden.

## 8. Risks & Mitigations

- **R1 (deferred crate split orphaned)** — *Mit:* Follow-Ups name it, seeded by the
  map; ADR-001 keeps it as the named escalation.
- **R2 (laundering / dynamic reach)** — macro / re-export / wildcard could route an
  upward edge `syn` misses (ASM-2). *Mit:* no present breach (codex-verified: no
  production `macro_rules!` / wildcard `use crate::*`); review / `/inquisition`
  covers the residue; claim narrowed to "literal `crate::` path gate" (F-4).
- **R3 (classify-to-green)** — tiers chosen to pass, not by what a module knows.
  *Mit:* most-knowing-wins (D9) makes "relabel command to hide" honest *only* when a
  command file genuinely exists; per-module rationale authored before green; lying
  doc-comments fixed.
- **R4 (false green from a parser miss)** — *Mit:* pure `check` unit-tested with
  synthetic upward edge + intra-SCC new edge + unclassified + tangle-grew (VT-2);
  EX-1 hand-introduces a real upward edge and asserts `just gate` fails naming it.
- **R5 (ratchet rot / false confidence / overclaim)** — *Mit:* baselines cite
  follow-ups; `StaleAccepted` + monotonic-down force tightening; ADR-001 amendment
  states rule 2 unmet-and-tracked, not satisfied; the gate is named "literal
  `crate::` path gate," not "machine-enforced layering" (F-4).
- **R6 (gate value unknown)** — *Mit:* PHASE-01 go/no-go (D7); mostly-baseline →
  `/consult`.
- **R7 (umbrella-laundering residual, F-2)** — a mostly-engine umbrella with one
  command file masks its pure parts' upward edges (D9 limitation). *Mit:* named +
  disclosed; the decomposition slices (SL-115/116) shrink the umbrellas; the ratchet
  still pins intra-tier cycles. Accepted as the cost of top-level granularity.

## 9. Quality Engineering & Validation

- **VT-1 (real-graph gate, primary).** Asserts zero violations against the
  production graph under the authored baselines; red on any future upward edge,
  accepted-set growth, cyclic-edge increase, or unclassified module. Under `just gate`.
- **VT-2 (bite-proof, pure unit tests).** `check` over synthetic inputs: legal →
  `[]`; upward edge (not accepted) → `UpwardEdge`; accepted upward edge → `[]`;
  **new edge inside an existing same-tier SCC → `TangleGrew`** (the F-1 case);
  accepted-absent → `StaleAccepted`; src-only → `Unclassified`; map-only →
  `StaleEntry`.
- **VT-3 (cfg(test) exclusion).** Fixture: a `#[cfg(test)]`-scoped `crate::<command>`
  path → no edge.
- **VT-4 (doc-link exclusion).** Fixture: `crate::X` inside a `///` link → no edge
  (F-5).
- **EX-1** — hand-introduce a real upward edge in non-test code → `just gate` fails
  naming it; revert. Recorded, not committed.
- **EX-2** — `LAYER_MAP` keys == discovered modules (add/drop → red).
- **EX-3** — ADR-001 amended (rule 1 enforced; rule 2 ratcheted/tracked; table →
  definitions + `layers.toml` pointer; `input` reclassified; rejection overturned),
  via REV at reconcile (D5) — a co-requisite for close, not optional.
- **EN-1** — PHASE-01 go condition met. **EN-2** — `syn` dev-dep landed, gate green.
- **Phases.** PHASE-01 classify-first spike (author `layers.toml` + rationale;
  measure upward baseline + per-tier cyclic-edge count; **go/no-go**). PHASE-02 build
  the gate (shell + pure `check` + VT-1..4, EX-1..2; settle `layers.toml` home).
  PHASE-03 ADR-001 amendment via REV (co-requisite).
- **`just gate`** green (clippy zero-warning, fmt, `test-all`).

## 10. Review Notes

### Internal adversarial pass (2026-06-19) — integrated

Empirical probe of the production `crate::` graph. Findings, all integrated:
**F-1** `relation_graph` production upward reaches → command-tier (confirms the walk
must collect expr-position paths). **F-2** `state→install` baselined. **F-3**
`integrity` reaches 11 command shells → command-tier. **F-4** whole graph not
acyclic (intra-command SCC; engine core clean bar `conduct↔dtoml`/CHR-015) →
overturned the "no cycles (SCC size 1)" check into directional gate + tangle
ratchet; §2 corrected; ratchet-by-count added at the User's prompt. **F-5**
doc-comment intra-doc links are false edges → `syn` path-nodes dodge them. **F-6**
`input` is engine, not leaf. Process: gate value unproven until classification +
classification gameable → PHASE-01 go/no-go (D7) + per-module rationale (R3).

### External pass — codex / GPT-5.5 (2026-06-19, source-verified, read-only)

Five charges; verdict "fundamentally flawed." Dispositions:

- **C1 — tangle ratchet doesn't catch a new edge inside an existing SCC. ACCEPTED
  (real defect).** `Σ(SCC−1)` only moves on node growth/merge; `adr→backlog` inside
  the existing command SCC adds real coupling, metric unchanged. *Fix:* metric →
  per-tier **cyclic-edge count** (edges inside non-trivial same-tier SCCs); D3 / OQ-4
  / §5.1 / VT-2 updated.
- **C2 — top-level granularity launders via umbrella modules. ACCEPTED (deep).**
  `catalog/scan.rs` reaches command; `catalog/mod.rs:2` claims engine. *Fix:* D9
  most-knowing-wins (tier = highest altitude of any non-test file), fix the lying
  doc-comments, name the residual hole (R7) mitigated by SL-115/116. (Codex
  over-reached in treating the doc-comment self-claim as authoritative — it is not;
  but the umbrella-heterogeneity hole is real and now disclosed.)
- **C3 — `LAYER_MAP` as Rust `const` violates the storage rule. ACCEPTED.** *Fix:*
  map + baselines → authored `layers.toml`, test reads it (D2 reversed); ADR-001
  points at it.
- **C4 — overclaims "machine-enforced"; macro/re-export blind spots. ACCEPTED
  (wording).** Codex verified no production `macro_rules!` / wildcard imports → no
  present breach. *Fix:* claim narrowed to "literal `crate::` path gate" (F-4, R2, R5).
- **C5 — false doctrinal-alignment; ADR-001 currently rejects this test. ACCEPTED.**
  A future REV doesn't unsay today's accepted ADR. *Fix:* D5 reframed — the slice
  **overturns** the rejection; the amendment is a co-requisite (EX-3), not downstream.
- **Non-finding:** ASM-1 (CWD = package root) confirmed against the cargo manual —
  OQ-2 fully closed.

### Doctrinal alignment

ADR-001 — the slice **overturns** its rejection of a module-graph test and amends
it (co-requisite, EX-3/D5); rule 1 hard-gated (literal paths), rule 2 enforced as a
non-increasing cyclic-edge ratchet with the command tangle openly unmet-and-tracked,
rule 3 deferred. ADR-013 — amendment via REV at reconcile. Storage rule — map +
baselines in authored TOML (D2), not prose, not code. Pure/imperative split honored.
The F-4 governance fork (rule-2 reframing) was raised with and decided by the User
(the ADR-001 authority); no `/consult` outstanding.

### Next external pass — optional

The mechanism changed materially (metric, granularity, TOML canon, claim scope). A
short re-pass on the *revised* mechanism — does the cyclic-edge ratchet now genuinely
constrain? does most-knowing-wins close enough of the umbrella hole? — is available
before lock at the User's election.
