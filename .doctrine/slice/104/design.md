# SL-104 — Estimate hardening: NFR verification, confidence legitimization, polish

<!-- Reference forms: entity ids padded (SPEC-020, ADR-001, REQ-275, IMP-112);
     doc-local refs bare (D1, OQ-1, C-REQ). See .doctrine/glossary.md. -->

Design canon for SL-104. Descends SPEC-020 (estimation facet), the hardening pass
after SL-101 (model/parse/validate/unit), SL-102 (display functions), SL-103
(catalog/graph exposure).

## 1. Current → target

**Current state on `main`** (the stale "facets unwired" memory is obsolete —
SL-102/103 integrated them):

- `src/estimate.rs` / `src/value.rs` — models, custom `Deserialize`, normalization
  to finite `f64`, validation matrix, project-wide unit resolution. Pure leaf tier.
- `src/dtoml.rs` — wires `[estimation]` + `[value]` config tables.
- `src/catalog/scan.rs` — reads both facets generically off any entity toml
  (`table.get("estimate")`), with per-facet malformed-isolation diagnostics.
- `src/catalog/graph.rs` — projects per-node `estimate`/`value` + a top-level
  `units{estimation,value}` block; policy-free contract.
- `src/slice.rs` — `SliceDoc` carries `Option<EstimateFacet>`/`Option<ValueFacet>`;
  surfaced via `slice show --json`.
- **Unspec'd residue:** `EstimationConfig.{lower,upper}_confidence`,
  `resolve_confidence`, `DEFAULT_*_CONFIDENCE` — dead code, no governing REQ,
  no mention in SPEC-020/PRD-014.
- **NF-001 holds by accident** — zero facet refs in the gating modules, but nothing
  enforces it.
- **Display unwired** — `estimate/display.rs` renderers are `expect(dead_code)`; the
  human `show` path renders no facet.

**Target:** confidence is spec-legitimate; NF-001 is structurally pinned; ≥2 real
entities carry validated facets; edge cases covered; stale `expect` reasons fixed.
No display wiring (deferred — IMP-112).

## 2. Decisions

- **D1 — Confidence is legitimized, not deleted (user ruling).** The percentile
  model is intended: an estimate's `lower`/`upper` are read as a project-wide
  confidence band (`lower` at P-low, `upper` at P-high), defaults `0.1`/`0.9` from
  `doctrine.toml [estimation]`, configurable. Its sole intent is **display framing**
  of the bounds; **no gating, aggregation, or normalization effect in v1.** A REQ
  homes it; SPEC-020 is amended. The existing `resolve_confidence` code becomes
  spec-traceable.

- **D2 — Governance routes through a Revision folded into SL-104's reconcile**
  (ADR-013), mirroring **REV-002** (which amended this same spec). The REQ + spec
  amendment land at reconcile, not as a standalone pre-slice REV. SL-104 designs and
  hardens against the decided model now.

- **D3 — Display wiring is out of scope (→ IMP-112).** Confidence's display-framing
  intent and the estimate display renderers both require wiring the human `show`
  path — a feature, barred by this slice's non-goal. Deferred together to IMP-112.
  SL-104 only legitimizes the spec and corrects the misleading dead-code reasons.

- **D4 — NF-001 proof is structural and dependency-free.** Two tiers, no `syn`, no
  coupling to SL-112 (proposed, unstarted — its syn-based dependency-fitness gate is
  the *future* consolidation home for this rule, not a prerequisite):
  - **Tier 1 (dispatch / execute / audit):** a source-scanning integration test
    asserts the pure gating modules carry **zero** facet symbols.
  - **Tier 2 (close):** a compile-time exhaustive-destructure guard on `Gate` —
    if `Gate` ever grows an `estimate`/`value` field the test fails to compile.

- **D5 — NF-002 / NF-003 are confirmed, not rebuilt.** Both are already covered by
  SL-101/103 tests; SL-104 reinforces with real-data dogfood (NF-002) and
  cross-references existing forward-compat tests (NF-003), adding only a thin
  value pure-boundary assertion where coverage is weak.

## 3. Confidence — spec design (foundational)

**The REQ** (functional; id allocated at reconcile, doc-local **C-REQ**):

> **Frame estimate bounds with a project-wide confidence band.** An estimate's
> `lower`/`upper` are interpreted as a project-wide confidence band — `lower` at
> percentile P-low, `upper` at P-high. The band resolves from
> `doctrine.toml [estimation].lower_confidence`/`upper_confidence`, defaulting
> `0.1`/`0.9`; each bound finite, in `[0,1]`, `low < high`. The band frames the
> bounds for display when authoring and interpreting; it has **no gating,
> aggregation, or normalization effect in v1**, and no entity-local confidence field.
>
> *Acceptance:* (1) bounds carry a project-wide percentile reading resolved from
> config; (2) defaults `0.1`/`0.9`, configurable, rejected unless finite ∧ in `[0,1]`
> ∧ `low < high`; (3) display-framing only — no predicate / aggregation / validation
> depends on it; (4) no entity-local field.

**SPEC-020 amendment surface (the REV, at reconcile):** one new responsibility
bullet (confidence-band resolution); one `### Confidence band resolution`
subsection under "Project-wide unit resolution"; the REQ added to `members.toml`.
Mirrors REV-002's mechanics.

**Code touch this slice:** no behavior change. Only the stale
`expect(dead_code, reason=…)` strings on `resolve_confidence`,
`DEFAULT_*_CONFIDENCE`, and `mod display` are corrected to cite C-REQ + IMP-112.
The dead-code tripwire stays armed (self-clears when IMP-112 consumes it).

## 4. NF-001 — structural non-blocking tripwire

**Placement:** `tests/e2e_estimate_non_blocking.rs` (Tier 1) — matches the existing
source-scanning integration-test precedent (`e2e_relation_migration_storage.rs`,
`e2e_skills_dispatch_shrinkage.rs` use `CARGO_MANIFEST_DIR`); plus a unit guard in
`src/slice.rs` (Tier 2, needs `pub(crate)` `Gate`).

**Tier 1 — denylist scan.** Assert the pure gating modules — `dispatch.rs`,
`dispatch_config.rs`, `lifecycle.rs`, `reconcile.rs`, `review.rs`, `governance.rs`
— contain **zero** facet symbols. Symbol set is precise (not bare words, which
collide with `toml::Value`):

```
EstimateFacet · ValueFacet · EstimationConfig · ValueConfig · resolve_confidence
crate::estimate · crate::value · estimate:: · value::
```

`resolve_confidence` is in the set → this also discharges the confidence
"no-consumer" negative test (D1's display-only guarantee). A future predicate that
reads a facet must name one of these in a gating module → the test fails.

**Tier 2 — `Gate` type confinement.** The closure gate's input is `Gate` (+ lifecycle
status + requirement set); `Gate` carries no facet field. A unit test in `slice.rs`:

```rust
// NF-001: the closure gate cannot branch on estimate presence — the facet is
// structurally absent from its input type. Exhaustive (no `..`): a future
// estimate/value field on Gate breaks this compile.
let Gate { /* …current fields, no estimate/value… */ } = Gate::default();
```

If `Gate` grows an `estimate`/`value` field, this fails to compile — the strongest
possible structural proof, zero deps.

**Sharp edge (CHR-014):** source-scan tests bake `CARGO_MANIFEST_DIR`; a shared
target dir can leak a stale worktree path. The scan locates `src/` via the
compile-time manifest dir and tolerates the standard `cargo test` build — kept
tree-relative, noted inline.

**Future consolidation (not this slice):** when SL-112's syn-based dependency-fitness
gate lands, NF-001's rule is a candidate to re-express there. SL-104 does not grow a
bespoke AST guard that SL-112 would obsolete (no parallel implementation).

## 5. Dogfood (≥2 kinds)

| Entity | Facets | Purpose |
|---|---|---|
| **SL-104** (this slice) | `[estimate]` + `[value]` | non-blocking proven on a *live* slice we drive — it still gates/closes |
| **SL-101** (done) | `[estimate]` + `[value]` | the facet's origin slice, stable |
| **ADR-013** | `[estimate]` only | second-kind **mechanical** kind-agnosticism proof (commented as such, not a semantic estimate) |

All valid facets. **Verify each:** `slice show`/`adr show` parse clean; the entity
still `list`s; `catalog`/graph projects the facet + `units` block; `2`-authored
bounds normalize to `2.0` and round-trip stable.

## 6. Edge-case tests

- **Large finite bounds** — `lower=1e9, upper=5e9` parse/validate/round-trip.
- **int ≡ float normalization** — `2` and `2.0` yield identical `EstimateFacet`;
  serialized form is the normalized one (pins FR-004 round-trip).
- **Value asymmetry** — `[value] value=-5` is **valid** (finite, no range constraint),
  unlike estimate's `lower>=0` — pins the deliberate estimate/value divergence.
- (zero-width `lower==upper` already `e4`; display width-overflow is IMP-112's.)

## 7. Residual cleanup

- Correct the stale `expect(dead_code, reason=…)` strings (`mod display`,
  `DEFAULT_*_CONFIDENCE`, `resolve_confidence`) — re-cite C-REQ + IMP-112 as the
  real future consumer. Tripwire stays armed.

## 8. Verification alignment

| Surface | Evidence |
|---|---|
| NF-001 (REQ-275) | `tests/e2e_estimate_non_blocking.rs` (Tier 1) + `Gate` destructure guard (Tier 2) green; structural argument in `audit.md` |
| NF-002 (REQ-276) | dogfood ≥2 kinds + value pure-boundary assertion; existing graph VT-3 cross-ref |
| NF-003 (REQ-277) | existing forward-compat tests cross-ref (no new mechanism) |
| C-REQ (confidence) | negative test (no `resolve_confidence` consumer) folded into Tier 1; SPEC-020 amendment + REQ at reconcile (REV, D2) |
| Dogfood | `show`/`list`/`catalog` green on real faceted entities; diff review |
| Edges | new unit tests in `estimate.rs` / `value.rs` |

**No production-code behavior change.** New artifacts: one integration test, a few
unit tests, corrected `expect` strings, authored facet tables in ≥2 entity toml.
At reconcile: the confidence REQ + SPEC-020 amendment (via REV).

## 9. ADR / governance alignment

- **ADR-001** (layering) — all new test code; pure leaf tier untouched; no
  clock/disk/rng in the facet layer (source-scan test touches disk, legitimately, as
  a test).
- **ADR-009** (lifecycle) — SL-101 is `done`/terminal and not reopened; SL-104 is the
  sanctioned hardening vehicle.
- **ADR-013** (revision routes governance) — the confidence REQ + spec amendment land
  via a REV folded into reconcile (D2), precedent REV-002.
- **Spec storage rule** — REQ structured in `members.toml`/`requirement` toml; spec
  prose in `spec-020.md`; no derived data in prose.

## 10. Open questions

- **OQ-1 — confidence REQ classification.** Drafted as *functional* (defines
  resolution + framing behavior). If the spec author prefers it as a *quality*
  attribute of the estimate facet, reclassify at reconcile. Low stakes.
- **OQ-2 — dogfood semantic honesty.** SL-104/SL-101 estimates are real-ish; the
  ADR-013 estimate is explicitly mechanical (kind proof). If a reviewer objects to a
  non-semantic estimate on an ADR, swap the second kind for another slice and prove
  kind-agnosticism via the existing graph VT-3 alone. Resolved-by-default: keep ADR,
  comment it.
