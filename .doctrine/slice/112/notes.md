# SL-112 audit notes — durable findings from the reconciliation audit

<!-- Runtime notes; the slice owns this file. -->

## 2026-06-20 — audit (RV-104)

**Evidence baseline.** Candidate `cand-112-review-001` reviewed (db948c4,
3-way merge of `review/112` onto `main`). `cargo test --test architecture_layering`:
17 passed, 0 failed, 1 ignored (`dump_real_graph`). Clippy: zero warnings.

**Findings dispositioned (3):**
- F-1 (minor, fix-now): `relation_graph` missing from layering.toml [tiers]; hard-coded in
  `load_layering()`. Fix the authored canon + remove fallback.
- F-2 (nit, aligned): `MixedUmbrella` variant drops `file` field vs design §5.2.
  No functional harm — module name suffices to direct sub-classification.
- F-3 (minor, aligned): `main` exempted from completeness check — reasonable
  (binary entrypoint, not an architectural module).

**Gate proven to bite.** Eight synthetic bite-tests cover all violation types;
the real-graph `architecture_layering_gate` asserts zero violations under
authored baselines.

**PHASE-03 deferred to reconcile.** ADR-001 amendment (overturn rejection, record
gate as enforcement, replace prose tier table with definitions + layering.toml
pointer, reclassify `input`) is the close co-requisite routed through the
reconciliation brief (REV).

**Standing risks:**
- `load_layering()`'s manual `[[accepted_violation]]` parser expects inline
  `from = "A"; to = "B"` syntax — fragile to TOML reformatting (e.g. multi-line).
- Boundary-stability caveat (R8): folding/splitting units shifts tangle count, but
  the partition is reviewed canon (`layering.toml`, REV-routed).
- Literal `crate::` path scope means macro/re-export laundering can evade `syn`
  (design F-4) — no present breach, review-covered.
