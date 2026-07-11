# Review RV-268 — reconciliation of SL-216

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance-mode self-audit of SL-216 (per-component gauge scope), solo
in-tree execution on edge (PHASE-01 89066bd4, PHASE-02 96633ed3). Surface
reviewed: the primary working tree at those commits — no dispatch candidate
branch exists for this slice.

Lines of attack:

1. **Path conformance** — `slice conformance SL-216` algebra: every
   undeclared/undelivered cell dispositioned, registry vs design-target
   selectors vs actual diff.
2. **Design §2 mechanism fidelity** — components() + per-component regime
   selection matches the locked D1; no signature changes above place();
   ProjectionCfg/ValueProvenance/wire surfaces structurally untouched.
3. **Verification contract** — every PHASE-01 VT keyword present and
   assert-backed; VA-1 diff audit (movers = declared s2/s8 + new tests only);
   e2e_compare_inference green UNEDITED in PHASE-01, comment-only in PHASE-02.
4. **Preservation gate** — single-component goldens, p10/p11/p14, priority
   suites, 4043-test full run green unchanged.
5. **D2 amendment sweep completeness** — SL-213 design §3 amendments tagged;
   no stale global-scope claim survives the token grep + P7/P8/P12 comment
   audit; declared surface deltas (explain render flip, disconnect membership
   growth) tested and documented.
6. **Gate integrity** — `doctrine check gate` outcome, incl. the known jail
   lint-js breakage vs EX-5.

## Synthesis

SL-216 delivers what its locked design promised, in two phases, on edge, with
a clean TDD receipt.

**Mechanism (D1) is faithful.** `components()` is a pure BFS over the merged
out ∪ inn adjacency (BTree collections, min-member order for free via
`pop_first`); `place()` became an outer component loop delegating to
`place_component()` — the old body verbatim, invoked per component with a
`component_anchors` filter. No signature changes above `place()`;
`ProjectionCfg`, `ValueProvenance`, and the wire/explain surfaces are
structurally untouched. The preservation argument (anchored machinery pure
over adjacency, no cross-component edges) held in practice: the entire
preservation set — single-component goldens, p10/p11/p14, priority suites,
e2e_compare_inference — stayed green unchanged through both phases (4043
tests, zero failures).

**Evidence is contractual, not incidental.** Six cases were RED against the
shipped global trigger before `components()` existed, each failing at exactly
the pinned shipped value (s2 f 1.25→1.3333, singleton 0.6667→1.0, s8
provenance flip, p12 first-anchor "P moved", disconnect ["w"]→["w","z"]).
p12's anchored↔anchored witness was retained verbatim (RV-267 F-4); its two
cross-regime freeze cases were green pre-implementation, as predicted —
strengthening, not movers. Both declared user-visible deltas (explain render
flips island ladder nodes to Gauge; AnchorGaugeDisconnect membership grows
island-wide) are pinned by tests.

**The contract now tells the truth (D2).** SL-213 design §3 is amended in
place with four [amended by SL-216] tags (P1 exception retired, P8 component
H with the RV-266 F-3 reconciled note superseded, P12 unscoped); the module
header states component scope as adjudicated; findings and e2e narratives
speak component language. The sweep audit found one stale site beyond the
design's enumerated list (the module Method ¶'s "faithful port" claim) —
evidence the seed-not-boundary sweep rule earned its keep — and all grep
survivors are legitimate (historical amendment notes, BFS/preservation
comments).

**Standing risks / accepted tradeoffs.**
- Gate js-lint leg unverified in the jail (F-3, tolerated): pre-existing env
  breakage, ISS-222; host-side gate run owed before anyone leans on lint-js.
- Behavioural change consciously shipped: in a mixed corpus an anchor-free
  island's loser lands below default_value (judged-and-lost < unjudged) —
  adjudicated in design (IMP-279), pinned by mixed_corpus_island.
- Conformance registry gap (F-1, → reconcile): e2e file delivered but not
  selector-declared; mechanical fix at reconcile.

## Reconciliation Brief

### Per-slice (direct edit)
- **F-1** — selector registry: `doctrine slice selector add SL-216
  tests/e2e_compare_inference.rs` (design-target role) — the load-bearing
  change that turns the conformance "undeclared" cell green for the delivered
  PHASE-02 sweep site. Prose mirror: SL-216 design.md D2 known-sites list
  gains "the e2e ISS-050 narrative (tests/e2e_compare_inference.rs)" alongside
  the findings.rs and p12 sites.

### Governance/spec (REV)
- None. SL-213 design.md §3 was amended directly in PHASE-02 per locked D2
  (per-slice artefact, not governance kind); no ADR/policy/standard/spec is
  touched by this slice.

## Reconciliation Outcome

### Direct edits applied
- **Selector registry** (`slice-216.toml`): added
  `tests/e2e_compare_inference.rs` (intent `design-target`) — the load-bearing
  write. Conformance now reads the delivered PHASE-02 sweep site as
  `conformant` (was spurious `undeclared`). Drives RV-268 F-1.
- **design.md D2** (known-sites list): prose mirror gains the e2e ISS-050
  narrative (`tests/e2e_compare_inference.rs:458-462`) alongside the
  `findings.rs` and `p12` sites. Drives RV-268 F-1.

### REVs completed
- None. No governance/spec surface touched by this slice; SL-213 design.md §3
  was amended directly in PHASE-02 per locked D2 (per-slice artefact).

### Withdrawn / tolerated
- RV-268 F-2 (nit, aligned): remaining undeclared paths (case-notes.md,
  notes.md, slice-216.toml) are inherent lifecycle/process writes, not scope
  creep — no registry action owed.
- RV-268 F-3 (major, tolerated): gate `lint-js` leg fails on pre-existing jail
  env breakage (eslint `/usr/bin/env` shebang); ISS-222 minted. Rust legs
  clean. Host-side gate run owed before trusting lint-js — carried as tolerated
  disposition, not a reconcile write.

### Conformance after reconcile
4 conformant, 0 undelivered, 3 undeclared (all F-2 lifecycle/process).

Reconcile pass complete — handoff to /close.
