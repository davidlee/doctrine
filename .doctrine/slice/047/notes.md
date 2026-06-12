# SL-047 implementation notes

Durable cross-phase notes. Runtime progress lives in the gitignored phase sheets;
this file carries the API/seam contract each phase hands the next.

## PHASE-01 — priority graph adapter + scan seam (landed `cb03be6`)

**The D5 scan seam (`src/relation_graph.rs`).** Extracted the reusable half of the
old private `build_relation_graph` as a `pub(crate)` seam:

- `relation_graph::scan_entities(root) -> Vec<ScannedEntity{ key, kind, status, outbound }>`
  — the all-kind KINDS-walk (table order, ids sorted asc), one row per entity.
- `status_for` — per-prefix dispatch (mirrors `outbound_for`) reading each kind's
  AUTHORED status: REC → `None`; RV → `review::derived_status_string` over its finding
  ledger; all others → `meta::read_meta`.
- `inspect`'s `build_relation_graph` is now re-expressed on `scan_entities` — mints in
  the same scan order → **byte-identical** output (behaviour-preservation gate held).
- `EntityKey` promoted to `pub(crate)` so `priority` reuses it (no parallel key type).

**The third graph (`src/priority/graph.rs`).** `pub(crate) fn build() -> Result<PriorityGraph>`:
scan → consequence pre-pass (inbound tally over the work/lineage label SUBSET only:
`specs/requirements/slices/descends_from/parent/members`; `reviews`/`owning_slice`
EXCLUDED) → mint in `(consequence desc, canonical-id asc)` with the `backlog_order` C4
pre-intern pass + distinct-key assert → edges (reference/lineage → `ref_overlays`
resolve-only/dangler; `needs` → `dep_overlay` Reject B→A `(0,0)`; `after` →
`seq_overlay` Evict `(rank,age)`, kind-agnostic per DD-2) → `OrderSpec[dep Along,
seq Along]` → cordage `build()`.

`PriorityGraph` fields (what PHASE-02 reads): `graph, projection, attrs:
BTreeMap<EntityKey,NodeAttr>, consequence: BTreeMap<EntityKey,u32>, dep_overlay,
seq_overlay, ref_overlays: Vec<OverlayId>, dangling`. `NodeAttr{ kind:
&'static entity::Kind, status: Option<String>, promoted: bool }` — `status` is the
RAW authored string (NO workable/terminal classification yet — that is PHASE-02).
`promoted` from the typed `Resolution::Promoted` via new `backlog::dep_seq_for` /
`DepSeq` (yields needs/after + promoted, read once per backlog item).

**Thin accessors added:** `review::derived_status_string`, `backlog::dep_seq_for`
(+`DepSeq`).

### ⚠ Flag for PHASE-02 (carried into its worker prompt)

The priority adapter has **no live CLI consumer yet**, so `graph.rs` rides a
`not(test)`-scoped self-clearing `#[expect(dead_code)]` (module-level; item-level on
`ScannedEntity.kind`/`.status`) per `mem.pattern.lint.dead-code-expect-vs-cfg-test`.
PHASE-02/03 wiring a real caller must **remove** these suppressions (the `expect`
fails if the lint stops firing). `backlog::dep_seq_for`/`DepSeq` are reachable from
live code only via priority but are test-exercised; keep a live caller or they go dead.

## PHASE-02 — partition + channels, pure policy core (landed `1402dc3`)

**`src/priority/partition.rs`.** `pub(crate) enum StatusClass{Workable,Terminal,
Unrecognised}` + `const PARTITION` (the §5.3 table verbatim, keyed by `kind.prefix`
since `entity::Kind` is not `Eq`/`Copy`). `pub(crate) fn status_class(&entity::Kind,
Option<&str>) -> StatusClass`: `Some(s)`→table; `None`+REC→Terminal (no diagnostic);
unknown status→Unrecognised. Drift canary reads each kind's REAL `*_STATUSES` const
(now `pub(crate)`: adr/backlog/policy/standard/spec/review/slice; req already was);
slice binds the ADR-009 lifecycle set via `SLICE_STATUSES`; `SPEC_STATUSES` covers
both PRD + tech-spec rows.

**`src/priority/channels.rs`** — pure over `PriorityGraph`: `eligible`,
`blocked_by` (`in_edges(dep) ∩ {class != Terminal}`, BTreeSet-deduped), `blocked`,
`actionable = eligible && !blocked` (D12; direct-blocker I1, no closure), `blocking`
(`out_edges(dep)`), `consequence` (reads `g.consequence`), `order_key`
(`graph.ordered()` remapped via `projection.key_of`, the `backlog_order::ordered`
shape), `dep_cycles` (provenance cycles filtered to dep overlay → `remap_set`).
**`promoted` is its OWN channel** reading `NodeAttr.promoted` (NOT folded into
`status_class`, per F1).

### ⚠ Flags for PHASE-03

1. **THREE dead-code suppressions to self-clear.** `graph.rs`, `partition.rs`, AND
   `channels.rs` each carry the `not(test)` self-clearing `#[expect(dead_code)]`
   (no live caller until the CLI lands). PHASE-03 wires the four verbs + inspect
   block — the live consumer — which must make ALL THREE suppressions stop firing.
   Remove each as its module gains a live caller (the `expect` errors if left when
   the lint no longer fires).
2. PHASE-03 render inputs: `StatusClass::Unrecognised` (D12 conservative diagnostic),
   `promoted()` (F1 reason), `dep_cycles()` (REQ-076 cycle-degrade) are the diagnostic
   signals the surfaces layer renders. `order_key` is the topo order; seq-rank /
   consequence fallback tiers are already baked into the graph's OrderSpec + mint
   order from PHASE-01.
3. `status_class` takes `&entity::Kind` (by ref — not Copy/Eq); `NodeAttr.kind` is
   already `&'static entity::Kind`; identity is via `kind.prefix`.

## PHASE-03 — surfaces, structured reasons, render + goldens (landed `ae569e4`)

**`view.rs`** — `ReasonKind` + `SurveyRow/NextRow/Explanation/Actionability`
(+`BlockersView`/`ActionabilityBlock`), the render source of truth (REQ-072).
**`render.rs`** — human (rides `listing::render_table`) + `--json` (`serde_json::json!`)
FROM the view types; stamps `PRIORITY_POLICY_VERSION = "priority.v1"`; `write!`, no
trailing newline (golden contract). **`surface.rs`** — the impure shell: one
`graph::build` per verb, composing the pure channels into rows.

**`main.rs`** — four read verbs: `survey [--all][--json]`, `next [--json]`,
`blockers <ID> [--transitive][--json]`, `explain <ID> [--json]`; plus the `inspect`
actionability block. **The inspect block composes at the COMMAND layer**
(`relation_graph::run`→`render(root,id,format)->String` + `inspect_value()` exposed;
`main` appends the priority block) — `relation_graph` never calls `priority` (ADR-001).
**Promoted backlog items are excluded from survey AND next** (F1/REQ-075 — the worker
caught during smoke that the own-reason exclusion must also drop them from the
actionable worklist). Titles via lenient `relation_graph::title_for` (status-less
RV/REC safe), captured in `scan_entities`→`ScannedEntity.title`→`NodeAttr.title`;
pure channels stay title-free. Added `channels::blocked_by_transitive`/
`blocking_transitive` (cordage `reachable`) + `evicted_seq_edges`.

Smoke-tested live: `survey`/`next` rank cross-kind by consequence; `explain SL-047`
reads `plan → Workable`. Goldens: `e2e_priority_golden` (13), `e2e_inspect_golden`
(+additive actionability block, relation portion byte-identical).

### Audit flags (for /audit)

- **Narrowed dead-code expects remain** (`not(test)`, per-field): priority
  `dangling`/`ref_overlays` and `ReasonKind::Fallback` are design-vocabulary
  completeness with no v1 surface emitting them. Investigated, intentionally kept
  (not blindly re-suppressed) — audit should confirm this is acceptable or capture a
  follow-up to wire/drop them.
- **`explain` fidelity (v1 honest scope):** `OrderContrib.dep_level` uses the
  transitive-prereq count as an agent-legible depth proxy (the composed cordage level
  is internal); `seq_rank` is `None` in v1, surfaced instead via `evicted_seq_edges`.
- **Cordage denylist note (worker-raised, UNCONFIRMED):** the worker reported
  `cargo test -p cordage --test denylist` failing on a `<task>` token. The token is
  ABSENT from `crates/cordage/README.md` at base `d2406e6` and at HEAD — claim does
  not reproduce against the README; likely a test-fixture artifact. Outside the
  `just check` gate (gate is green) and outside SL-047 scope. Audit to investigate /
  capture a cordage-hygiene backlog item if real.

## Audit (RV-007) — reconciliation outcome

Audited 2026-06-12 against design §9/§10, plan EX/VT, SPEC-001/PRD-011/ADR-001/004/009/010.
Gate green; four verbs smoke-verified live; charge-bound §10 facts held (asserted by the
13+9 passing goldens). Three findings raised, all terminal, NO blocker:

- **F-1 (minor → follow-up, ISS-007).** The cordage denylist note WAS real (handover's
  "does not reproduce" was a stale baked-`CARGO_MANIFEST_DIR` artifact). `cargo test -p
  cordage --test denylist` is red on a whole-word `task` in `crates/cordage/README.md`
  (REQ-079) — pre-existing (dc120a7), disjoint from SL-047, and outside the gate (`just
  check`'s `cargo test` tests the root package only; cordage needs `--workspace`/`-p`).
  Kept out of SL-047; captured as ISS-007 + recorded
  `mem.pattern.build.just-check-tests-root-package-only`.
- **F-2 (nit → tolerated).** Flag-1's `dangling`/`ref_overlays` expects are GONE at HEAD
  (now read); only `ReasonKind::Fallback`'s self-clearing `#[expect(dead_code)]` remains —
  §5.4 vocabulary completeness, accepted.
- **F-3 (minor → tolerated).** `explain` v1 honest scope (`seq_rank=None` via
  `evicted_seq_edges`; `dep_level` transitive-prereq proxy) meets REQ-072 + D11.

RV-007 `done · await=none`. Ready for `/close`.
