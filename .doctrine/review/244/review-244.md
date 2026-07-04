# Review RV-244 — reconciliation of SL-196

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode** — conformance (post-implementation, dispatched slice). **Self-audit**:
raiser and responder are the same agent (`--as` role assertion, ADR-007).

**Surface reviewed** — the candidate interaction branch, not raw evidence
(dispatched-slice rule). `dispatch candidate create` published
`candidate/196/review-001` (tip `054826cc`, base `main` + impl bundle
`review/196` @ `54f4fce`), worktree at
`.doctrine/state/dispatch/candidate/cand-196-review-001`. Gate/tests/verify-vt
run against that tree. Evidence refs (`review/196`, `phase/196-01..04`) are
immutable (R2). Ledger driven from the primary tree (`edge`).

**Invariants held to** (design SL-196 §5.5):
- INV-1 descriptor excluded from edge identity `(label, role, target)`.
- INV-2/-10 `descriptor_bearing` true on exactly `references:concerns`, disjoint
  from `degree_bearing`.
- INV-3 row serializes `descriptor` iff present (diff stability).
- INV-4 existing relation behaviour unchanged absent a descriptor
  (behaviour-preservation gate).
- INV-5 placement legality is write-path only; `read_block` permissive (degree
  parity, OQ-6).
- F-D: `CatalogEdge.descriptor` carries the serde omission guard (/api/graph
  contract byte-identical).

**Lines of attack:**
1. **Conformance algebra** — 3 undeclared paths: are they scope creep or
   compile-forced coupled-caller fan-out of the declared signature/struct
   changes (design R1)?
2. **`with_degree` → `with_descriptor`** — the P02 constructor swap removed the
   old constructor; is the removal dead-code DRY (behaviour-preserving) or a
   silent narrowing beyond design?
3. **Gate + VT + regression** — does the integrated bundle (union of 4 phases,
   which no single phase proved) compile clean, pass clippy, all suites, and the
   13 VT mandates?
4. **Design divergence** — does any observed behaviour contradict design §5, or
   are the design's forward-references (ISS-211, SL-197 OQ-5) still sound?

## Synthesis

**Closure story.** SL-196 lands the per-edge `descriptor` facet exactly as
designed — a `Degree`-seam re-skin: one `descriptor_bearing` rule column pinned
to `references:concerns`, one `--descriptor` write param threaded
validate_link → append_edge → row builder, the serde on-disk cell, the
`read_block` deserialize path (external F-C's late-caught site), `CatalogEdge`
hydration with the F-D omission guard, outbound-only render, and the search
lex-index source-join. The design was hardened by an integrated external
inquisition (F-A..F-D) *before* implementation, so the build had no residual
design defects to reconcile — the two audit findings are both mechanical, both
`aligned`.

**Evidence (candidate bundle `054826cc`).** Compiles clean; `cargo clippy
--bin doctrine` zero warnings; `cargo test --bin doctrine` 3080 passed / 0
failed; `slice verify-vt SL-196` 13/13 PASS across all four phases; per-phase S1
regression diffs clean at drive time. Conformance: 0 undelivered, 6 conformant,
3 undeclared — all three confirmed as compile-forced coupled-caller fan-out
(F-1), not scope creep.

**Findings.** F-1 (undeclared paths) and F-2 (`with_degree` removal) both
`aligned` — the observations are correct and the implementation is right as-is.
No `fix-now`, no `tolerated`, no blocker.

**Standing risks (carried, not defects of this slice):**
- **SL-197 / OQ-5** — the driver (`link CPT-… references … --role concerns
  --descriptor …`) needs `CPT` added to the *hand-enumerated* `references:concerns`
  sources array. SL-197's premise that CPT auto-inherits via `RECORD` is false;
  SL-197 (currently `design` stage) must add CPT explicitly. Out of scope here;
  SL-196 is deliberately source-set-agnostic. Verified SL-197 exists.
- **ISS-211** — the pre-existing `contextualizes` write/read-drop mismatch that
  motivated excluding `contextualizes` from `descriptor_bearing`. Verified open;
  not this slice's fix.

Neither touches SL-196's correctness; both are already captured durably
(ISS-211 backlog item, SL-197 design + OQ-5). No new backlog item warranted.

## Reconciliation Brief

Both findings dispositioned `aligned`; the implementation matches design and
governance with zero drift. **No reconciliation writes are required.**

### Per-slice (direct edit)
- _None._ design.md §5 matches the implemented behaviour; the §2 Current-State
  mention of `with_degree` is a descriptive snapshot, not maintained truth (F-2).

### Governance/spec (REV)
- _None._ No ADR, standard, policy, or spec contradicted. ADR-004 (outbound-only,
  no inbound descriptor), ADR-010 (Tier::One seam), ADR-016 (no new label/role)
  all upheld by construction.

`/reconcile` has no spec/governance edits to apply — its role here is to confirm
the clean outcome and advance the lifecycle.

## Reconciliation Outcome

Brief empty — both findings (F-1, F-2) dispositioned `aligned` and verified
terminal, no writes needed. No per-slice edits, no REVs. Implementation matches
design and governance (ADR-004/010/016 upheld by construction); the two standing
cross-slice risks (SL-197 OQ-5, ISS-211) are already captured durably and are
not SL-196 defects. Reconcile pass complete — handoff to /close.
