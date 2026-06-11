# Design SL-039: backlog dependency ordering — item edges + cordage adapter

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-036, ADR-004, RSK-002, SPEC-001, IMP-021); doc-local refs bare — D1 (§7),
     OQ-A, I2 (§6), VT-3 (§8). -->

## 1. Design Problem

Wire **cordage's first consumer**. cordage (SL-036, SPEC-001) is a shipped,
zero-dep, pure-leaf graph-evaluation engine; no crate depends on it yet. SL-039
adds the `doctrine → cordage` path dependency (the ADR-001 layering edge) and the
*consumer half* of cordage's D1 ownership split: **core owns the mechanism, the
consumer owns the vocabulary.**

Concretely: give backlog items two authored item→item edge types and render the
backlog in a composed order — **deps ≻ manual-seq ≻ exposure ≻ creation** —
produced by a multi-layer cordage `OrderSpec` plus its native `NodeId` fallback.
The adapter is a thin vocabulary layer (id mapping + named overlay handles +
wrong-wiring-proof newtypes); cordage does the ordering. This earns the engine its
keep through its headline feature (multi-layer composition) at real-but-small
scale, and harvests the one budgeted R-C interface rev SL-036 reserved.

## 2. Current State

- **Backlog model** (`src/backlog.rs`): five `ItemKind`s over one kind-blind
  `entity` engine; each its own tree + counter. `Relationships { slices, specs,
  drift }` is outbound-only (ADR-004), `#[serde(default)]`, seeded `[]` in both
  templates. A risk-only `[facet] { likelihood, impact, origin, controls }`
  carries the two assessable axes as the `"" -> None` seam. `backlog list` sorts
  `(kind.ordinal, id)`; terminal items (`resolved`/`closed`) hide via
  `Status::is_terminal`. **No item→item edge exists.**
- **cordage** (`crates/cordage`): public surface `GraphBuilder` (`node()→NodeId`,
  `overlay(cfg)→OverlayId`, `edge(ov,src,dst,attrs)`, `order_spec`, `build()`),
  `Graph::{order_key, ordered, provenance, reachable, evaluate, explain}`. A
  workspace member with **no dependents** (`cargo tree -p cordage` shows it alone).
- **Ordering signal**: none beyond id. Only risks carry structured facet data
  (`likelihood`/`impact`); the other four kinds carry none.

## 3. Forces & Constraints

- **D1 ownership (SPEC-001).** The core carries no doctrine vocabulary; the
  adapter owns `ItemId↔NodeId`, the overlay meanings, and the order recipe. cordage
  never learns the word "dependency".
- **Outbound-only relations (ADR-004).** Each item→item edge is authored on **one**
  canonical side; the reverse is derived, never stored. No new ADR — both edges are
  instances of ADR-004.
- **Pure leaf untouched (ADR-001, leaf invariant).** No edit to `crates/cordage/**`.
  Any API friction is *recorded* (the R-C harvest), not patched here.
- **ordering is pure before/after.** `EdgeAttrs { rank, age }` is **eviction
  durability only** — it never enters `OrderKey`. Order is composed topology; the
  numeric `Level` is materialised longest-path depth, not a rank.
- **`ordered()` is longest-path, not lexicographic (SL-036 F11).** `OrderKey =
  (longest-path level in the merged DAG U, NodeId)`. Layer precedence governs only
  **eviction** during composition — a lower-layer edge that closes a cycle against a
  higher layer is dropped — **not** a per-layer level tuple. This is load-bearing
  for the whole design (see §5.1, I1).
- **Determinism (REQ-077, inherited).** Same inputs → byte-identical order. Node
  allocation order is fixed `(exposure desc, created, canonical-id)`; cordage is
  deterministic.
- **Small corpus.** Tens of items, sparse edges — far below every cliff
  (RSK-002/003/004). `explain()` (RSK-002, exponential) is **not called**.
- **pure/imperative split.** The adapter is pure over its projected inputs; the
  `order` verb is the thin impure shell (read items → adapter → print).

## 4. Guiding Principles

- **Ordering lives in cordage, not the adapter.** The adapter builds two overlays +
  one `OrderSpec`, allocates nodes in a fixed order, then reads `ordered()`. It
  performs **no sort**. This is the D9/D10 "dep-topology → seq → fallback" recipe
  SL-036 reserved, made real.
- **One uniform overlay invariant.** Every cordage edge `s→t` means "**s ordered
  before t**"; **every** `OrderLayer` is `Along`. The single inversion is the
  authored dependency direction (below). One rule, no per-overlay direction
  reasoning.
- **The R-C kill is encapsulation, not ceremony.** `NodeId`/`OverlayId` never leave
  the adapter; `ItemId` is the only token a caller touches. Wrong-wiring (a node
  from another graph, the wrong overlay handle) is *inexpressible* because those
  tokens have no public surface. Direction can't be misordered because edges are
  *ingested from authored maps*, not hand-wired.
- **Hardest signal wins, softest yields — and yielding is surfaced.** The four
  order tiers are a monotone hierarchy; cordage's layer-precedence eviction enforces
  the hard-vs-soft conflicts, and every dropped soft edge is reported (`Evict` +
  info), never silent.
- **Name the signal honestly.** The derived per-risk value is **exposure**
  (`likelihood`×`impact`) — a sanity-ordering input, **not priority**. A real
  priority model is out of scope; it would one day *consume* exposure as one input
  (IMP-021 captures the related list-surface affordance).

## 5. Proposed Design

### 5.1 System Model — the four order tiers

```text
  src/backlog.rs  (vocabulary)        →  projects items to OrderInput, derives exposure
        │  Vec<OrderInput>
  src/backlog_order.rs  (adapter)     →  ItemId↔NodeId bimap, 2 overlays, 1 OrderSpec, fixed alloc order
        │  GraphBuilder → build() → Graph
  crates/cordage  (mechanism)         →  composes U, ordered(), provenance
```

The composed order, highest-precedence tier first:

| Tier | Carried by | Conflict policy | Source |
|---|---|---|---|
| 0 deps | `depends_on` overlay (layer 0) | **Reject** + hard error | authored, hard prerequisite |
| 1 manual-seq | `before` overlay (layer 1) | **Evict** + info | authored, soft preference |
| 2 exposure | `NodeId` fallback, part 1 | — | derived `likelihood`×`impact` |
| 3 creation | `NodeId` fallback, part 2 | — | allocation order |

`OrderSpec = [Along(depends_on), Along(before)]`. The `NodeId` fallback (tiers 2–3)
is realised by **allocating nodes in `(exposure desc, created, canonical-id)`
order**, so the smallest `NodeId` is the highest-exposure / earliest item.

**How the order is actually produced (and why this shape).** `ordered()` returns
nodes by `(longest-path level in U, NodeId)`. Tiers 0–1 are *overlay edges* → they
set the **levels** (the genuine before/after structure); a `before` edge that
contradicts a `depends_on` edge is evicted (`UnionCycleVsLayer`, tier-0 authority),
so deps ≻ manual-seq holds. Tiers 2–3 are the **fallback** → they break ties only
*among items at the same composed level* (i.e. unordered by deps/seq). **Exposure is
deliberately NOT an overlay** — an exposure overlay would emit cross-level edges and
drag dependency-incomparable items across levels (a high-exposure but deeply-blocked
item would bury an independent, actionable one — see §10 finding A1). As a fallback
it acts only where deps/seq are silent, which is exactly "order otherwise-unordered
items by exposure, then creation."

**The `≻` is shorthand, not a lexicographic sort (don't read it as one).** deps and
`before` *jointly* determine the longest-path levels; layer precedence governs only
**eviction** (I1), never a per-tier level decomposition. Exposure and creation break
ties **within** a level. So `before` constrains only the pairs it touches and the
fallback orders everything else: with `A.before=[B]` and an unrelated higher-exposure
`C`, the order is `C, A, B` (C and A share level 0; B is level 1) — *not* a global
"manual-seq outranks exposure" ranking. And there is no `deps`-then-`before`
lexicographic split at all: when `A.before=[B]` and `B.depends_on=[C]`, B's level is
jointly caused by both layers. The honest contract is **two constraint layers + a
within-level fallback; precedence decides evictions** — the four-tier table reads
that, nothing stronger.

**The uniform invariant + the one flip.** Every overlay edge `s→t` ⇒ s before t,
all layers `Along`:

- `depends_on`: authored `A.depends_on=[B]` (dependent→prerequisite). Stored as
  cordage edge **`B→A`** (the flip) so "src before dst" holds: B (prereq) before A.
- `before`: authored `X.before=[Y]` (X precedes Y). Cordage edge **`X→Y`**.

### 5.2 Data model — two outbound-only item→item edges

`Relationships` (and both templates) gain two keys, seeded `[]`:

```toml
[relationships]
slices = []
specs  = []
drift  = []
depends_on = []   # canonical refs this item needs done first (hard)
before     = []   # canonical refs this item should precede (soft)
```

```rust
struct Relationships {
    // …existing slices/specs/drift…
    #[serde(default)] depends_on: Vec<String>,  // on the dependent
    #[serde(default)] before:     Vec<String>,  // on the earlier item
}
```

Reciprocity (what depends on me / what precedes me) is **derived**, never stored
(ADR-004). `show` renders the two outbound axes alongside the existing three.

### 5.3 Exposure — derived only, not priority

No new authored field. A pure projection over the item's own state:

```rust
fn exposure(item: &BacklogItem) -> u8 {  // 0 = baseline; assessed risk 1..=16
    match &item.facet {
        Some(f) => match (level_val(f.likelihood), level_val(f.impact)) {
            (Some(l), Some(i)) => l * i,   // low/med/high/critical → 1..4, product 1..16
            _ => 0,                         // unassessed → baseline
        },
        None => 0,                          // non-risk → baseline
    }
}
fn level_val(l: Option<RiskLevel>) -> Option<u8> { /* Low=1 … Critical=4 */ }
```

Exposure feeds only the **node allocation order** (§5.1 fallback) — it is never an
overlay and never an `OrderKey` numeric field; cordage's `NodeId` tiebreak carries
it. Equal-exposure items fall through to creation order. Vocabulary is deliberate:
this is risk exposure, a future-priority *input*, not priority (§4).

### 5.4 The adapter (`src/backlog_order.rs`) — the R-C kill

```rust
/// The only token a caller handles. cordage NodeId/OverlayId never surface.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ItemId { kind: ItemKind, id: u32 }   // renders "RSK-002"

/// The vocabulary projection backlog.rs hands to the adapter (BacklogItem stays
/// private). One row per non-terminal item.
pub(crate) struct OrderInput {
    item: ItemId,
    created: String,          // for the (exposure, created, id) allocation order
    exposure: u8,
    depends_on: Vec<ItemId>,  // resolved from the authored refs
    before: Vec<ItemId>,
}

/// Built once; owns the graph, the bimap, and the two minted overlay handles.
/// The handles are **named fields, not a positional `[OverlayId;2]`** — positional
/// storage invites a transposition bug (the wrong handle filtered against
/// provenance), which the token-hiding does NOT prevent (§10 E4).
pub(crate) struct BacklogOrder {
    /* graph: Graph, by_item: BTreeMap<ItemId,NodeId>, by_node: Vec<(NodeId,ItemId)>,
       depends_on_overlay: OverlayId, before_overlay: OverlayId */
}

impl BacklogOrder {
    /// Build the two overlays + OrderSpec from the projected inputs. Nodes are
    /// allocated in (exposure desc, created, canonical-id) order so the NodeId
    /// fallback = exposure-then-creation; each NodeId is captured from
    /// builder.node() (never constructed — mem.pattern.cordage.opaque-ids-capture-
    /// from-builder). Edges whose endpoint is not in the node set (terminal /
    /// absent) are dropped and recorded.
    pub(crate) fn build(inputs: &[OrderInput]) -> Self;

    /// The composed order: ordered() → map each NodeId back through the bimap.
    pub(crate) fn ordered(&self) -> Vec<ItemId>;

    /// Dependency cycles (the dep overlay's CycleDiagnostics), members as ItemIds.
    /// Non-empty ⇒ the order render is a hard error.
    pub(crate) fn dep_cycles(&self) -> Vec<Vec<ItemId>>;

    /// Surfaced soft-edge drops (the "overrides" info): `before` edges evicted
    /// intra-overlay (Evict cycle) or cross-layer (UnionCycleVsLayer vs a dep),
    /// plus dangling edges dropped at build. Each as (from, to, reason) in ItemIds.
    pub(crate) fn overrides(&self) -> Vec<Override>;
}
```

cordage build inputs: two `OverlayConfig`s — `depends_on` `(Reject, Unbounded)`,
`before` `(Evict, Unbounded)`. Every edge carries `EdgeAttrs::new(0, 0)` —
durability is unused (all edges equal); the `before` overlay's `Evict` then selects
the dropped edge deterministically by the `(rank, age, src, dst)` key, i.e. by
`(src, dst)` `NodeId`s when `rank=age=0`. `build()` is infallible for our
well-formed input (ids from the builder, each overlay once in the `OrderSpec`,
`Along` not `None`); a `BuildError` is an adapter bug, propagated as an internal
error.

### 5.5 CLI surface

- `backlog depends-on <ITEM> <PREREQ>…` — append prereqs to `ITEM.depends_on`,
  edit-in-place (`toml_edit`, the `set_backlog_status` precedent). Validates each
  ref exists, then **builds the dependency graph including the new edge and refuses
  on a cycle** (clear error naming the members; nothing written). Immediate author
  feedback; reuses the adapter's `dep_cycles`.
- `backlog before <ITEM> <SUCCESSOR>…` — append to `ITEM.before`. Validates refs
  exist; **never** rejects a cycle (soft, `Evict` — surfaced at `order`).
- `backlog order [filters]` — the new ordered view. Reads non-terminal items,
  projects, builds the adapter, prints the composed order (reusing the `list`
  column model) followed by an **overrides** block (`overrides()`). A dependency
  cycle is a **hard error** (stderr, non-zero exit) naming the members — no
  misleading order printed. `show` gains the two outbound axes; `list` is untouched
  (its `(kind.ordinal, id)` goldens unchanged). Inbound ("what depends on me") is
  not shown — reverse is derived elsewhere (ADR-004).

### 5.6 Membership & dangling edges

Node set = **non-terminal** items across **all five kinds** (cross-kind deps
allowed). A terminal item cannot participate in a live ordering (it is hidden from
the view), so an authored edge whose endpoint is:

- **terminal** (`resolved`/`closed`) — dropped and recorded in `overrides()` **with
  the endpoint's status AND `resolution` named** ("dep IMP-007 dropped — closed/
  wont-do"). The render does **not** silently claim the prerequisite was *satisfied*:
  a terminal `resolution` may be satisfied (`done`/`fixed`/`mitigated`) **or
  abandoned** (`wont-do`/`obsolete`/`expired`/`duplicate`/`accepted` — backlog.rs
  `Resolution`). Dropping a hard `depends_on` whose prereq was *abandoned* floats the
  dependent unblocked, so the drop must be **loud**, never silent — the author judges
  staleness from the named resolution (§10 E1; whether to go further and treat
  abandoned-terminal deps distinctly is **OQ-D**).
- **absent** (no such id) — a stale ref; the set verbs reject it at author time, so
  at `order` time it is dropped + recorded defensively.

Terminal elision does **not** synthesise transitive edges: with `A.before=[B]` and
`B.before=[C]` and `B` terminal, `A` and `C` are left unordered (both endpoints of
the surviving-nothing chain) — recorded, and for the soft `before` edge acceptable;
re-author the sequence if it must outlive the middle (§10 E3). For `depends_on`, a
satisfied-terminal middle correctly releases the transitive constraint (the blocker
is done); an abandoned-terminal middle is the OQ-D case.

This keeps the graph total over the live node set; cordage never sees a foreign id.

## 6. Invariants, Assumptions & Edge Cases

- **I1 — composition, not sorting; longest-path, not lexicographic.** `ordered()`
  is the sole order authority; the adapter calls no comparison to *rank output*.
  Exposure/creation comparisons exist only to *allocate nodes*; deps/seq
  comparisons only to *emit edges*. Order = `(longest-path level in U, NodeId)`;
  layer precedence is eviction authority only (SL-036 F11). An implementer must not
  compute per-layer level tuples.
- **I2 — the hierarchy holds.** A `before` edge contradicting a `depends_on` edge is
  evicted (`UnionCycleVsLayer`); deps therefore set levels unopposed. Exposure and
  creation act only within an equal composed level (the fallback), so manual-seq ≻
  exposure (a level beats the fallback) and exposure ≻ creation (the fallback key
  order). REQ-076 "degrade, never falsify" inherited — every eviction is surfaced.
- **I3 — determinism.** Allocation order `(exposure desc, created, canonical-id)` is
  total; cordage is deterministic ⇒ byte-identical order + overrides across runs,
  invariant under input permutation.
- **I4 — leaf untouched.** `cargo tree -p cordage` shows cordage alone; no
  `crates/cordage/**` diff.
- **Edge cases:** empty backlog / no edges → pure `(exposure, creation)` order; a
  dep cycle → hard error (set-time refusal is the normal guard; `order`-time is the
  backstop); a `before` cycle → min-`(src,dst)` edge evicted + info, order still
  printed; deps and `before` agreeing → no conflict; an edge to a terminal/absent
  item → dropped + recorded (§5.6); a risk with only one facet axis assessed →
  baseline exposure; two items same `(exposure, created)` → tie broken by canonical
  id.
- **Assumption A1.** `created` is validated as an **opaque `String`**, not a typed
  date (backlog.rs `RawBacklogToml.created`), so the allocation's middle key is
  **lexicographic on that string**. This is total and deterministic ⇒ **I3 holds
  unconditionally** (string order + the canonical-id final tiebreak never tie). It is
  *chronological* only insofar as `created` is well-formed `YYYY-MM-DD` (the scaffold
  seeds exactly that). A hand-edited malformed date (`2026-6-9` sorts after
  `2026-06-10`) perturbs only **tier 3** (creation, the lowest tiebreak, reached only
  among equal-exposure items) — never determinism, never deps/seq/exposure. Parsing
  `created` to a typed date is a backlog-model change, out of this slice's scope
  (§10 E5).

## 7. Decisions, Rationale & Alternatives

- **D1 — composition in cordage, no adapter sort.** *Alt rejected:* adapter sorts
  the output — puts ordering logic in the consumer and re-derives what `OrderSpec`
  composition + the `NodeId` fallback already give. The multi-layer recipe is
  precisely why cordage exists.
- **D2 — four-tier hierarchy deps ≻ manual-seq ≻ exposure ≻ creation.** Each tier is
  harder / more intentional than the one below. deps/manual-seq are *overlay edges*
  (levels); exposure/creation are the *fallback* (within-level tiebreak).
- **D3 — two edge types, one mechanism.** `depends_on` and `before` are the *same*
  before/after overlay primitive (DD1), differing only in layer position and cycle
  policy. No second mechanism.
- **D4 — uniform "src before dst", all `Along`; flip the authored dep direction.**
  *Alt rejected:* keep `depends_on` as `A→B` and order it `Against` — forces
  per-overlay direction reasoning. One flip at ingest is cheaper to hold.
- **D5 — exposure is a fallback tiebreak, NOT an overlay.** An exposure overlay
  emits `P→Q` for any exposure gap, i.e. *cross-level* constraints; merged into U's
  longest-path it drags dependency-incomparable items across levels — a
  high-exposure but deeply-blocked item buries an independent actionable one (§10
  A1, worked example). The fallback acts only within an equal composed level, which
  is the correct "order otherwise-unordered items" semantics. Simpler too (2
  overlays, no O(n²) edges). *Alt rejected:* the overlay (the original design).
- **D6 — exposure is derived-only, no authored field, and is not priority.** Writing
  less code; honest that only risks carry a structured signal today. The name is
  exposure (`likelihood`×`impact`), an input to a future priority model — not
  priority. *Alt rejected:* an authored `priority`/`exposure` field on all five
  templates — more surface for a mostly-baseline signal; reopen if a real cross-kind
  signal emerges. (IMP-021 captures the related `list` filter/sort affordance.)
- **D7 — `before` is `Evict` + info, `depends_on` is `Reject` + error.** A
  self-contradictory *soft* preference should resolve deterministically and be
  reported, not block; a contradictory *hard* prerequisite is an authoring error
  worth refusing. *Alt rejected:* `Reject` on `before` — blocks the backlog on a
  mere preference conflict.
- **D8 — new `backlog order` verb.** Keeps `list`'s sort contract and black-box
  goldens stable (SRP). *Alt rejected:* `list --by-deps` — churns the `list` sort
  contract for an orthogonal view.
- **D9 — no new ADR.** Both edges are ADR-004 outbound-only relations; the cordage
  mapping is slice-local mechanism. Revisit only if more item→item edge categories
  emerge.

## 8. Quality Engineering & Validation

Black-box where possible; the adapter is pure over `OrderInput` and unit-testable
without disk. TDD red/green/**refactor** per phase.

| Behaviour | Evidence |
|---|---|
| **VT-1 schema** | both templates seed `depends_on=[]`/`before=[]`; a virgin item round-trips through `validate`; `show` renders both axes when populated. |
| **VT-2 dep order** | `A.depends_on=[B]` ⇒ B before A in `ordered()` (the flip + `Along`). |
| **VT-3 hierarchy** | a `depends_on` overriding a contradicting `before` (the `before` edge appears in `overrides()`); `before` ordering two otherwise-unordered items; equal-level pair falls to `(exposure, creation)`. |
| **VT-4 exposure** | a high-exposure risk precedes a baseline item **when at the same composed level**; an independent baseline item is **not** buried behind a deeply-blocked high-exposure item (the §10 A1 regression); unassessed risk + non-risk both = baseline. |
| **VT-5 dep cycle** | `A→B→A` ⇒ `dep_cycles()` names {A,B}; `order` exits non-zero; the set verb refuses the closing edge. |
| **VT-6 soft cycle** | `before` `X↔Y` ⇒ min-`(src,dst)` edge evicted, order still printed, eviction in `overrides()`. |
| **VT-7 membership** | terminal/absent endpoint dropped + recorded; node set is non-terminal, cross-kind. |
| **VT-8 determinism** | build twice (incl. input permutation) → byte-identical order + overrides; allocation order `(exposure, created, id)`. |
| **VT-9 leaf invariant** | `cargo tree -p cordage` shows cordage alone; no `crates/cordage/**` diff. |
| **VT-10 R-C** | a public-surface audit (test/doc) asserts no `NodeId`/`OverlayId` appears in any adapter `pub(crate)` signature — so **adapter callers cannot pass a raw cordage id** (the bounded claim; cordage's own tokens stay `pub`, §10 E4). Internal handle-transposition is contained separately by the named overlay fields (§5.4), not by token absence. |

`just check` zero-warnings after every file; the cordage-side lint bans (BTree not
Hash, `.get` not index, `try_from` not `as`, `#[expect(reason=…)]`) apply to adapter
code. `cargo clippy -p doctrine` / `-p cordage` — never `--all-targets`.

## 9. Open Questions & Unknowns

- **OQ-A (soft, impl).** Where the `OrderInput` projection + `exposure()` live
  exactly — in `backlog.rs` (near the facet) handing a projection to
  `backlog_order.rs`, vs accessors on `BacklogItem`. Lean: projection in
  `backlog.rs` (keeps `BacklogItem` private, derivation near its data). Settled in
  `/plan`/impl, not load-bearing.
- **OQ-B (harvest, expected).** The one budgeted R-C interface rev — recorded only
  if real use surfaces a concrete cordage API bend (objective 5). Captured as a
  durable note + `/record-memory`; **not** patched in this slice.
- **OQ-C (cosmetic).** Verb naming (`depends-on`/`before`) — final clap shapes
  settled in `/plan`.
- **OQ-D (semantics, USER — gates lock).** A hard `depends_on` whose prereq is a
  terminal item with an **abandoned** resolution (`wont-do`/`obsolete`/`expired`/
  `duplicate`/`accepted`). Two options: **(D-min)** the §5.6 honest-record form —
  drop the edge, surface it loudly in `overrides()` with the resolution named, do not
  adjudicate; the author judges staleness. Lightest, stays inside the settled
  "dropped+recorded" rule. **(D-split)** add a satisfied-vs-abandoned `Resolution`
  taxonomy and treat an abandoned-terminal hard dep distinctly (e.g. a stronger
  "stale dependency" diagnostic, or a hard error). More correct for the hard
  contract, but adds a resolution-kind classification this slice didn't plan. Lean:
  **D-min** for a first-consumer small-corpus tool, taxonomy as a follow-up IMP if it
  bites. Design currently authored to D-min; D-split is additive. (§10 E1.)

## 10. Review Notes

### Internal adversarial pass (round 1) — 4 findings, all integrated

- **A1 (blocker, ordering model) — FIXED.** The first draft modelled exposure as a
  third overlay. cordage's `ordered()` is longest-path over the *merged* U (not
  lexicographic-by-layer, SL-036 F11), so an exposure overlay's cross-level edges
  drag dependency-incomparable items across levels: with `A` independent/baseline
  and `B` exposure-16 behind a 3-deep prereq chain, the surviving `B→A` exposure
  edge pushes `A` to `level 4` — an actionable independent item rendered dead last.
  Fix: exposure is the `NodeId` fallback (allocation order), not an overlay, so it
  acts only within an equal composed level; `OrderSpec` drops to two layers. (§5.1,
  §5.3, §5.4, D1/D5, I1, VT-4.)
- **A2 (significant, F11 misread risk) — FIXED.** §5.1's "tier" table + "≻" framing
  invited the exact F11 lexicographic-level bug (sort by dep-level, then seq-level…).
  Added the explicit "longest-path, not lexicographic; precedence = eviction
  authority" clarification and I1. (§3, §5.1, I1.)
- **A3 (moderate, naming) — FIXED (user).** "Priority" overclaimed —
  `likelihood`×`impact` is risk *exposure*, an input to a future priority model, not
  priority. Renamed throughout; the `list` filter/sort affordance split out to
  IMP-021. (§1, §4, §5.3, D6.)
- **A4 (minor, contracts) — FIXED.** `before` `Evict` tie-break determinism pinned
  to `(src, dst)` under `rank=age=0`; VT-10's "won't compile" softened to a
  public-surface token-absence audit (no `trybuild` overclaim). (§5.4, VT-10.)

### External adversarial pass (round 2) — codex MCP (GPT-5.5), 2026-06-11

Hostile pass over the order-correctness proof, determinism, dangling/terminal
handling, the R-C claim, and assumed-but-absent cordage API. Five findings; every
source citation independently re-verified against `src/backlog.rs` /
`crates/cordage/src/{lib,resolve}.rs` before disposition. No finding re-litigated a
user-settled call.

- **E1 (blocker, terminal semantics) — FIXED (framing) + OQ-D (taxonomy).** §5.6
  claimed a terminal endpoint means the prerequisite is *satisfied*. False:
  `Status::is_terminal` = `Resolved|Closed` (backlog.rs:219), and a terminal
  `Resolution` may be **abandoned** — `wont-do`/`obsolete`/`expired`/`duplicate`/
  `accepted` (backlog.rs:244). Dropping a hard `depends_on` on a `wont-do` prereq
  floats the dependent unblocked, falsifying the hard contract. Fixed §5.6 to drop
  the false "satisfied" claim and **record the dropped endpoint's status + resolution
  loudly** (never silent). The deeper choice — honest-record (D-min) vs a
  satisfied/abandoned taxonomy (D-split) — is **OQ-D**, a user decision that gates
  lock. Design authored to D-min; D-split is additive. (§5.6, §9 OQ-D.)
- **E2 (significant→clarification, contract overclaim) — FIXED.** The
  `deps ≻ manual-seq ≻ exposure ≻ creation` headline reads as a global lexicographic
  precedence the longest-path key cannot represent (`ordered()` = `(level in merged
  U, NodeId)`, lib.rs:337/869, resolve.rs:507). Counterexample integrated: `A.before=
  [B]` + unrelated higher-exposure `C` ⇒ `C, A, B`, i.e. `before` constrains only the
  pairs it touches; and `A.before=[B]` + `B.depends_on=[C]` makes B's level *jointly*
  caused — no deps-then-seq decomposition. The mechanism was already correct (A1/I1);
  this hardens the *framing* to "two constraint layers + within-level fallback;
  precedence decides evictions." (§5.1.)
- **E3 (moderate, transitive elision) — FIXED.** Terminal middle severs authored
  sequence: `A.before=[B]`, `B.before=[C]`, `B` terminal ⇒ A,C unordered. Documented
  as deliberate (no synthetic transitive edge); acceptable for soft `before`,
  re-author if it must survive; for `depends_on` a *satisfied* middle correctly
  releases the constraint, an *abandoned* middle is the OQ-D case. (§5.6.)
- **E4 (significant, R-C claim) — FIXED.** VT-10's "wrong-wiring inexpressible by
  token-absence" overclaimed: `NodeId`/`OverlayId` + builder/`OrderLayer::new` stay
  `pub` (lib.rs:25,33,316,594); hiding tokens from *callers* stops one misuse class
  but not the adapter transposing its own two handles, nor a module bypassing the
  adapter. Weakened VT-10 to the bounded "callers cannot pass a raw cordage id", and
  replaced the sketch's positional `[OverlayId;2]` with **named fields**
  (`depends_on_overlay`/`before_overlay`) to contain internal transposition. (§5.4,
  VT-10.)
- **E5 (moderate, determinism semantics) — FIXED.** `created` is an opaque `String`
  (backlog.rs:315), so the allocation's middle key is **lexicographic, not
  chronological** — a malformed hand-edited date reorders it. Determinism (I3) was
  never at risk (string order is total + id final tiebreak); only the *creation-order
  semantics* of the lowest tier are best-effort. Assumption A1 rewritten to say so;
  typed-date parsing noted as out-of-scope. (§6 A1.)

_Design is NOT locked. Round-2 fixes integrated; **OQ-D** awaits the user. On
sign-off a Lock stanza is appended here (date + commit), then `/plan`._
