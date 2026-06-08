# SL-022 implementation notes

Durable implementation notes — durable findings/decisions taken during execution.
Runtime task progress lives under `.doctrine/state/` (gitignored); this is the
committed record.

## PHASE-02 — Registry edges, product set & FK / subject-kind checks

**Shape.** `Registry` gained `product_specs: BTreeSet`, `parents: Vec<ParentEdge>`,
`descents: Vec<DescentEdge>` (each edge carries `on_product`). Two new pure HARD
checks `descent_findings` / `parent_findings`, both scope-aware, both 3-way
(invalid-kind on product subject / clean / invalid-kind on wrong-kind target /
dangling). `dangling_interaction_targets` rewritten to split invalid-kind (product
target) from dangling. `validate` extends both new checks; signature unchanged.

**Behaviour change (disclosed, not a gate breach).** REQ-084 / PRD-012 §6: an
interaction target that is a product spec is now *invalid kind*, not *dangling*.
The SL-015 test `non_tech_interaction_target_is_flagged_tech_only` asserted the old
contract → rewritten as `product_interaction_target_is_invalid_kind_not_dangling`.
This is the ONE intended behaviour change in an otherwise behaviour-preserving
phase. Carry to audit.

**Charge I — new fallible parse + widened error surface.** `build_registry` now
parses each `spec-NNN.toml` (it parsed none before — only `members.toml` /
`interactions.toml`). Consequence: a malformed `spec-NNN.toml` that `validate`
never opened before now fails the build. Intended widening, proven by Layer C
`build_registry_surfaces_a_malformed_spec_toml`, NOT assumed from the hand-built
unit suites (they bypass this seam). Both arms parse + harvest both tech-only
fields so a product carrying one is seen and flagged, not dropped (codex F5b).

**DRY.** Added `canonicalize_spec_ref` (free fn in `spec.rs`) — the single
canonicalisation path for every outbound spec→spec ref; the interaction-target
block now reuses it too.

**Seam left for PHASE-03 (clean, additive).** PHASE-02's parse `Err` arm just
propagates `"Failed to parse"` via `?`. PHASE-03 inserts the `second_parent`
classification branch *before* the `?` and adds the `build_findings` carrier —
no PHASE-02 rework required.

**Existing-test edits (mechanical, non-behavioural).** Only the `clean()`
`Registry` literal gained `..Default::default()` (covers PHASE-02's three fields
AND PHASE-03's future `build_findings` in one edit). No existing assertion changed
value, except the deliberate REQ-084 test rewrite above.

**Lint.** `map(..).unwrap_or_else(..)` on a `Result` trips `clippy::map_unwrap_or`
(pedantic) — use `map_or_else(default, f)`.
