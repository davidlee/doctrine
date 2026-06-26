# SL-138 Design — Relation-transitive walk for `inspect`

> Surface (a) of RFC-001 (transitive impact / blast radius). Co-designed
> 2026-06-26. Supersedes the original `slice-138.md` scope where they conflict
> (see §8 — the outbound-only non-goal is **lifted**; the `governed_by` example
> was direction-confused and is corrected).

## 1. Problem

`inspect <ID>` renders **1-hop only** (outbound / inbound / danglers). The
team-load-bearing question — *"if I change ADR-005, what transitively depends on
it?"* — has no surface. `blockers --transitive` walks N-hop but only over the
dep/seq (`needs`/`after`) overlay; the cross-kind **relation** overlays
(`governed_by`, `implements`, `descends_from`, `supersedes`, …) have no
transitive query.

The reachability primitive exists and is clean: SL-140 unified cordage's walks,
closing IMP-020 (the triplication gate IMP-120 named). `cordage::reachable`
already walks any overlay in either direction (`Direction::Along` = out-edges,
`Direction::Against` = in-edges). The missing piece is **exposure**, plus a
**depth bound** the current primitive does not provide.

## 2. Locked decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | A `--transitive` **flag on `inspect`**, not a new `impact` verb | Reuses inspect's scan/gate/render plumbing entirely; smallest correct change. Folds IMP-120's intent into the inspect surface. |
| D2 | **Direction-selectable**, `--direction inbound\|outbound\|both`, `up`/`down` aliases | `reachable` supports both for free; outbound-only (the old non-goal) is arbitrary and contradicts the impact thesis. `inbound` = `Against` = blast radius; `outbound` = `Along` = derivation/governance ancestry. |
| D3 | **Default `both`, rendered as two separated sections** (not a merged union) | Mirrors bare `inspect` (outbound + inbound sections); the awareness view. Union would discard the direction, the most useful bit. `--direction inbound` is the tracing tool. |
| D4 | **Per-label sections, all overlay-backed labels by default**; `--labels a,b` narrows (comma-separated, `--kinds` precedent; `--label` hidden alias) | Mirrors inspect's per-label grouping; a `governed_by` closure and a `related` closure are different questions — never unioned. |
| D5 | **Uniform `--max-depth 5`** default; `--max-depth N` overrides; `0`/`all` = unbounded; **truncation indicator** when the cap bites | 5 reaches the bottom of the real lineage spine (PRD→SPEC→REQ→SL→phase ≈5) so it rarely truncates legitimately, but cuts a pathological hub. Uniform (not direction-conditional). |
| D6 | Add **`cordage::reachable_bounded`** returning **depth-tagged** results (`BTreeMap<NodeId, usize>` + `truncated`), display consumes only the node set today | The cap *already forces* per-node depth tracking inside `walk_bfs`; returning it vs dropping it is ~one field. One cordage excursion instead of two; a future path/tree view consumes `depths` with zero cordage rework. Depth values unit-tested in cordage now. |

**Non-goals (this slice):** indented-tree / path render (the `depths` field is
returned but unconsumed — a clean follow-up); inbound default-exclude of `related`
noise (escape hatch is `--labels`); **no-overlay labels** in transitive — the
`TargetSpec::Unvalidated` set is exactly **`{contextualizes, drift, decision_ref}`**
(C2; pinned by the complement test at `relation_graph.rs:~1690`), 1-hop-only by
nature, silently omitted from the default set and rejected by `--labels`.

**Known limitations (from adversarial review):**
- **F2 — memory refs.** `--transitive` operates on the **entity** relation graph
  only. A memory ref (`mem_*` / `mem.key`) + `--transitive` is **rejected** with a
  clear error pointing at `retrieve --expand <N>` (the memory graph's own
  transitive surface). `run_inspect` must gate this *before* its memory early-return.
- **F3 — `references` is role-agnostic when transitive.** The cordage overlay is
  label-keyed; roles (`implements`/`scoped_from`/`concerns`) ride the edge payload,
  not the graph (R5). 1-hop inspect re-keys by `(label, role)` from the payload, but
  a transitive walk follows the single `references` overlay — so the transitive
  `references` section is **one collapsed section, not per-role**. Documented; a
  per-role transitive walk is out of scope (would need payload-aware traversal).

## 3. Governance alignment

- **ADR-001 (layering).** Traversal mechanism lives in the cordage leaf
  (`reachable_bounded`); `relation_graph` consumes it; the command layer renders.
  The transitive view is relation-only — it does **not** touch `priority`, so no
  actionability block, no `relation_graph`→`priority` up-call. Layering preserved.
- **ADR-004 (outbound-only storage, derived reciprocity).** The inbound walk reads
  nothing stored — it walks `Against` over the same per-label overlay, exactly as
  1-hop inbound is derived from `in_edges` today. No reverse field invented.
- **SL-140 single-locus intent.** Depth + cap thread through `walk_bfs` — the loop
  SL-140 shares between `reachable` and `spine_path` (C3: `predecessor_cone`
  deliberately does *not* use it, and is untouched here). `spine_path` ignores the
  new depth field; only `reachable`/`reachable_bounded` consume it. No new walk.

## 4. Current vs target behaviour

**Current:** `inspect ADR-005` → outbound / inbound / danglers, each 1 hop,
grouped by `(label, role)`. Plus an actionability block.

**Target (additive):** `inspect ADR-005 --transitive` → relation-only transitive
view (no actionability block):

```
ADR-005 — transitive (depth 5)

depends on this (inbound):
  governed_by:   SL-012, SL-047, SL-133, SL-146
  implements:    REQ-054
  references:    RFC-002, SL-149
this depends on (outbound):
  (none)

… some chains truncated at depth 5 — re-run with --max-depth all
```

Bare `inspect ADR-005` (no `--transitive`) is **byte-unchanged** (regression gate).

```bash
inspect ADR-005 --transitive                          # both directions, all labels, depth 5
inspect ADR-005 --transitive --direction inbound      # blast radius only
inspect PRD-001 --transitive --direction outbound --max-depth all   # full derivation closure
inspect SL-047 --transitive --labels governed_by,references         # narrow to two labels
inspect ADR-005 --transitive --json
```

## 5. Code impact

| File | Change |
|------|--------|
| `crates/cordage/src/query.rs` | **new** `reachable_bounded(out, incoming, overlay, start, direction, max_depth) -> Reach`; depth+cap threaded through `walk_bfs`; `reachable` re-expressed as `reachable_bounded(.., None)` (behaviour-identical). |
| `crates/cordage/src/lib.rs` | declare `Reach` at crate root; add public `Graph::reachable_bounded` method (cordage's API is flat/re-export-free — C5); re-express `Graph::reachable` over it. |
| `src/relation_graph.rs` | **new** `TransitiveView` / `TransitiveGroup` + `transitive_from(scanned, root, id, dirs, labels, max_depth)`; reuses `build_relation_graph_from` + the `require_minted` existence gate; per-overlay × per-direction `reachable_bounded`, NodeId→EntityKey→canonical. **new** `render_transitive_human` / `render_transitive_json`. |
| `src/commands/inspect.rs` | `run_inspect` branches on `transitive`: route to the transitive view (relation-only — no actionability/priority call). |
| `src/commands/cli.rs` | `Inspect` gains `--transitive`, `--direction`, `--labels` (+`--label` alias), `--max-depth`; new `Dir` `ValueEnum`; dispatch threads them. |

### Signatures

```rust
// crates/cordage/src/lib.rs — the PUBLIC surface (C1). relation_graph holds a
// `Graph`, never the private `out`/`incoming` indices, so the new entry point is a
// Graph METHOD, mirroring the existing `Graph::reachable`.
pub struct Reach {
    pub depths: BTreeMap<NodeId, usize>, // node → min hops from start; start excluded (strictness preserved)
    pub truncated: bool,                 // cap bit AND an unvisited neighbour remained beyond it
}
impl Graph {
    pub fn reachable_bounded(&self, overlay: OverlayId, node: NodeId,
        direction: Direction, max_depth: Option<usize>) -> Reach
        { query::reachable_bounded(&self.out, &self.incoming, overlay, node, direction, max_depth) }
    // existing `reachable` re-expressed over it — behaviour-identical:
    // pub fn reachable(..) -> BTreeSet<NodeId> { self.reachable_bounded(overlay, node, dir, None).depths.into_keys().collect() }
}

// crates/cordage/src/query.rs — the private helper (NOT public; OutIndex/InIndex are crate-private).
pub(crate) fn reachable_bounded(
    out: &OutIndex, incoming: &InIndex,
    overlay: OverlayId, start: NodeId,
    direction: Direction, max_depth: Option<usize>, // None = unbounded == today's reachable
) -> Reach;
// query::reachable(..) == reachable_bounded(.., None).depths.into_keys().collect()

// src/relation_graph.rs
pub(crate) struct TransitiveGroup { pub label: RelationLabel, pub targets: Vec<String>, pub truncated: bool }
pub(crate) struct TransitiveView {
    pub id: String,
    pub max_depth: Option<usize>,        // None = unbounded
    pub truncated: bool,                 // view-level OR across emitted groups (C4)
    pub inbound: Option<Vec<TransitiveGroup>>,   // Some iff direction includes inbound (emitted FIRST)
    pub outbound: Option<Vec<TransitiveGroup>>,  // Some iff direction includes outbound
}
pub(crate) fn transitive_from(
    scanned: &[ScannedEntity], root: &Path, id: &str,
    dir: Dir,                            // Inbound | Outbound | Both (the same enum the CLI parses — F1)
    labels: Option<&[RelationLabel]>,    // None = all overlay-backed
    max_depth: Option<usize>,
) -> anyhow::Result<TransitiveView>;
```

`targets` sorted canonical-id ascending (REQ-077 determinism). `truncated` on a
group is the OR of the cap-hit across that label's walk.

**Output contract (C4 — pinned, golden-tested).** Mirrors `inspect_value`'s
`"kind"`-discriminated envelope (`relation_graph.rs:872`):

- **Section order (table + JSON):** `inbound` **before** `outbound` (blast-radius
  first — the impact framing); within a direction, groups sorted by label `name()`
  ascending. A direction not requested is **omitted** (not empty) — `--direction
  inbound` emits no `outbound` key/section.
- **Within a group:** `targets` id-ascending; empty group renders `(none)` (table)
  but is still emitted in JSON as `"targets": []`.
- **JSON envelope:**
  ```json
  { "kind": "inspect-transitive", "id": "ADR-005", "max_depth": 5, "truncated": false,
    "inbound":  [ { "label": "governed_by", "truncated": false, "targets": ["SL-012","SL-047"] } ],
    "outbound": [ ] }
  ```
  `max_depth` is `null` when unbounded. View-level `"truncated"` = OR across all
  emitted groups (the table's "… some chains truncated" line reads the same bit).
  No `role` key — transitive `references` is role-collapsed (F3). No `danglers`
  (no-overlay labels are out by construction). No `actionability` (relation-only).

### CLI flag shapes (clap)

```rust
#[arg(long)] transitive: bool,
#[arg(long, value_enum, default_value_t = Dir::Both, requires = "transitive")] direction: Dir,
#[arg(long = "labels", alias = "label", value_delimiter = ',', requires = "transitive")] labels: Vec<String>,
#[arg(long, requires = "transitive")] max_depth: Option<String>, // absent→5; "0"|"all"→unbounded; "N"→N
```

`Dir`: `Inbound` (`alias = "up"`), `Outbound` (`alias = "down"`), `Both`. `labels`
validated via `RelationLabel::from_name` **and the table-derived overlay-backed
predicate** (F4/C2) — reuse `OverlayMap::overlay_for` (= `RELATION_RULES` `target
!= Unvalidated`), never a hardcoded list. An unknown name *or* a no-overlay name
(**`contextualizes`/`drift`/`decision_ref`**) → clean error listing the valid
(overlay-backed) set; the two cases may share one "not transitively walkable"
message. `requires = "transitive"` makes the modifier flags
error if supplied without `--transitive`; because `--direction`'s `default_value_t`
is applied as a *default* (not "present on the command line"), bare `inspect <ID>`
does not trip `requires` — covered by a VT (F5-adjacent).

## 6. Verification

**Behaviour-preservation (gate):** existing `inspect`, `blockers`, and cordage
suites green **unchanged** — `reachable` re-expressed over `reachable_bounded`
must be byte-identical; bare `inspect <ID>` output unchanged (VT).

**cordage unit (`reachable_bounded`):**
- VT — depth values are min-hop distance (diamond: node reached by a 2-hop and a
  3-hop path records depth 2).
- VT — `max_depth: Some(k)` excludes nodes beyond k; `truncated` true ⟺ a node at
  depth k had an unvisited successor; false when the closure ends exactly at k.
- VT — `None` == legacy `reachable` (same set) on a fixture.
- VT — cycle-safe over a `Reject`-degraded view (visited-set bound); foreign
  overlay / foreign node → empty `Reach`.

**relation_graph unit (`transitive_from`):**
- VT — inbound vs outbound differ correctly on a directed fixture
  (`SL→governed_by→ADR`: `inspect ADR --inbound` reaches SL; `--outbound` empty).
- VT — per-label sectioning; `labels` narrowing to a subset; unknown/no-overlay
  label rejected.
- VT — depth cap truncates + sets the group/view flag; `all` is unbounded.
- VT — existence gate: never-minted id → error (reuse `require_minted`).

- VT (F5) — `truncated` invariant: a neighbour suppressed at the cap is unvisited
  ⟹ genuinely deeper than the cap (BFS visits shallower first), so `truncated`
  never false-positives a node reachable within depth via another path.

**Command-layer / e2e / golden:**
- VT — `inspect --transitive` table + `--json` goldens; each direction; `--labels`
  filter; `--max-depth N` and `all`; truncation line present/absent.
- VT (F2) — `inspect mem.<key> --transitive` (and `mem_<uid>`) → clean error
  naming `retrieve --expand`, gated before the memory early-return.
- VT (F3) — transitive `references` from a slice with mixed-role outbound edges
  renders ONE `references:` section (roles collapsed), distinct from 1-hop inspect.
- VT (F4/C2) — `--labels contextualizes`, `--labels drift`, `--labels bogus` → "not
  transitively walkable" error listing overlay-backed labels; and `contextualizes`
  is absent from the default (no-`--labels`) section set. The overlay-backed set is
  asserted table-derived (no hardcoded list), guarding against drift.
- VT (C4) — JSON envelope is `"kind": "inspect-transitive"`, `inbound` before
  `outbound`, view-level `truncated`, `max_depth: null` when unbounded; a
  non-requested direction key is absent (golden).
- VT (clap) — bare `inspect <ID>` (no `--transitive`) does not trip `requires` and
  is byte-unchanged; `--direction inbound` without `--transitive` errors.

## 7. Invariants & boundaries

- Start node excluded from its own closure (cordage strictness, unchanged).
- `depths` is min-hop (BFS first-visit wins); `truncated` is the only signal that
  the set is partial — an empty section is "no such reachable edges," never a cap.
- Cycles/self-loops terminate via cordage's visited set; each node emitted once.
- `--direction inbound` ⇒ `outbound: None` (section omitted), and vice-versa.
- Unbounded (`all`) on a hub may be large — accepted ("suck it and see," D5); the
  default 5 keeps the awareness view bounded.

## 8. Scope reconciliation (vs `slice-138.md`)

The original scope is **amended** (done after this doc locks):
- **Non-goal "No inbound transitive walk" → removed.** Inbound is the primary
  (blast-radius) direction (D2/D3).
- **`governed_by` example corrected.** Old: "`inspect SL-047 --transitive --label
  governed_by` → everything it governs" — outbound `governed_by` from SL-047
  reaches its *governor* ADR-010, not "what it governs." The blast-radius example
  is `inspect ADR-010 --transitive --direction inbound`.
- **Terrain "Reuse `reachable` — no changes" → corrected.** The depth cap +
  truncation require the new `reachable_bounded` (D6); `reachable`'s public
  behaviour is preserved.
- `--label` → `--labels` (multi, D4).

## 8a. Adversarial review log (2026-06-26, self-pass)

| # | Finding | Resolution |
|---|---------|------------|
| F1 | `transitive_from` used an undefined `Directions` type | reuse the `Dir` enum (§5) |
| F2 | memory refs + `--transitive` undefined | reject before the memory early-return, point at `retrieve --expand` (§2 limitations, VT) |
| F3 | transitive `references` can't split roles (label-keyed overlay, R5) | documented as one collapsed section; per-role transitive out of scope (§2, VT) |
| F4 | `--labels` must reject *no-overlay* names, not just unknowns | validate via `from_name` + overlay-backed predicate (§5, VT) |
| F5 | `truncated` correctness rests on a BFS-ordering argument | named tested invariant (§6) |
| F6 | `--max-depth 0 == unbounded` is a footgun | accepted (your call); `all` is the primary spelling, `0` documented alias |

### External inquisition (codex / GPT-5.5, 2026-06-26)

| # | Sev | Finding | Resolution |
|---|-----|---------|------------|
| C1 | **blocker** | Proposed `reachable_bounded(&OutIndex, &InIndex, …)` is uncallable — those indices are crate-private; `relation_graph` holds only a `Graph`, whose public surface is `Graph::reachable`. | New **public `Graph::reachable_bounded` method**; private `query::reachable_bounded` does the work; `Graph::reachable` re-expressed over it (§5). |
| C2 | **major** | No-overlay set is wrong — `contextualizes` is also `TargetSpec::Unvalidated`; the real set is `{contextualizes, drift, decision_ref}` (pinned by `relation_graph.rs:~1690`). Poisons `--labels` validation + the default label set. | Corrected throughout (§2/§5); predicate is table-derived (`overlay_for`), not a hardcoded list; VT guards drift. |
| C3 | minor | "single locus" overstated — `walk_bfs` is shared by `reachable`+`spine_path` only; `predecessor_cone` deliberately doesn't use it. | Claim narrowed (§3). |
| C4 | minor | Transitive output contract under-specified (section order, `kind`, JSON envelope, where view-level truncation lives). | Pinned + golden-tested (§5 output contract). |

Verdict was "not safe to plan against yet; must change first (C1, C2)." All four
integrated above; re-verified C1/C2 against the code (`lib.rs:807`, `relation.rs:434`,
`relation_graph.rs:163/1690`).

**Confirm pass (codex, same thread):** C1–C4 all **PASS** (re-checked against code);
clap `requires`/`default_value_t` claim verified sound (defaulted args aren't
"explicit", so bare inspect doesn't trip `requires`). One new **minor (C5)**: §5
file-table said lib.rs would "re-export" — wrong after the Graph-method reshape and
against cordage's re-export-free posture (`lib.rs:9`). Corrected. **Final verdict:
safe to plan against.**

## 9. design-target selectors

```
crates/cordage/src/query.rs
crates/cordage/src/lib.rs
src/relation_graph.rs
src/commands/inspect.rs
src/commands/cli.rs
```
