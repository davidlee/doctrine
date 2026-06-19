# SL-104 — Estimate hardening: NF-001 tripwire + confidence legitimization

<!-- Reference forms: entity ids padded (SPEC-020, ADR-001, REQ-275, IMP-112);
     doc-local refs bare (D1, OQ-1, C-REQ). See .doctrine/glossary.md. -->

Design canon for SL-104. Descends SPEC-020 (estimation facet), the hardening pass
after SL-101 (model/parse/validate/unit), SL-102 (display functions), SL-103
(catalog/graph exposure).

**Scope is deliberately narrow.** The slice was challenged as ceremony beyond its two
real deliverables; the cut-set (§12) was confirmed. SL-104 ships exactly: the NF-001
structural tripwire, confidence spec legitimization, and one untested-contract test —
nothing inert. No dogfood, no redundant edge tests, no external review.

## 1. Current → target

**Current state on `main`** (the stale "facets unwired" memory is obsolete —
SL-102/103 integrated them):

- `src/estimate.rs` / `src/value.rs` — models, custom `Deserialize`, normalization to
  finite `f64`, validation matrix, project-wide unit resolution. Pure leaf tier.
- `src/dtoml.rs` — wires `[estimation]` + `[value]` config tables.
- `src/catalog/scan.rs` — reads both facets generically off any entity toml, with
  per-facet malformed-isolation diagnostics.
- `src/catalog/graph.rs` — projects per-node `estimate`/`value` + top-level
  `units{estimation,value}`; policy-free contract.
- `src/slice.rs` — `SliceDoc` carries the facets; surfaced via `slice show --json`.
- **Unspec'd residue:** `EstimationConfig.{lower,upper}_confidence`,
  `resolve_confidence`, `DEFAULT_*_CONFIDENCE` — dead code, no governing REQ.
- **NF-001 holds by accident** — zero facet refs in gating modules, nothing enforces it.
- **Display unwired** — `estimate/display.rs` is `expect(dead_code)`; deferred → IMP-112.

**Target:** confidence is spec-legitimate; NF-001 is structurally pinned; the value
"no range validation" contract is tested; stale `expect` reasons fixed. No display
wiring, no dogfood.

## 2. Decisions

- **D1 — Confidence is legitimized, not deleted (user ruling).** The percentile model
  is intended: an estimate's `lower`/`upper` are read as a project-wide confidence
  band (`lower` at P-low, `upper` at P-high), defaults `0.1`/`0.9` from
  `doctrine.toml [estimation]`, configurable. Sole intent is **display framing** of
  the bounds; **no gating, aggregation, or normalization effect in v1.** A REQ homes
  it; SPEC-020 is amended. The existing `resolve_confidence` becomes spec-traceable.

- **D2 — Governance routes through a Revision folded into SL-104's reconcile**
  (ADR-013), mirroring **REV-002** (which amended this same spec). The REQ + spec
  amendment land at reconcile, not as a standalone pre-slice REV.

- **D3 — Display wiring is out of scope (→ IMP-112).** Confidence's display-framing
  intent and the estimate display renderers both require wiring the human `show` path
  — a feature, barred by this slice's non-goal. SL-104 only legitimizes the spec and
  corrects the misleading dead-code reasons.

- **D4 — NF-001 proof is structural and dependency-free.** Two tiers, no `syn`, no
  coupling to SL-112 (proposed; its syn-based fitness gate is the *future*
  consolidation home, not a prerequisite):
  - **Tier 1 (allowlist confinement):** a source-scanning integration test asserts
    facet symbols appear **only** in the known exposure files (+ tests).
  - **Tier 2 (close):** a compile-time exhaustive-destructure guard on `Gate`.

- **D5 — NF-002 / NF-003 are already covered; SL-104 adds nothing for them.** Both are
  green via SL-101/103 tests (graph VT-3 proves kind-agnosticism on a second kind;
  `e19`/`v7`/`custom_deserialize_unknown_keys` prove forward-compat). Cited at audit,
  not re-built. **Dogfood is cut** — authored facets on real entities re-prove NF-002
  and sit inert until a consumer (IMP-112 / Cordage) exists.

## 3. Confidence — spec design (foundational)

**The REQ** (functional; id allocated at reconcile, doc-local **C-REQ**):

> **Frame estimate bounds with a project-wide confidence band.** An estimate's
> `lower`/`upper` are interpreted as a project-wide confidence band — `lower` at
> percentile P-low, `upper` at P-high. The band resolves from
> `doctrine.toml [estimation].lower_confidence`/`upper_confidence`, defaulting
> `0.1`/`0.9`; each bound finite, in `[0,1]`, `low < high`. The band frames the bounds
> for display when authoring and interpreting; it has **no gating, aggregation, or
> normalization effect in v1**, and no entity-local confidence field.
>
> *Acceptance:* (1) bounds carry a project-wide percentile reading resolved from
> config; (2) defaults `0.1`/`0.9`, configurable, rejected unless finite ∧ in `[0,1]`
> ∧ `low < high`; (3) display-framing only — no predicate / aggregation / validation
> depends on it; (4) no entity-local field.

**Confidence is estimate-only** — the value facet is a single point magnitude with no
band; `[value]` config has no confidence fields.

**SPEC-020 amendment surface (the REV, at reconcile):** one new responsibility bullet
(confidence-band resolution); one `### Confidence band resolution` subsection under
"Project-wide unit resolution"; the REQ added to `members.toml`. Mirrors REV-002.

**Code touch this slice:** no behavior change. Only the stale
`expect(dead_code, reason=…)` strings on `resolve_confidence`, `DEFAULT_*_CONFIDENCE`,
and `mod display` are corrected. **REQ-id sequencing (F5):** C-REQ has no id until
reconcile, so the `expect` reasons cite IMP-112 + a descriptive phrase ("the
confidence requirement, SL-104 reconcile") now; reconcile rewrites them to the
concrete `REQ-NNN`. The dead-code tripwire stays armed (self-clears when IMP-112
consumes it).

## 4. NF-001 — structural non-blocking tripwire

**Placement:** `tests/e2e_estimate_non_blocking.rs` (Tier 1) — matches the existing
source-scanning integration-test precedent (`e2e_relation_migration_storage.rs`,
`e2e_skills_dispatch_shrinkage.rs` use `CARGO_MANIFEST_DIR`); plus a unit guard in
`src/slice.rs` (Tier 2, needs `pub(crate)` `Gate`).

**Tier 1 — allowlist confinement.** Scan all of `src/**/*.rs` for the facet symbol
set; assert every matching file is in the **allowlist** — the known exposure surface:

```
allowlist: estimate.rs · value.rs · estimate/display.rs · dtoml.rs
           catalog/scan.rs · catalog/graph.rs · catalog/hydrate.rs · slice.rs
symbols (precise — not bare words, which collide with toml::Value):
   EstimateFacet · ValueFacet · EstimationConfig · ValueConfig · resolve_confidence
   crate::estimate · crate::value · estimate:: · value::
```

Any *new* file naming a facet fails the test → forces a conscious decision: legitimate
exposure site, or a gating read that must not exist? `resolve_confidence` in the set
→ its absence from every non-allowlist file also discharges the confidence
"no-consumer" negative test (D1's display-only guarantee). The allowlist is robust
where a denylist is not: it needs no enumeration of gating modules.

**Tier 2 — `Gate` type confinement.** The closure gate's input is `Gate` (+ lifecycle
status + requirement set); `Gate` carries no facet field. A unit test in `slice.rs`:

```rust
// NF-001: the closure gate cannot branch on estimate presence — the facet is
// structurally absent from its input type. Exhaustive (no `..`): a future
// estimate/value field on Gate breaks this compile.
let Gate { /* …current fields, no estimate/value… */ } = Gate::default();
```

If `Gate` grows an `estimate`/`value` field, this fails to compile — the strongest
*type-level* proof, zero deps.

**Residual gap (documented, not hidden):** `slice.rs` is allowlisted (it carries
`SliceDoc`'s facet fields), so a hand-written close fn reading `SliceDoc.estimate`
directly would evade both tiers. Tier 2 covers the *typed* gate input; the
hand-written-bypass case is mitigated by review + the `audit.md` argument, not by the
test. This is the honest boundary of the structural proof.

**Sharp edge (CHR-014):** source-scan tests bake `CARGO_MANIFEST_DIR`; a shared target
dir can leak a stale worktree path. The scan locates `src/` via the compile-time
manifest dir and tolerates the standard `cargo test` build — kept tree-relative,
noted inline.

**Future consolidation (not this slice):** when SL-112's syn-based dependency-fitness
gate lands, NF-001's rule is a candidate to re-express there. SL-104 does not grow a
bespoke AST guard SL-112 would obsolete (no parallel implementation).

## 5. The one defensive test — value asymmetry

`[value] value = -5` is **valid** — the value facet has no range constraint, only a
finite check (FR-008), unlike estimate's `lower >= 0`. This deliberate asymmetry is
**currently untested** (`v5` only covers NaN-rejected). One unit test in `value.rs`
pins it: a negative finite magnitude parses to a valid `ValueFacet`.

(Zero-width `lower==upper` already `e4`; large-bounds and int≡float were cut as
redundant — `e2`/`e3` already exercise the integer/float normalization path, and a
large finite `f64` is the same code path as a small one.)

## 6. Residual cleanup

- Correct the stale `expect(dead_code, reason=…)` strings (`mod display`,
  `DEFAULT_*_CONFIDENCE`, `resolve_confidence`) — re-cite C-REQ + IMP-112 as the real
  future consumer. Tripwire stays armed.

## 7. Verification alignment

| Surface | Evidence |
|---|---|
| NF-001 (REQ-275) | `tests/e2e_estimate_non_blocking.rs` (Tier 1 allowlist) + `Gate` destructure guard (Tier 2) green; structural argument + F2 residual-gap noted in `audit.md` |
| C-REQ (confidence) | negative test (no `resolve_confidence` consumer) folded into Tier 1; SPEC-020 amendment + REQ at reconcile (REV, D2) |
| Value asymmetry (FR-008) | new `value.rs` unit test — negative finite magnitude accepted |
| NF-002 / NF-003 | **no new evidence** — cited green at audit (graph VT-3; `e19`/`v7`) |

**No production-code behavior change.** New artifacts: one integration test, one unit
test, corrected `expect` strings. At reconcile: the confidence REQ + SPEC-020
amendment (via REV).

## 8. ADR / governance alignment

- **ADR-001** (layering) — all new test code; pure leaf tier untouched; source-scan
  test touches disk legitimately (a test, not the facet layer).
- **ADR-009** (lifecycle) — SL-101 is `done`/terminal, not reopened; SL-104 is the
  sanctioned hardening vehicle.
- **ADR-013** (revision routes governance) — confidence REQ + spec amendment via a REV
  folded into reconcile (D2), precedent REV-002.
- **Spec storage rule** — REQ structured in `members.toml`/`requirement` toml; spec
  prose in `spec-020.md`; no derived data in prose.

## 9. Open questions

- **OQ-1 — confidence REQ classification.** Drafted *functional* (defines resolution +
  framing behavior). If the spec author prefers a *quality* attribute, reclassify at
  reconcile. Low stakes.

## 10. Internal adversarial pass (integrated, then narrowed)

Pre-review hostile findings, all resolved:

- **F1** — Tier-1 is **allowlist** confinement (§4), robust against a forgotten gating
  module.
- **F2** — Tier-2 residual gap named honestly (§4): a hand-written close fn in
  allowlisted `slice.rs` reading `SliceDoc.estimate` evades both tiers; mitigated by
  review + `audit.md`.
- **F3 / F4** — **superseded by the cut.** F3 (dogfood edit-preserving survival) and F4
  (value pure-boundary) both lived in the dogfood/edge scope that was cut (§11);
  dogfood is gone and serde round-trip is already proven by
  `slice_doc_round_trips_estimate_facet`.
- **F5** — `expect`-string REQ-id sequencing pinned (§3).
- **F6** — confidence is estimate-only (§3).

## 11. Scope cut (CONFIRMED)

Challenged as ceremony; cut-set confirmed by the user. **KEPT:** NF-001 tripwire
(Tier 1 + Tier 2), confidence legitimacy (REQ + spec amend + `expect` fix), one
value-asymmetry test. **CUT:** dogfood on real entities (re-proves NF-002, inert
until a consumer), large-bounds + int≡float tests (redundant), NF-002/NF-003
"confirmation" busywork (cite at audit), external review / inquisition (ceremony for a
1-test + spec-amend slice → straight to `/plan`). If dogfood is ever wanted, it's a
one-line backlog note when a consumer (IMP-112 / Cordage) lands.
