# SL-194 Design — Actionability interestingness findings

Text-first probe (RFC-007 workstream 2 "Legible", originates IMP-241). Pure
finding-functions over the priority engine, surfaced through a new `findings`
verb — one line per finding. No visualisation. Validates whether
findings-over-picture reads more useful than the flat `next`/`survey` list,
**before** any rendering-policy or visualisation follow-on is committed.

## Decisions (locked)

- **D1 — scope: full catalogue, sequenced.** All 9 findings ship, phased:
  today's-data core first (runnable), β-family second. The β-family (contested
  orderings) is the differentiated value a flat list conspicuously cannot show,
  so it is in-scope — not deferred — but sequenced after the core proves out.
- **D2 — surface: new `findings` verb** (not `survey --interesting`). Findings
  are *aggregate/relational* (a fork = hub + arm set; an inversion = an edge; a
  plateau = a segment); `survey`/`next` rows are *per-node* tuples. A dedicated
  verb fits the shape, keeps the probe cleanly removable, and gets its own
  render + `--json`. The RFC-007 "buried verb" concern is specific to `explain`
  (per-entity truth that belongs inline); an aggregate lens legitimately earns
  its own surface. Fold-into-`survey` (badges) is a follow-on the probe informs.
- **D3 — representation: enum `Finding` + accessor `impl`**, mirroring
  `ReasonKind` (the render-source-of-truth idiom next door in `view.rs`). Each
  variant carries its own structured payload; renderer only formats. Provenance
  findings **reuse** the existing `ReasonKind` provenance variants (DRY).
- **D4 — β semantics (canon: SL-172).** β ≡ `cfg.estimate.skew`. `est_cost =
  floor_eps(lower + skew·(upper − lower))`. β=0 → cost=lower (optimistic), β=1 →
  cost=upper (pessimistic). Perturbation = rebuild with swept skew over the
  **same scan**. Sweep = endpoints only `{0, 1}` (probe-grade "×3 builds"); a
  finer grid (precise flip-β) is a later refinement.
- **D5 — thresholds:** seeded named-const defaults (STD-001), calibrated from
  the first live-corpus output run. `[priority.findings]` config is a possible
  follow-up, not in scope.

## Substrate (settled by code)

`PriorityGraph` (`src/priority/graph.rs`) is the honest substrate — NOT the thin
`ActionabilityView` (web projection: total score + rank + blockers only). Fields
the detectors read:

- `graph` (cordage) — carries provenance: `provenance().cycles()` (SCCs) and,
  via `channels::evicted_seq_edges(g, k)`, the seq-overlay evictions.
- `attrs: BTreeMap<EntityKey, NodeAttr>` — `base_score`, `facets`.
- `score` / `leverage` / `optionality` maps — the display sort key + components.
- `dep_overlay` / `seq_overlay` handles.

**Provenance seam already exists** — no new plumbing. Evicted edges +
degraded/cycle SCCs are reachable today; the finding layer wraps them as
`Finding::Provenance(ReasonKind::{EvictedEdge,CycleDegraded})`.

## Architecture

Layering (ADR-001: leaf ← engine ← command). One new engine module, pure;
impurity in the existing shell; renderer + CLI at their existing layers. No
cycle.

```
src/priority/findings.rs   NEW  engine, PURE   — Finding enum + impl + detectors + thresholds
src/priority/order.rs      NEW  engine, PURE   — frontier_order + surviving_seq_predecessors (extracted)
src/priority/graph.rs      edit engine         — extract build_from_with_cfg (rebuild seam)
src/priority/surface.rs    edit impure shell   — fn findings(root); beta_endpoints; next reuses order.rs
src/priority/render.rs     edit engine         — findings_human + findings_json
src/priority/mod.rs        edit engine         — run_findings dispatch entry (peer of run_survey/run_explain)
src/commands/cli.rs        edit command        — `findings` verb wiring (match arm + members list)
```

`findings.rs` imports `graph` (PriorityGraph), `channels`
(`score`/`blocked_by`/`blocking`/`dep_cycles`/`evicted_seq_edges`/`class_of`),
`order` (the extracted ordering primitives), `config` (PriorityConfig).
`render.rs` imports `findings`. Both engine, acyclic.

**Ordering-primitive extraction (DRY).** `next`'s order is
`frontier_order(actionable, score, surviving_seq_predecessors(…))` — pure fns
currently *private in `surface.rs`*. The detectors need the same linear orders.
Extract both to a new pure `order.rs`; `surface::next` reuses them (no behaviour
change — byte-identical `next`), and detectors derive whatever order-basis they
need from an in-memory graph. No order re-implementation, purity preserved.

### The rebuild seam

`build_from` today loads config internally (`config::load(root)`), so β cannot
be injected. Extract, behaviour-preserving:

```rust
pub(crate) fn build_from_with_cfg(scanned: &[ScannedEntity], root: &Path, cfg: &PriorityConfig)
    -> anyhow::Result<PriorityGraph>
pub(crate) fn build_from(scanned, root)                       // delegates:
    = build_from_with_cfg(scanned, root, &config::load(root))
```

Existing callers unchanged ⇒ byte-identical output. **Behaviour-preservation
gate:** the `graph` + `surface` suites stay green *unmodified*, plus an
equivalence test (`build_from == build_from_with_cfg(…, load(root))`).

### Purity boundary — pre-built sweep, not an injected closure

`build_from` reads disk (`root` retained for per-item `dep_seq_for`). A
`rebuild: impl Fn(β) -> PriorityGraph` closure called *by the detectors* would
make the pure layer fallible + impure. Instead the **shell pre-builds the
sweep**; detectors only read pre-built graphs.

```rust
// impure shell (surface.rs) — owns all disk
pub(crate) fn findings(root: &Path) -> anyhow::Result<Vec<Finding>> {
    let scanned = relation_graph::scan_entities(root, &mut vec![], ScanMode::default())?; // ONE scan
    let cfg = config::load(root);
    let base = graph::build_from_with_cfg(&scanned, root, &cfg)?;
    let betas = beta_endpoints(&scanned, root, &cfg)?;   // Some iff a non-terminal estimate exists
    Ok(findings::detect(&base, &cfg, betas.as_ref()))
}

// pure engine (findings.rs) — total, no disk, no clock
pub(crate) fn detect(base: &PriorityGraph, cfg: &PriorityConfig, betas: Option<&BetaEndpoints>)
    -> Vec<Finding>;
pub(crate) struct BetaEndpoints { lo: PriorityGraph /* skew=0 */, hi: PriorityGraph /* skew=1 */ }
```

`beta_endpoints` builds `lo`/`hi` via `build_from_with_cfg` over the **same
`scanned`** with `skew` set to `BETA_LO`/`BETA_HI`; returns `None` when no
non-terminal estimate exists (no wasted builds; β-family findings simply do not
fire — the "starved until estimates set" behaviour, made explicit).

`detect` stays **graph-only** — it takes the base graph + `Option<&BetaEndpoints>`
(raw `lo`/`hi` graphs) and derives every order it needs internally via the pure
`order.rs` primitives. The shell owns disk (scan + the three builds); the
detectors own order derivation (pure, over in-memory graphs).

## The catalogue

```rust
pub(crate) enum Finding {
    Fork          { hub: String, arms: Vec<String> },
    Join          { node: String, prereqs: Vec<String> },
    GatingFanOut  { record: String, blocks: Vec<String> },
    ValueInversion{ blocker: String, blocked: String, gap: f64 },
    Displacement  { node: String, score_rank: usize, constrained_rank: usize, delta: usize },
    Plateau       { members: Vec<String>, span: f64 },
    OrderInstability { high: String, low: String },
    ArmResequencing  { hub: String, order_lo: Vec<String>, order_hi: Vec<String> },
    Provenance(ReasonKind),   // wraps EvictedEdge | CycleDegraded — reuses type + render
}
impl Finding {
    fn magnitude(&self) -> f64;            // ranks/caps output — catalogue self-prioritises
    fn kind_label(&self) -> &'static str;  // group header + json tag
}
```

Detectors — pure `fn(&PriorityGraph, …) -> Vec<Finding>`; edge access via
`channels` (cordage `Graph` has **no** `edge_count`/`node_count` — iterate
overlay edges per node):

| variant | reads (channels/order) | rule | magnitude | const |
|---|---|---|---|---|
| Fork | `blocking` (out), `class_of` | non-terminal out-deg ≥2 ∧ hub NOT gating-class | arm count | `FORK_MIN_ARMS=2` |
| Join | `blocked_by` (in) | in-deg ≥2 | prereq count | `JOIN_MIN_PREREQS=2` |
| GatingFanOut | `blocking` (out), `class_of` | hub gating-class (ADR-017) ∧ non-terminal out-deg ≥2 | block count | `GATING_MIN_BLOCKS=2` |
| ValueInversion | `score` + dep edges | `score(blocked) − score(blocker) > ε` | gap | `INVERSION_MIN_GAP` |
| Displacement | survey-order vs pure-score-order | `|score_pos − constrained_pos| ≥ ε` | delta | `DISPLACEMENT_MIN_DELTA` |
| Plateau | `score` over `next` order (`order.rs`) | maximal adjacent run within ε | run length | `PLATEAU_EPS` |
| OrderInstability | `BetaEndpoints` orders | **adjacent** pair in base order that inverts lo↔hi | positions moved | — |
| ArmResequencing | base Fork ∩ `BetaEndpoints` orders | fork whose arm order differs lo↔hi | arms moved | — |
| Provenance | `dep_cycles` + `evicted_seq_edges` | any eviction / SCC | nodes/edges | — |

**Direction correctness** (the trap): the dep overlay stores `needs` as
prereq→dependent (the B→A flip). *Fork* = `channels::blocking` (out — what
settling the hub unblocks); *join* = `channels::blocked_by` (in — its
prerequisites); *gating fan-out* = `blocking` (out) from a gating-class hub. Both
accessors match `survey`/`explain` eligibility/terminal handling — no re-derived
edge logic.

**Detector subtleties (adversarial-review fixes):**
- **`blocking` does NOT filter terminal successors** (unlike `blocked_by`). Fork
  and GatingFanOut therefore filter arms to **non-terminal** (`class_of ≠
  Terminal`) — settling a hub opens *unsettled* work, not already-done arms.
- **Fork ↔ GatingFanOut precedence:** a gating-class hub with fan-out is reported
  as **GatingFanOut only** (Fork excludes gating-class hubs) — no double report.
- **Order basis differs per finding, all derived via `order.rs`:**
  *Displacement* = position in the **survey order** (actionability→score, the
  constraint-bearing order) vs position in the **pure-score order** (score desc,
  constraint-free) — the delta is "constraints doing real work" (a high-score
  blocked item sinking below low-score actionable ones). *Plateau* = adjacent
  near-ties in the **next** frontier order. *OrderInstability / ArmResequencing*
  = the **frontier order** recomputed at β=0 vs β=1.
- **OrderInstability is bounded to adjacent transpositions** — pairs *adjacent*
  in the base order whose relative order inverts between lo and hi. O(N), not the
  O(N²) all-pairs flood; captures "the order right *here* is contested".

**Overlay split:** fork / join / gating / inversion read the **dep** overlay
(`needs`, hard, Reject — never evicted). Provenance findings read the **seq**
overlay evictions + cycle SCCs. The soft-edge eviction noise lives only in the
provenance finding, where it *is* the signal.

**Constants:** one named `const` block in `findings.rs` (STD-001, no magic
literals): the six thresholds + `BETA_LO=0.0` / `BETA_HI=1.0`. `PLATEAU_EPS`,
`INVERSION_MIN_GAP`, `DISPLACEMENT_MIN_DELTA` are the judgment knobs — seeded
defaults, calibrated from the first live output.

**Output ordering:** `detect` returns all findings sorted `(kind_label,
magnitude desc)`; renderer groups by kind. No hard cap (probe wants the field).

## Surface & render

`doctrine findings [--json]`. Human render groups by `kind_label()`, one line
per finding; json emits `[{kind, …payload, magnitude}]`. Provenance findings
reuse the existing `explain_human` fragment for `EvictedEdge`/`CycleDegraded`
(extract the fragment; do not re-format). Source-of-truth stays in the `Finding`
types; the renderer only formats.

```
forks
  QUE-003  settles → {IMP-054, IMP-071, ISS-028}   (3 arms)
value inversions
  IMP-071 (2.1) gates IMP-054 (18.4)               Δ16.3
plateaus
  {IMP-001 … IMP-009}  score≈0.42                  (9, span 0.01)
order instability
  IMP-054 ↔ IMP-071   (flips β0↔β1)
provenance
  evicted  SL-x → SL-y  (cycle-break)
```

## Phasing

Two phases, each ends runnable + green (probe wants an early verdict on the core
before investing in β).

- **PHASE-01 — runnable core probe.** `build_from_with_cfg` extraction +
  `order.rs` extraction (both behaviour-preserving); `findings.rs` (enum + `impl`
  + thresholds); today's-data detectors (Fork, Join, GatingFanOut,
  ValueInversion, Displacement, Plateau, Provenance); `surface::findings`;
  `run_findings` + `findings` verb + `findings_human`/`findings_json`. End:
  `doctrine findings` runs against the live corpus → judge the core.
- **PHASE-02 — β-family.** `beta_endpoints` sweep (skew 0/1 over the one scan);
  `OrderInstability` + `ArmResequencing`; render extension. End:
  contested-ordering findings live.

## Verification alignment

- **Detectors** — VT, positive + negative fixture each. Detector *functions* are
  pure/total; their *tests* may seed a temp corpus + `build` (the established
  `graph`/`surface` test idiom) — "no disk" binds the detector, not its harness.
- **Rebuild seam** — VT behaviour-preservation: existing `graph`/`surface`
  suites green *unchanged*; equivalence `build_from == build_from_with_cfg(…,
  load(root))`.
- **β** — VT: interval that flips order lo↔hi; `beta_endpoints` returns `None`
  on an estimate-free corpus (no wasted builds, β-family silent).
- **Surface** — VT: verb golden (human) + json shape.
- **Probe verdict — VA/VH (closure gate):** run against the live corpus; does
  the output read more useful than the flat list? Design records the verdict +
  whether the rendering follow-on (arc-strip, fold-into-`survey`) is warranted.

## Constraints (canon)

ADR-001 (module layering — finding module engine-layer, pure), ADR-015 (score
components), ADR-017 (gating status class — GatingFanOut), STD-001 (named
constants), SL-172 (β-skew cost model), render-source-of-truth discipline
(`view.rs`), pure/imperative split (no disk/clock/rng in `findings.rs`).

## Risks / open items

- **R1 — gating-fan-out & β-family may ship starved.** ADR-017 gating records
  and authored estimates are thin today (RFC-007 workstream 3 "Populate" not
  done). Accepted: these findings activate as data grows; the probe still judges
  the mechanism on the findings that do fire.
- **R2 — ISS-003** (cordage `explain()` foreign-node bug, RFC-007) is adjacent.
  Findings do not go through `explain`, but if the provenance fragment reuse
  touches the shared path, watch for it. Out of scope to fix here.
- **R3 — ε defaults are guesses** until the first live run. The probe itself is
  the calibration instrument (D5).

## Follow-ups (out of scope)

- Rendering-policy follow-on (arc-strip linear view, include-by-finding graph
  inclusion, semantic synthesis, web) — mint if the probe validates.
- Fold `explain` into `next`/`survey` (`--why`), coefficient what-if trace —
  sibling RFC-007 (2) concerns.
- `[priority.findings]` threshold config; precise flip-β via a finer sweep grid.
