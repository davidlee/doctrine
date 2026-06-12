# Design SL-047: Cross-kind actionable survey/next/explain/blockers CLI

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-046, REQ-238, ADR-004); doc-local refs bare — OQ-8 (SPEC-001), D1 (§7),
     DD-1 (design decisions), R1 (§8). -->

## 1. Design Problem

PRD-011 / SPEC-001 specify a derived, explainable "what should I look at next, and
why?" view over doctrine's entity graph, **across all admitted kinds**. SL-046 (the
spine, design-locked) projects every kind into a `cordage` graph and ships the
relation-only `inspect` verb. This slice — **slice 2 of 3** — builds the
operator-facing **derived priority surfaces** on top of that graph: `survey`,
`next`, `explain`, `blockers`, and the actionability extension of `inspect`.

The canon is settled: PRD-011 (`6d59397`) broadens the actionable channel to all
kinds via the status×relations synthesis (FR-008/`REQ-237`); SPEC-001 (`c3cb719`)
fixes the mechanism — `actionable = eligible ∧ ¬blocked`, uniform across kinds, the
per-kind workable|terminal partition being policy data (D12, FR-006/`REQ-238`); D9
(order_key composition), D10 (survey vs next sort keys), D11 (blocking display)
complete the contract. This slice settles the **policy data** the spec left to it
(OQ-8 partition rows) and ships the **surfaces**; it introduces **no new graph
mechanism**.

## 2. Current State

- **`crates/cordage/`** — generic graph core, consumed unchanged. Relevant API:
  `GraphBuilder::{overlay, edge, order_spec, build}`; `Graph::{ordered, in_edges,
  out_edges, reachable, provenance}`; `OverlayConfig::new(CyclePolicy, Arity)`;
  `OrderSpec`/`OrderLayer`; `EdgeAttrs{rank, age}`.
- **`src/backlog_order.rs`** — the existing dep/seq ordering adapter and the
  **reuse template**. It builds `needs`(Reject/hard) + `after`(Evict/soft) overlays
  + an `OrderSpec(needs, after)` both `Along`, mints nodes in `(exposure desc,
  created asc, canonical-id asc)` so the monotonic `NodeId` carries the fallback
  tiers, and reads back `ordered()` / `dep_cycles()` / `overrides()` over cordage
  `ordered()`/`provenance()`. Pure; sees only `OrderInput`. **This is exactly the
  D9 composition `next` needs — backlog-scoped today, cross-kind here.**
- **`src/relation_graph.rs`** (SL-046, design-locked, unbuilt) — the all-kind scan:
  walks `integrity::KINDS`, sorts ids ascending, interns each entity into
  `Projection<EntityKey>`, emits reference/lineage edges via per-kind
  `relation_edges` accessors dispatched by `outbound_for(kind, id)`, builds a Graph
  with ~11 reference overlays, and answers the `inspect` query (outbound / derived
  inbound via `in_edges` / danglers). **Excludes** `needs`/`after`/`triggers` — by
  design those are *this slice's* overlays.
- **`src/projection.rs`** (SL-046) — `Projection<K: Copy + Ord>`: `intern`
  (mint-or-get), `resolve`, `key_of`, `remap_set`. Consumed as-is with
  `K = EntityKey`.
- **`src/integrity.rs`** — `KINDS`, the corpus-wide id table; `parse_canonical_ref`
  / `kind_by_prefix` (the id-parse seam).
- **Per-kind status enums** (closed, serde-validated, each with a `*_STATUSES`
  known-set lockstep canary): `SliceStatus`, `AdrStatus`, `PolicyStatus`,
  `StandardStatus`, `SpecStatus` (PRD + tech share it), `ReqStatus`, backlog
  `Status` (5 item-kinds share it). **RV** has no *stored* status, but a derived
  `ReviewStatus{Active,Done}` is computed at read time by `review::derived_status`
  over its **authored** append-only `[[finding]]` ledger (`review.rs:312,594` —
  committed, not the gitignored runtime baton); **REC** is status-less, no
  lifecycle.

## 3. Forces & Constraints

- **ADR-001** — module layering leaf ← engine ← command, no cycles.
- **ADR-004** — relations outbound-only; reverse derived. The derived surface adds
  no stored field (REQ-074 / REQ-078).
- **SPEC-001 D1** — `cordage` locked; consumed as-is, no doctrine vocabulary added
  (REQ-079). All doctrine policy lives in the doctrine layer.
- **SPEC-001 D2 / NF-002 (REQ-095)** — the boundary test: a rule needing semantic
  interpretation ("an accepted ADR is terminal") lives in **policy**, not the core.
  The whole of this slice is policy + adapter; it adds nothing to `cordage`.
- **Determinism** (REQ-077) — no clock / RNG / `HashMap` iteration order; repo bans
  `HashSet`/`HashMap` (use BTree); `as` casts, indexing-slicing, `print_stdout`
  banned (the standing clippy denies).
- **Behaviour-preservation gate** — `backlog_order` + `cordage` suites stay green
  **unchanged**; `backlog order` output byte-identical.
- **No parallel implementation** — ride `backlog_order`'s dep/seq pattern, SL-046's
  scan + `Projection`, cordage's `reachable`/`in_edges`. Build no second ordering
  engine.
- **kind-is-data, not a trait** (mem.pattern.entity.kind-is-data-not-trait) — the
  partition and status reads are data-driven matches over `entity::Kind`.
- **Pure/imperative split** — the scan/status read is impure (shell); channel and
  partition logic are pure.

## 4. Guiding Principles

- **No new mechanism — compose what exists.** Every surface maps onto cordage +
  the `backlog_order` pattern + SL-046's scan. This slice's novelty is *policy*
  (the partition, the channels, the two sort keys) and *surface* (the four verbs).
- **Status and relations are orthogonal halves** (D12). `eligible` is pure status;
  `¬blocked` is pure relations; neither alone decides actionability.
- **Kind-agnostic by construction.** The partition, channels, and dep/seq
  extraction are uniform over `entity::Kind`. No backlog special-casing; cross-kind
  enrichment is a capture-flip away (DD-2).
- **Derived is a view, never truth.** Recompute from authored state; never write
  back (REQ-078). Reasons are structured; prose is rendered from them (REQ-072).
- **Degrade, don't lie.** A `dep` cycle or an unrecognised status surfaces a
  diagnostic and a conservative classification, never a false order / false
  actionable (REQ-076 spirit, applied to both the relation and status axes).

## 5. Proposed Design

### 5.1 System Model

SL-047 is a thin **policy + surface** layer over two existing graph adapters. ADR-001
layering:

```
command:  src/main.rs            survey / next / explain / blockers handlers;
                                 inspect handler EXTENDED (actionability section)
engine:   src/priority/          NEW — doctrine policy + the priority adapter
            graph.rs             priority adapter: shared scan → 3rd Graph
                                 (refs + dep/seq overlays + node attrs + OrderSpec)
            partition.rs         per-kind workable|terminal status table (OQ-8 policy data)
            channels.rs          eligible / actionable / blocked_by / blocking / consequence / order_key
            view.rs              structured reasons + row/explanation types
            render.rs            human + --json, riding src/listing.rs
          src/relation_graph.rs  SL-046 — its pub(crate) scan seam reused; inspect extended
leaf:     src/projection.rs      SL-046 — Projection<EntityKey>, reused as-is
          crates/cordage/        consumed, unchanged
```

`priority` (engine) → `relation_graph` (engine peer; calls its `pub(crate)` scan
seam) → `projection` (leaf) → `cordage`. No new cycle.

**Reuse map (no parallel implementation):**

| SL-047 needs | Already exists | Source |
|---|---|---|
| all-kind scan → `Projection<EntityKey>` | `KINDS` walk + sorted ids + `outbound_for` | SL-046 `relation_graph` (scan seam) |
| work/lineage overlays = **consequence** inputs (excl. `reviews`/`owning_slice`, Q3) | the reference overlays + `in_edges` | SL-046 |
| `dep`(Reject)+`seq`(Evict) overlays + `OrderSpec` → `order_key` | exact build + `ordered()`/`dep_cycles()`/`overrides()` | `backlog_order` pattern |
| transitive blockers | `Graph::reachable` | cordage |
| direct blockers / blocking | `Graph::in_edges`/`out_edges` (one hop) | cordage |
| cycle degrade (REQ-076) | `provenance().cycles()` → `dep_cycles()` | `backlog_order` pattern |
| evicted-seq provenance (explain) | `overrides()` | `backlog_order` |
| id parse / canonical render | `parse_canonical_ref` / `EntityKey::render` | SL-046 |

### 5.2 Interfaces & Contracts

**The priority adapter (engine, `graph.rs`).** Builds a **third** Graph (distinct
from `backlog_order`'s and `inspect`'s; shares only the `Projection` *type* —
SL-046 §5.1's established pattern), reusing SL-046's scan seam and adding the
dep/seq overlays + node attributes + `OrderSpec`:

```rust
// status: None = status-less REC only. RV carries a status DERIVED at read time
// (review::derived_status over its authored finding ledger → "active"/"done"),
// authored-tier like every other kind — never a runtime/gitignored read (Charge I).
// promoted: backlog `resolution == Promoted` (backlog.rs:259, the typed authority) —
// a DISTINCT exclusion from status-terminal (REQ-075 AC2), since `Status` has no
// Promoted variant. NOT read from the free-text `origin` field (F1 / Charge VI).
struct NodeAttr { kind: entity::Kind, status: Option<String>, promoted: bool }

pub(crate) struct PriorityGraph {
    graph: Graph,
    projection: Projection<EntityKey>,
    attrs: BTreeMap<EntityKey, NodeAttr>,
    consequence: BTreeMap<EntityKey, u32>,   // pre-pass inbound-reference tally
    dep_overlay: OverlayId,
    seq_overlay: OverlayId,
    ref_overlays: Vec<OverlayId>,            // SL-046's reference/lineage set (consequence inputs)
    dangling: Vec<...>,
}
fn build() -> anyhow::Result<PriorityGraph>;
```

Build order (breaks the mint-order ↔ consequence ↔ graph cycle):
1. **Scan** via SL-046's seam → the entity set, each kind's reference/lineage edges,
   and `NodeAttr` (read each kind's authored `status`; **RV's via `derived_status`**
   over its authored findings; `None` for REC only).
2. **Consequence pre-pass** — tally inbound targets of the **consequence-bearing
   label subset** (work/lineage: `specs` / `requirements` / `slices` /
   `descends_from` / `parent` / `members`; **excludes** the `reviews` / `owning_slice`
   bookkeeping edges, Q3 / Charge V) into `BTreeMap<EntityKey, u32>` directly from the
   scanned outbound edges (no graph needed yet).
3. **Mint** every node into `Projection<EntityKey>` in **`(consequence desc,
   canonical-id asc)`** order — the cross-kind analog of `backlog_order`'s
   `(exposure desc, created asc, id asc)` fallback; this monotonic `NodeId` is
   `order_key`'s tier-3 fallback (REQ-077: consequence is the importance proxy,
   canonical-id the total stable tiebreak). Mirrors `backlog_order`'s C4
   dedicated-pre-intern-pass discipline; assert distinct keys (canonical ids unique
   by prefix).
4. **Edges** — reference/lineage onto `ref_overlays` (resolve-only; unresolved →
   dangler); `needs`→`dep_overlay` (Reject, `EdgeAttrs::new(0,0)`, B→A flip),
   `after`→`seq_overlay` (Evict, `EdgeAttrs::new(rank, age)`) — the **exact**
   `backlog_order` orientation, read kind-agnostically (DD-2: only backlog populates
   them in v1).
5. `order_spec(OrderSpec::new(vec![dep Along, seq Along]))`, then `build()`.

**The dep/seq extraction (DD-2, kind-agnostic).** A `priority`-side accessor reads
`needs`/`after` via the same per-kind dispatch shape as SL-046's `outbound_for`,
over whatever kinds author them. Today only backlog → dep/seq edges connect only
backlog nodes. Non-backlog nodes carry no dep/seq edge → `¬blocked` vacuously true
→ their actionability reduces to `eligible`. When IMP-033 authors a slice→slice
`needs`, it lights up with zero change here.

**The channels (engine, `channels.rs`).** Pure over `PriorityGraph`:

```rust
fn eligible(g, n)     -> bool                         // status_class == Workable
fn blocked_by(g, n)   -> Vec<EntityKey>               // in_edges(dep,n) ∩ {status_class != Terminal}
fn blocked(g, n)      -> bool                          // !blocked_by.is_empty()
fn actionable(g, n)   -> bool                          // eligible && !blocked  (D12)
fn blocking(g, n)     -> Vec<EntityKey>               // out_edges(dep,n)
fn consequence(g, n)  -> u32                           // g.consequence[n]
fn order_key(g)       -> Vec<EntityKey>               // g.graph.ordered() remapped (D9)
fn dep_cycles(g)      -> Vec<BTreeSet<EntityKey>>     // REQ-076 degrade diagnostic
```

**The partition (engine, `partition.rs`) — OQ-8 policy data:**

```rust
pub(crate) enum StatusClass { Workable, Terminal, Unrecognised }
struct KindPartition { kind: entity::Kind, workable: &'static [&'static str], terminal: &'static [&'static str] }
const PARTITION: &[KindPartition] = &[ /* table below */ ];
pub(crate) fn status_class(kind: entity::Kind, status: Option<&str>) -> StatusClass;
```

Resolution: `Some(s)` → table lookup (`s ∈ workable → Workable`; `s ∈ terminal →
Terminal`; else **`Unrecognised`** → non-eligible **+ diagnostic**, D12 conservative
default). `None` + known status-less kind (**REC** only) → **Terminal**, *no*
diagnostic (DD-4 context-only, expected). **RV** resolves via its derived
`active`/`done` through the same table (Charge I). A backlog node with
`promoted == true` is excluded
from default active output regardless of its `status_class` — the distinct
promoted-exclusion (F1; REQ-075 AC2), surfaced as its own reason.

The drift canary asserts `workable ∪ terminal` equals each kind's status vocabulary;
for kinds with a `*_STATUSES` const (ADR/POLICY/STANDARD/BACKLOG/…) it compares to
that const, otherwise it iterates the enum's variants via `as_str` (F4). **`slice` is
stringly (open) status** (`spec.rs:1058` contrast) — no closed enum to compare, so
its canary compares against the lifecycle state-machine's status set (ADR-009; the
`slice status` transition vocabulary), the actual authority; absent that, a slice
status outside the table rides the conservative `Unrecognised` default (Charge VII).

**Surfaces (command, `main.rs`).**

```
survey   [--all] [--json]        Vec<SurveyRow>, importance order (D10)
next     [--json]                Vec<NextRow>, actionable-only, advisory (REQ-071)
blockers <ID> [--transitive] [--json]   direct blocked-by + blocks; --transitive walks reachable (REQ-073)
explain  <ID> [--json]           Explanation, always to root (D11/REQ-072)
inspect  <ID>                    SL-046 relation view + appended actionability block (same verb, SL-046 D1)
```

### 5.3 Data, State & Ownership

- **`partition.rs` owns** the per-kind workable|terminal table — the OQ-8 policy
  data (D2: semantic interpretation → policy, not core).
- **`channels.rs` owns** the channel synthesis (pure).
- **`graph.rs` owns** the priority adapter: scan reuse, dep/seq emission, node
  attrs, consequence tally, `OrderSpec`.
- **`cordage` owns** topology, eviction, reverse index, reachability.
- **Nothing owns a stored derived field** — every channel recomputes per query from
  authored state (REQ-078).

**The OQ-8 partition table (DD-3 — work-only principle):** *workable = in-flight
authoring work remains on the entity itself; terminal = published / governing /
decided / satisfied / abandoned* (the authoring is complete; further change is *new*
work — the revision out-clause below). Generalises D12's `ADR accepted → terminal`,
`proposed → workable`.

| Kind | Workable (eligible) | Terminal (default-excluded) |
|---|---|---|
| slice | proposed, design, plan, ready, started, audit, reconcile | done, abandoned |
| ADR | proposed | accepted, rejected, superseded, deprecated |
| policy | draft | required, deprecated, retired |
| standard | draft | default, required, deprecated, retired |
| PRD (product spec) | draft | active, deprecated, superseded |
| tech spec | draft | active, deprecated, superseded |
| requirement | pending, in-progress | active, deprecated, retired, superseded |
| backlog ×5 | open, triaged, started | resolved, closed (+ promoted resolution) |
| RV (review) | active (derived: open findings remain) | done (every finding terminal) |
| REC | — (no lifecycle → context-only, DD-4) | all |

**The revision out-clause (DD-3 rationale).** A governing artifact (`active`
spec/PRD, `required` policy, `accepted` ADR) re-enters actionability **not** by its
status flipping, but when a *revision* is needed — and that revision-need surfaces
as its own actionable entity (a slice / backlog item, potentially
canon-revision-flagged), never via the static partition. So the priority engine
needs no special mechanism for "active specs that need work"; it rides the existing
capture loop. **Honest scope (Charge IV):** SL-047 ranks *already-captured* work; it
does **not** surface uncaptured revision demand — a stale governing artifact that
needs revision but carries no captured revision-need is, correctly, absent from
`next`/`survey`. Detecting un-captured staleness is a separate captured signal
(`validate`/drift), explicitly out of scope. *(No drift-as-consequence net is
claimed: specs author no `drift` field and SL-046 makes backlog `drift` a dangler —
no node, no edge — so it contributes zero consequence; Charge III.)*

**`audit`/`reconcile` are workable** (live closure work) — D12 enumerated
`proposed→started` and `done/abandoned` and was silent on the closure-seam statuses;
they are unambiguously active work.

### 5.4 Lifecycle, Operations & Dynamics

1. Any surface: `build()` the priority graph once (scan → consequence pre-pass →
   mint → edges → `OrderSpec` → `build()`).
2. **`survey`**: set = `eligible` nodes (terminal excluded unless `--all`); sort
   `authored-priority → actionability → consequence desc → canonical-id` (D10).
   v1 authored-priority slot **empty** (PRD-009 OQ-001 unbuilt; D10 allows) → effective
   sort `actionability(actionable > blocked) → consequence desc → id`. Blocked rows
   carry a BLOCKED badge + direct blocker.
3. **`next`**: filter to `actionable`; order by `order_key` (cordage `ordered()` =
   D9 dep-topology → seq rank → fallback). Blocked items absent — the divergence
   feature. Advisory; mutates nothing (REQ-071).
4. **`blockers <ID>`**: direct `blocked_by` (`in_edges` dep, non-terminal) + direct
   `blocking` (`out_edges` dep); `--transitive` walks both chains via `reachable`.
   Display depth never reorders.
5. **`explain <ID>`**: always walks to root — eligibility reason, transitive blocker
   chain, order_key contributors, evicted-seq edges (`overrides()`), consequence.
6. **`inspect <ID>`**: SL-046's outbound/inbound/danglers + an appended
   actionability block (eligible/actionable/blocked + direct blockers + blocking +
   consequence).
7. A `dep` cycle (`dep_cycles()` non-empty) → emit `CycleDegraded` naming the nodes.
   cordage `Reject` **preserves** the cyclic edges (SL-046 I1) and still yields an
   `ordered()` (the NodeId/consequence **fallback** decides the cyclic pairs, since
   no trustworthy topology exists for them, F2) — so the surface **degrades to
   fallback for the affected component**, never emits a false topological order
   (REQ-076). The diagnostic flags the component as untrusted; ordering elsewhere is
   unaffected. (Same posture as `backlog_order`'s `dep_cycles()`.)

**Structured reasons (`view.rs`, REQ-072 — "structured, not prose magic"):**

```rust
enum ReasonKind {
  Eligibility { status: Option<String>, class: StatusClass },
  BlockedBy   { items: Vec<String> },
  Blocking    { items: Vec<String> },
  Consequence { inbound: u32 },
  OrderContrib{ dep_level: u32, seq_rank: Option<i32> },
  EvictedEdge { from: String, to: String, reason: OverrideReason },
  CycleDegraded { nodes: Vec<String> },
  Fallback,
}
struct SurveyRow  { id, title, kind, status, act: Actionability, consequence: u32, blockers: Vec<String>, reasons: Vec<ReasonKind> }
struct NextRow    { id, title, kind, status, act: Actionability, reasons: Vec<ReasonKind>, blockers: Vec<String>, blocking: Vec<String> }
struct Explanation{ id, eligibility, blocker_chain, order_contrib, evictions, consequence }
enum Actionability { Actionable, Blocked }   // survey set is all eligible; both variants are eligible
```

Render (human + `--json`) is produced **from** these — reasons are source of truth
(REQ-072 AC3). `render.rs` rides `src/listing.rs` + the SL-045/SL-046 read-surface
precedent (uniform `--json`).

### 5.5 Invariants, Assumptions & Edge Cases

- **I1 — direct-blocker suffices for `actionable`; no closure needed for the
  boolean.** D11's "closure always computed for eligibility" is satisfied
  transitively-for-free: if A's only direct blocker B is itself blocked, B is
  non-terminal, so A is already `blocked` by the *direct* test. `reachable` is
  needed only for `blockers --transitive` display and `explain` root-cause — not the
  actionable filter. (Cheaper and equivalent.)
- **I2 — consequence is a pre-pass tally, not a cordage read.** Counting inbound
  reference/lineage edges needs only the scanned outbound edges → one
  `BTreeMap<EntityKey, u32>` before any graph is built. Breaks the mint-order ↔
  consequence ↔ graph cycle. Drives both survey's tier-3 key and next's tier-3
  fallback mint order.
- **I3 — determinism.** No clock/RNG/map-order; `(consequence desc, canonical-id
  asc)` mint order + cordage's `ordered()` are total and stable → output identical
  under input permutation (REQ-077). Inherited from the `backlog_order` posture.
- **I4 — terminal ≠ invisible.** Terminal items are excluded from *default* active
  output; `--all` reveals them (REQ-075), and `inspect`/`<kind> list` always show
  them. The work-only partition loses no reachability.
- **I5 — derived never authored.** No channel mutates `status`/relations; no stored
  reverse field (ADR-004 / REQ-078).
- **Edge cases**: unknown prefix → clean error; entity with no relations/status →
  empty channels, not an error; abandoned dep-predecessor → treated as
  non-blocking (terminal); a `seq` edge contradicting `dep` → evicted + surfaced
  (`overrides()`, D9); status-less kind → non-eligible, no diagnostic.

## 6. Open Questions & Unknowns

All slice-design OQs are resolved (DD-1..DD-4, OQ-8 partition table). Remaining
unknowns are downstream, not blocking:

- **Persisted cache** — v1 recomputes per query (SPEC-001 H1: full recompute is fine
  at the target scale); no persisted cache built. `--json` stamps `policy_version`
  (REQ-094 spirit + lets a future cache slot in without reshaping output). A
  policy-stamped disposable cache is a follow-up.
- **Slice phase-rollup actionability** (DD-4) — slice's mid-flight progress lives in
  the gitignored runtime phase tree; reading it would enrich mid-flight-slice
  actionability. Deferred to keep the v1 scan over authored state. (RV's derived
  active/done is **not** here — it is authored-derived and admitted to v1, Charge I.)
- **Coverage-driven requirement actionability** — a req whose observed
  `CoverageStatus` is Failed/Blocked is arguably work; v1 uses authored `ReqStatus`
  only (D12: lifecycle status). The 2nd-enum axis is a follow-up.

## 7. Decisions, Rationale & Alternatives

- **DD-1 — design now against locked SL-046; implement after SL-046 lands.** SL-046
  is `ready` (design-locked) → the scan/Projection/overlay contract is firm to
  design against (standard doctrine flow: designs ride locked designs).
  Implementation sequences after SL-046 lands. *Alt rejected*: build SL-046 first
  then design — stalls SL-047, re-treads a locked contract.
- **DD-2 — dep/seq engine kind-agnostic; capture deferred.** ADR-010 (accepted)
  **explicitly excluded** `needs`/`after` from its unified relation contract
  ("the dep/sequence axis SL-047 owns"); cross-kind dep *authoring* (IMP-033) has no
  governance and is PRD-009 capture surface. So v1 builds the cross-kind dep/seq
  *overlay + policy* kind-agnostic and consumes existing backlog `needs`/`after`;
  cross-kind blocking auto-lights when IMP-033 authors edges. The broadened intent
  (PRD-011 §5) rests on the **`eligible` (status) half**, which is fully cross-kind
  here — every PRD-011 / SPEC-001 acceptance gate is discharged by the status half
  alone (the v1 honest contract: **cross-kind eligibility + backlog-only,
  wired-dormant dep blocking**, Charge II); no gate needs a non-backlog `dep` edge. *Alts
  rejected*: full cross-kind dep authoring in SL-047 (pulls capture + an unsettled
  governance call into a read-derived slice — breaks the SL-046/SL-047/SL-048
  partition); narrow slice→slice-only capture (still a capture-surface + governance
  increment).
- **DD-3 — OQ-8 partition = work-only principle** (§5.3 table). `active`/governing/
  `accepted`/satisfied = terminal-as-work; only in-flight authoring statuses are
  workable. Grounded in PRD-011 §1's actionable examples (all in-flight: "slice
  awaiting design, requirement pending, spec in draft") and D12's explicit ADR rows.
  The revision out-clause (§5.3) covers "active artifacts that need work" via the
  capture loop, not the partition — so no third "context" class / D12 amendment is
  needed. *Alt rejected*: keep governing artifacts workable (they'd then show in
  `next` as do-now work — misreads a finished artifact; needs a canon revision).
- **DD-4 — v1 actionability reads authored state only (incl. RV's authored-derived
  status); only genuinely-runtime state is deferred.** **RV is admitted to v1** (User
  call, Charge I): its `active`/`done` is a pure read-time derivation
  (`review::derived_status`) over its **authored** finding ledger — authored-tier, not
  the gitignored runtime baton — so an `Active` RV is `eligible`. **REC** has no
  lifecycle (forced status-less → non-eligible via the status-less path, no
  diagnostic — not barred as a kind, D12; its consequence still propagates). The only
  genuinely-deferred runtime signal is the **slice phase-rollup**
  (`.doctrine/state/slice/`, gitignored) — a coherent follow-up enriching mid-flight
  slice actionability, kept out of v1 so the scan stays over authored state.
  *(Correction: the locked design wrongly attributed RV's derived status to a "runtime
  findings tree"; the findings are authored — Charge I.)*
- **D5 — the priority adapter builds a third graph from a shared scan seam.**
  `backlog_order`, `inspect`, and `priority` each build their own Graph sharing only
  the `Projection` *type* (SL-046 §5.1 pattern). The all-kind scan is factored into
  a `pub(crate)` seam in `relation_graph`, consumed by both `inspect` and `priority`
  — **fed into SL-046** as a coordination requirement (SL-046 exposes the scan seam;
  additive, non-breaking to `inspect`). *Alt rejected*: parameterize one
  `relation_graph` build with an inspect|priority mode flag — couples the two
  surfaces' evolution inside SL-046's inspect-focused module.
- **D6 — no persisted cache in v1; `policy_version` stamped in `--json`.** SPEC-001
  H1 licenses full recompute at scale; REQ-094 is satisfied vacuously (nothing
  persisted) while the stamp preps the follow-up cache. The stamp is a
  `PRIORITY_POLICY_VERSION` const (F3), bumped when the partition or channel
  semantics change. *Alt rejected*: build the cache now — premature; correctness is
  recompute regardless.

## 8. Risks & Mitigations

- **R1 — SL-046 lands with a contract drift from its locked design.** *Mitigation*:
  design against the locked design.md; the scan-seam coordination (D5) is fed into
  SL-046 so the seam exists when SL-047 implements. Integration risk bounded by
  SL-046 being design-locked.
- **R2 — consequence-as-fallback couples ordering to reference-edge counts.** A
  pure-noise inbound reference could nudge fallback order. *Mitigation*: the
  consequence-bearing labels are a **policy subset** (work/lineage only;
  `reviews`/`owning_slice` bookkeeping edges excluded, Q3 / Charge V) — administrative
  attachment no longer perturbs importance; fallback is the *last* tier (after dep +
  seq); canonical-id is the total tiebreak, so order is always deterministic and
  explainable; consequence is surfaced in reasons.
- **R3 — partition drift** (a new status enum variant un-mirrored in `PARTITION`).
  *Mitigation*: the lockstep canary test (`workable ∪ terminal == <kind>_STATUSES`)
  reds on drift — never a silent `Unrecognised` in production.
- **R4 — behaviour-preservation.** SL-047 adds a module + a scan-seam call; it must
  not touch `backlog_order`'s build. *Mitigation*: `backlog_order` + `cordage`
  suites stay green unchanged; `backlog order` golden byte-identical (the gate).
- **R5 — reading per-kind authored `status` for `NodeAttr` re-treads per-kind
  parsing.** *Mitigation*: ride the same per-kind module accessors SL-046 adds
  (`relation_edges` neighbours); add a thin `status` accessor per kind where not
  already exposed — cohesion (each module parses its own).

## 9. Quality Engineering & Validation

Pure unit tests (channels/partition) + black-box CLI goldens (the 4 verbs;
`mem.pattern.testing.black-box-cli-golden`) over a **seeded multi-kind corpus**:

- **Divergence (D10)** — a workable-but-blocked item: top-area of `survey` + BLOCKED
  badge, **absent** from `next`; an equal unblocked item leads `next`.
- **Blocking display (D11/REQ-073)** — `blockers --transitive` + `explain` surface
  the full chain; `survey`/`next`/`inspect` rows direct-only; depth toggle never
  reorders.
- **Cycle degrade (REQ-076)** — a `dep` cycle → `CycleDegraded` naming nodes, no
  false topo.
- **Determinism (REQ-077)** — identical output under input permutation.
- **Cross-kind actionability (FR-006/REQ-238, FR-008/REQ-237)** — non-backlog
  workable+unblocked item (incl. an `Active` RV) in `next`; same-kind terminal
  omitted; one comparable view across kinds. **Dep-blocking is verified
  backlog-scoped** (the dep-blocked-omitted fixture is a backlog item — non-backlog
  kinds cannot author a `dep` edge in v1; SPEC-001 D12's verification does not require
  the blocked item be non-backlog, Charge II). Cross-kind blocking is wired
  kind-agnostic but **dormant** until IMP-033.
- **Partition drift canary (DD-3)** — `workable ∪ terminal == <kind>_STATUSES` per
  partitioned kind.
- **Terminal exclusion (REQ-075)** — terminal excluded by default, revealed by
  `--all`; a **promoted** backlog item excluded as its own reason even if its
  `status` is not terminal (F1).
- **Conservative default (D12) / status-less (DD-4)** — unrecognised status →
  non-eligible + diagnostic; **REC** status-less → non-eligible, **no** diagnostic;
  an **`Active` RV is eligible**, a `Done` RV terminal (derived status, Charge I).
- **Consequence** — inbound count over the **work/lineage label subset** drives
  `survey` order + `next` fallback; a `reviews`/`owning_slice` edge does **not** raise
  a target's consequence (Charge V).
- **Structured reasons (REQ-072)** — every classification carries reasons; `--json`
  stamps `policy_version`.
- **Behaviour-preservation gate** — `backlog_order` + `cordage` suites green
  **unchanged**; `backlog order` byte-identical.

## 10. Review Notes

### External adversarial pass — Inquisition (codex GPT-5.5) — integrated

Full record: `inquisition.md`. Seven charges; every one re-verified against source
before integration. User dispositions (3 design calls) folded in.

- **Charge I (major) — RV deferred on a false premise.** The locked design claimed
  RV's active/done came from a "gitignored runtime findings tree." Source disproves
  it: findings are **authored** (`review.rs:594`, committed via `.gitignore` negation)
  and `review::derived_status` (`review.rs:312`) is a pure read-time function over
  them. **User call: admit RV to v1** — `Active` RV is `eligible`. §2/§5.2/§5.3
  partition row/§7 DD-4/§9 corrected; the `.doctrine/state/review/` runtime baton is
  *not* the findings.
- **Charge II (major) — "all gates pass" rested on an unconstructable state.**
  Non-backlog kinds cannot author a `dep` blocker in v1 (`backlog.rs:946` `parse_ref →
  ItemId`). **User call: v1 contract = cross-kind eligibility + backlog-only,
  wired-dormant blocking.** §7 DD-2 + §9 verification reworded; the dep-blocked
  fixture is backlog-scoped (SPEC-001 D12 does not require it be non-backlog).
- **Charge III (major) — drift-consequence net was a phantom.** Specs author no
  `drift` field and SL-046 makes backlog `drift` a dangler (no node/edge) — zero
  consequence. The claim is **struck** (§5.3).
- **Charge IV (major) — revision out-clause over-promised.** Reworded to the truth:
  SL-047 ranks *captured* work; it does not surface uncaptured revision demand (§5.3).
- **Charge V (major) — consequence tally was indiscriminate.** `reviews`/`owning_slice`
  bookkeeping edges perturbed importance/fallback. **User call: work/lineage label
  subset only**; the two bookkeeping labels are excluded (§5.1/§5.2/§5.3/§8 R2/§9).
- **Charge VI (minor) — F1 muddied "promoted" authority.** Fixed to
  `resolution == Promoted` (`backlog.rs:259`); the free-text `origin` is not read
  (§5.2).
- **Charge VII (minor) — slice drift-canary over-claimed.** `slice` is stringly
  status; its canary compares against the ADR-009 lifecycle status set, else rides the
  conservative `Unrecognised` default (§5.3).

**Survived interrogation (no change):** layering no-cycle, behaviour-preservation
gate, `cordage` boundary-purity, D12 `eligible ∧ ¬blocked` synthesis, I1
direct-blocker-suffices, I2/F2 consequence-pre-pass cycle-break and `Reject`-preserves
/ fallback-decides degrade.
