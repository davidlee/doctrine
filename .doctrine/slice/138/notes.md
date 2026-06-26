# Notes SL-138: Relation-transitive walk for inspect — analogous to blockers --transitive

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — cordage depth-bounded reachability primitive (done, green)

**Delivered.** `pub struct Reach { depths: BTreeMap<NodeId,usize>, truncated: bool }`
at the cordage crate root; public `Graph::reachable_bounded(overlay, node, direction,
max_depth: Option<usize>) -> Reach`; private `query::reachable_bounded`. `walk_bfs`
rewritten to thread depth (frontier carries `(NodeId, usize)`) and a cap, returning a
private `BfsWalk { order, depths, truncated }`. `query::reachable` re-expressed over
`reachable_bounded(.., None)`; `Graph::reachable` keeps delegating to it; `spine_path`
consumes `.order` and ignores depth.

**Behaviour-preservation (the #1 risk) held byte-for-byte.** The `None` path through
the new `walk_bfs` is iteration-identical to the pre-SL-138 walk (same order vec, same
visited-insert order), so `reachable`/`spine_path` are unchanged. Proof: existing
cordage suite + full workspace bin suites (blockers/inspect/relation) green UNCHANGED
under `just gate`.

**Depth/truncation semantics (design D6, F5).** `depths` = min-hop (BFS first-visit
wins; verified by the diamond VT where a node reachable at 2 and 3 hops records 2).
`truncated` ⟺ a node at the cap had a still-unvisited successor — genuinely deeper than
the cap by BFS ordering, never a false-positive on a within-cap node reached another
way. `start` excluded (only depth-0 entry, removed in `reachable_bounded`).
`Direction::None` / foreign overlay / foreign node → empty `Reach`.

**Verification.** `tests/reachability.rs` +7 VTs (15/15 green). `cargo test -p cordage`
green (incl. the denylist suite — new names `Reach`/`depths`/`truncated`/`max_depth`/
`reachable_bounded` are product-neutral). `just gate` green; clippy zero warnings; fmt
clean.

**For PHASE-02.** `Graph::reachable_bounded` is the engine seam `transitive_from` walks
(per overlay × per direction). `depths` is returned but unconsumed by the display today
(D6 — a future path/tree view consumes it with zero cordage rework). `predecessor_cone`
deliberately untouched (C3).

**Commit:** see `feat(SL-138): PHASE-01` (code + this notes/sheet, SL-138 paths only).

## PHASE-02 — relation_graph: transitive_from + view + render (done, green)

**Delivered (all in `src/relation_graph.rs`, file-disjoint).** Engine-layer
`enum TransitiveDir { Inbound, Outbound, Both }` (ADR-001 — NOT cli.rs);
`struct TransitiveGroup { label, targets, truncated }`; `struct TransitiveView
{ id, max_depth, truncated, inbound: Option<…>, outbound: Option<…> }`.
`transitive_from(scanned, _root, id, dir, labels, max_depth) -> Result<TransitiveView>`
rides `build_relation_graph_from` + `require_minted` (EX-4 existence gate, shared
with `inspect_from`), walks per selected overlay × direction (inbound=`Direction::Against`,
outbound=`Along`) via `Graph::reachable_bounded`, maps `depths.keys()` →
`projection.key_of` → `EntityKey::canonical`, sorts id-ascending (REQ-077).
`render_transitive_human` + `render_transitive_json`/`transitive_value` pin the §5
C4 contract.

**Empty-group contract (resolved an under-specification).** §5 said "empty group
renders (none) … targets: []" but the §4 example shows inbound with 3 non-empty
labels and outbound as `(none)`, and the §5 JSON envelope shows `"outbound": []`.
Reconciled: a `TransitiveGroup` is emitted ONLY for labels with ≥1 reachable
target; a REQUESTED direction with no groups → `Some(vec![])` → `(none)` / `[]`; a
NON-requested direction → `None` → omitted (no table section, no JSON key). A
truncated walk always has ≥1 target (the cap node is in `depths`), so suppressing
empty groups never drops a truncation signal.

**Predicate is table-derived (C2/EX-3).** `transitive_labels` selects the default
set from `OverlayMap::by_label.keys()` (allocated from `RELATION_RULES` non-`Unvalidated`)
and rejects any explicit `labels` entry lacking an overlay — the no-overlay set
`{contextualizes, drift, decision_ref}` — with a "not transitively walkable" error.
VT-3 asserts the default set == the table's resolvable labels (no hardcoded list).
Unknown-NAME rejection (`bogus`) is PHASE-03 CLI (`RelationLabel::from_name`); the
engine only sees parsed `RelationLabel`s.

**F3 role collapse.** The cordage overlay is label-keyed (R5) — roles ride the edge
payload, not the graph — so a transitive `references` walk follows the single
`references` overlay and collapses roles into ONE section (VT-2 proves it with
implements + concerns from one SL → one `references` group).

**JSON key order.** `serde_json` is default (no `preserve_order`) → keys serialize
alphabetically. So "inbound before outbound" holds for free, and omitting a
`None` direction's key satisfies "non-requested direction absent". The §5 example
showed `kind` first illustratively; the real (golden-pinned) order is alphabetical
(`id, inbound, kind, max_depth, outbound, truncated`), matching the existing
`inspect` golden.

**`_root` is intentionally unused (minor, for audit).** The §5 + plan EX-1
signature lists `root: &Path`, but the relation-only walk reads nothing per-entity
from disk (neither `build_relation_graph_from` nor `require_minted` take root). Kept
as `_root` for call-site symmetry with `inspect_from`/`render_from` (PHASE-03
threads the same `(scanned, root, id, …)` tuple), honoring the locked signature —
NOT a deviation. `warnings=deny` + `unused=deny` forced the leading underscore.

**Next-phase dead-code dance.** The whole transitive subgraph (11 items) has no
non-test caller until PHASE-03 wires `inspect --transitive`, so each carries
`#[cfg_attr(not(test), expect(dead_code, reason="SL-138 PHASE-03 wires …"))]` (the
`inspect` convenience-wrapper precedent). PHASE-03 RETIRES these as it adds the
caller (else `unfulfilled_lint_expectations` fires under not(test)).

**Verification.** 6 unit tests (VT-1..5 + human render shape), all green; full
`just gate` green (clippy `--workspace` zero warnings, fmt clean, every existing
suite green UNCHANGED — additive only, no edit to `inspect_from`/`inspect_value`).

**For PHASE-03.** Call surface: `transitive_from`, `render_transitive_human`,
`render_transitive_json` (or `transitive_value` to inject). Map the clap `DirArg`
(up/down aliases) DOWN to `TransitiveDir`; validate `--labels` via `from_name` +
the overlay-backed predicate; `--max-depth` absent→Some(5), `0`/`all`→None, N→Some(N);
gate memory refs (F2) before the memory early-return. Remove the dead-code `cfg_attr`s.

**Commit:** `feat(SL-138): PHASE-02` (code + notes, SL-138 paths only).
