# Review RV-144 — reconciliation of SL-142

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

### Lines of attack

1. **Formula fidelity** — Does `base_score` compute `tag_term = max(0.0, 1.0 + Σ(coeff − 1.0))`
   exactly per ADR-015 §1 / REV-009 and the design?
2. **Data path completeness** — Do `EntityFacets` → `read_facets` → `ScannedEntity` →
   `build_from` all carry tags correctly, including edge cases (absent, non-array)?
3. **No normalize_tag in read path** — RV-143 F-4 fix: tags pass byte-identical;
   `read_facets` does not call `tag::normalize_tag`.
4. **Test coverage** — Are all 5 PHASE-02 unit tests present, correct, and passing?
   Are VT-2 through VT-5 from PHASE-01's plan covered by scan.rs tests?
5. **Dead code removal** — Is `#[expect(dead_code)]` on `tag_coeff()` actually removed?
6. **Call site adaptation** — `slice.rs` and `hydrate.rs` test helpers correctly add
   `tags: vec![]` / `doc.tags.clone()`?
7. **Golden tests** — Identity semantics means no corpus tag-bearing entities → goldens
   unchanged. Verify this holds.
8. **Architecture layering** — No newly introduced cycles (ADR-001).
9. **Config clamping** — `tag_coefficients` values go through `clamp_general` (like
   `kind_weights`), preventing NaN/inf/negative affecting scores.

### Review surface

Candidate branch `candidate/142/review-001` at commit `a2382c292eb4`.

## Synthesis

### Summary

SL-142 wires entity tags into the priority scoring pipeline. The implementation
faithfully matches the design (`design.md`), plan (`plan.toml`), and governing
ADR-015 §1 / REV-009. All nine lines of attack were verified and confirmed
aligned:

1. **Formula fidelity** ✅ — `tag_term = max(0.0, 1.0 + Σ(coeff − 1.0))` is
   computed at line 79 of `graph.rs` and multiplied into `value_dim` at line 90.
   Exact match to the design and ADR-015 §1 per REV-009.

2. **Data path completeness** ✅ — `EntityFacets` (facet.rs), `read_facets`
   4-tuple return (scan.rs), `ScannedEntity` (scan.rs), and `build_from`
   constructor (graph.rs) all carry `tags: Vec<String>`. Tests cover present
   array (VT-2), absent key (VT-3), non-array value (VT-4), and
   pass-through byte-identical (VT-5).

3. **No normalize_tag in read path** ✅ — VT-5 (sl-142 PHASE-01) proves tags
   pass through byte-identical. `tag::normalize_tag` is NOT called in
   `read_facets`. RV-143 F-4 fix confirmed.

4. **Test coverage** ✅ — All 5 PHASE-02 `base_score` tag tests present and
   passing: identity (empty tags), promotion (single coeff 2.0), multiple tags
   (coeffs 1.5+2.0), demotion (coeff 0.5), and multi-demote floor (coeff 0.0).
   Plus 4 PHASE-01 `read_facets_tags_*` tests. 10 total tag-related unit tests.

5. **Dead code removal** ✅ — `#[expect(dead_code)]` on `tag_coeff()` is
   removed. grep confirms no `dead_code` or `expect` annotations remain in
   `config.rs` source.

6. **Call site adaptation** ✅ — `slice.rs` (5 locations: `run_show` + 4 test
   helpers) and `hydrate.rs` (1 test helper) correctly add `tags: vec![]` or
   `tags: doc.tags.clone()`.

7. **Golden tests** ✅ — All 16 `e2e_priority_golden` tests pass unchanged.
   No corpus entities carry tags, so identity semantics keep goldens stable.

8. **Architecture layering** ✅ — `cargo test` architecture_layering gate
   passes (17 tests, 0 failures). No new cycles introduced (ADR-001).

9. **Config clamping** ✅ — `tag_coefficients` values flow through
   `clamp_general` in `config.rs` lines 140-143, same as `kind_weights`.
   NaN/inf → fallback 1.0; negative → 0.0; > COEFF_MAX → capped.

### Boundary

`.doctrine.toml` seed values for `[priority.tag_coefficients]` remain
item B of RFC-002 and are explicitly out of scope. Without configured
coefficients in the working config, `tag_coeff()` returns 1.0 for all
tags — identity semantics. This is correct per scope.

### Pre-existing test failure

`e2e_memory_sync::sync_produces_all_shipped_dirs` fails on both `main` and
the candidate branch (jail filesystem path issue). Not related to SL-142.

### Conclusion

No findings. The implementation is complete, faithful to the design, and
all verification criteria from `plan.toml` are met. The slice is ready for
reconciliation.

## Reconciliation Brief

No spec/governance findings were raised during audit. The implementation
matches the design exactly. No reconciliation changes needed.

- design.md: aligned (no edit needed)
- ADR-015: aligned per REV-009
- plan.toml criteria: all covered and passing
- No REV or governance surface requires modification

## Reconciliation Outcome

No findings were raised during audit. The implementation faithfully matches
the design. No writes needed.

Reconcile pass complete — handoff to /close.
