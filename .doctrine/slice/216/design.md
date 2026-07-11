# SL-216 design — per-component gauge scope

Refactors SL-213's tier-3 projection gauge (design §3 P8) from global corpus-H
to per-component scope. Originates from IMP-279 (the reconsidered refactor of
RV-266 F-3's prototype-validated global-H). Two decisions adjudicated with the
user 2026-07-11; both locked.

## §1 Decisions

### D1 — Full per-component regime (locked)

Each weakly-connected component of the post-quarantine merged-class graph picks
its own regime by **its own** anchors:

- component has ≥1 anchor → anchored pipeline (P3–P6; P7 for nodes with no
  directed path to any anchor — weak connectivity is not directed reachability,
  so P7 survives inside anchored components);
- component has no anchor → P8 gauge spread with **component** `H`.

Rejected variants:

- *Per-component `H`, global trigger* — fixes only the within-pure-gauge
  coupling (leak 1). Leaves leak 2: the corpus's **first anchor** landing in
  component X flips disjoint island Y from P8 spread to P7+P5, moving Y's
  values (2-chain: 1.333/0.667 → 1.25/1.0). Strict P12 requires the *trigger*
  per-component, not just the normaliser.
- *Per-component + floor clamp ≥ DEFAULT in mixed corpora* — preserves
  neutral-loss but breaks P8's "centred" property, splits pure/mixed
  semantics, adds a regime-detection wart.

Behavioural change accepted (the adjudication IMP-279 required): in a mixed
corpus an anchor-free island's comparison loser lands **below** DEFAULT
(0.667 < 1.0) where P7+P5 previously kept every islander ≥ DEFAULT. User
rationale, recorded: *judged-and-lost should rank below unjudged* — this is an
improvement, not a cost. Consistent with the validated pure-gauge regime.

### D2 — Contract home: direct amendment to SL-213 design.md (locked)

SL-213 design.md §3 is amended in place, each edit tagged `[amended by
SL-216]`:

- **P1** — drop the "one deliberate exception" clause; component scope is
  universal.
- **P8** — `H` = **component** max height; the gauge-spread branch fires per
  anchor-free component (not only when the corpus has no anchors anywhere).
- **P12** — unscoped: universal locality, both regimes, including regime
  membership (an anchor delta in X never moves disjoint Y).

Module doc note (`src/comparison/project.rs` header, lines 25–37) rewritten:
no longer "follows the prototype / reported for orchestrator adjudication";
states component scope as adjudicated (SL-216, IMP-279, RV-266 F-3 follow-on).

Rejected: leaving SL-213 design as historical record with SL-216 as authority
(splits the contract across a slice chain readers must chase); REV routing
(design docs are per-slice artefacts, not governance kind — reconcile doctrine
gives them direct edit).

Deferred → IDE-037: extracting the P1–P15 projection contract to an evergreen
tech spec once the comparison layer stabilises (post-RFC-019 Phase C).

## §2 Mechanism

All changes inside `src/comparison/project.rs`; no signature changes above
`place()`; `ProjectionCfg`, `ValueProvenance`, wire/explain surfaces untouched
structurally.

1. **New pure helper**

   ```rust
   /// Weakly-connected components of the merged-class graph (undirected
   /// reachability over `out ∪ inn`), ordered by minimum member id.
   fn components(
       nodes: &BTreeSet<ClassId>,
       out: &Adj,
       inn: &Adj,
   ) -> Vec<BTreeSet<ClassId>>
   ```

   BFS over the union adjacency; component order by min member id
   (determinism; the result map is a `BTreeMap`, so ordering is
   internal-only). `BTreeSet` throughout — `HashSet` is clippy-disallowed
   (determinism).

2. **`place()` restructured** — outer loop over `components(...)`; per
   component:
   - `component_anchors` = anchors filtered to members;
   - empty → the existing P8 block computed over the member set
     (`topo_order`/`heights` restricted to members; `H` = component max);
   - non-empty → the existing anchored loop restricted to members.

3. **Preservation argument** — the anchored machinery (`topo_order`,
   `longest_up`, `depth_below_ceiling`, `successor_max`) is pure over
   adjacency, and no cross-component edges exist by construction; running it
   per component is bitwise-identical to running it globally for anchored
   components. The only behaviour that moves is anchor-free components in a
   multi-component corpus — exactly the declared change surface.

Single-component corpora are unaffected in all regimes; live-data migration
risk low (IMP-279 compatibility analysis: of 22 validated scenarios only S2
has ≥2 disjoint components).

## §3 Verification

Evidence that must change (pinned re-pins, part of the declared change):

- **`s2_partial_order_gauge`** — pendant f/g: 0.8/0.4 → **1.3333/0.6667**;
  comment rewritten (no longer pins the global-gauge coupling — pins the
  component scale).
- **`s8_incremental_locality`** — beyond IMP-279's stated surface: its base
  case is a mixed corpus with anchor-free island `{z>w}`. z 1.25 *Projected* /
  w 1.0 *Gauge* → z **1.3333** / w **0.6667**, both **Gauge**. The provenance
  flip is part of the pinned change.

Evidence that must be added:

- **`p12` unscoped + strengthened** — three-way:
  1. perturb anchored X → gauge island Y frozen;
  2. perturb gauge island Y → anchored X frozen;
  3. add the corpus's **first anchor** to X → disjoint gauge Y frozen
     (kills leak 2 explicitly; RED against the shipped global trigger).
- **New golden `mixed_corpus_island`** — anchored diamond + free pendant;
  pendant takes the centred component spread, Gauge provenance.
- **Golden audit sweep** — every existing scenario with ≥2 components checked
  for silent re-pin; s2 and s8 are the known movers, sweep confirms no others.

Evidence that must NOT change (behaviour-preservation gate):

- single-component goldens (s1, s3–s7, y1, y2) byte-identical;
- p10 (order consistency / no-NaN), p11 (determinism under permuted input),
  p14 (scoped affine equivariance) unchanged;
- priority suites unchanged (`build_from_with_cfg` integration untouched).

`just gate` clean; TDD red/green/refactor — new p12 case 3 and
`mixed_corpus_island` written RED first.

## §4 Open questions

None. OQ-1 (contract home) resolved as D2; OQ-6 (ratio elicitation) out of
scope, unaffected.
