# Review RV-085 — reconciliation of SL-101

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-101 (Estimate & Value facets) against `design.md`, the
phase `VT-`/`EX-` criteria, ADR-001, and the SPEC-020/PRD-014 lineage. Candidate
surface reviewed: `refs/heads/candidate/101/review-001` (tip `700c6caa`) — the
no-ff 3-way merge of the impl bundle (`review/101`, `e459259f`) onto `main`
(`ec2de060`); the `notes.md` merge conflict was resolved (both sides kept).

Lines of attack (invariants this audit holds SL-101 to):

1. **ADR-001 leaf purity** — `estimate.rs`/`value.rs` import only external crates.
2. **FR-002 validation matrix** — present bounds required, finite, ordered; absent
   clean; no silent repair.
3. **NF-001 non-blocking** — no workflow predicate reads facet presence.
4. **NF-003 forward-compat** — unknown keys tolerated at parse, dropped at normalise.
5. **FR-004 round-trip** — valid facets survive parse→serialise→parse.
6. **Behaviour-preservation gate** — existing conduct/verification/slice suites
   stay green unchanged.
7. **Design §3.3 "No runtime effect in this slice"** — confidence bounds are
   "purely informational until consumed"; the slice must not give them a runtime
   effect that couples unrelated config reads.
8. **Design signature fidelity** — impl signatures match design §3.3/§4.3, or
   divergences are consciously dispositioned.

Evidence: 1767 bin tests pass on the candidate (0 failed, 1 ignored); `cargo fmt`
+ `cargo clippy` clean. Pre-existing main failure
(`dispatch_router_skill_is_shrunk`) is out of scope — SL-101's bundle does not
touch that SKILL.md.

## Synthesis

SL-101 ships two clean, kind-agnostic pure-leaf facet modules that honour the
design's intent. The leaf-purity (ADR-001), validation-matrix (FR-002),
forward-compat (NF-003), round-trip (FR-004), and non-blocking (NF-001)
invariants all hold: `estimate.rs`/`value.rs` import only `toml`/`serde`/`anyhow`/
`std`; unknown keys are absorbed by `#[serde(flatten)] _extra` and dropped at
normalise; no workflow predicate reads facet presence; 1768 bin tests pass.

The one substantive finding (F-1, major → fix-now) was a real behaviour-
preservation divergence: `dtoml::parse()` — the shared reader for conduct,
verification, and coverage_store config — eagerly ran `resolve_confidence()?`,
so a malformed `[estimation]` confidence table would fail every doctrine.toml
read across unrelated commands. This contradicted design §3.3's explicit "No
runtime effect in this slice — purely informational until consumed" and the
PHASE-03 EX-4 intent. The fix removed the eager call and marked the now-v1-unused
facet API with the same `#[cfg_attr(not(test), expect(dead_code, …))]` convention
already used by `parse_optional` — these symbols are consumed by SL-102/SL-103.
A regression test pins the fix. F-2 (the dead `resolve_unit` discards) rode the
same edit.

F-3 and F-4 are design-doc inaccuracies the impl resolved correctly: §3.3's
`resolve_confidence` signature shows a bare `(f64, f64)` while its own docstring
promises validation (impl correctly returns `Result`); §6.1's `SliceDoc` derive
still lists `Eq`, impossible now that the struct carries f64 facets (impl
correctly dropped `Eq`). Both are delegated to `/reconcile` as per-slice
`design.md` edits — no code change wanted, the impl is right.

**Standing risks / tradeoffs consciously accepted:**
- The facet API (`resolve_unit`, `resolve_confidence`, the `DEFAULT_*` constants,
  `DoctrineToml.estimation`/`.value`) is dead in v1 by design — marked, not
  removed. SL-102/SL-103 must consume them or trim them.
- `parse_optional` re-serialises its `&Table` arg through `toml::to_string` then
  `toml::from_str` rather than `toml::from_value`. Works, tested, but a minor
  inefficiency — not raised as a finding (no behavioural impact; cosmetic).
- Pre-existing main failure `dispatch_router_skill_is_shrunk` (54 lines vs ≤45
  target) is out of SL-101's scope; flagged to the user, not the slice.

## Reconciliation Brief

### Per-slice (direct edit on `design.md`)

- **F-3 → design §3.3:** `resolve_confidence` signature shows
  `fn(cfg: &EstimationConfig) -> (f64, f64)` but the docstring says "validated:
  finite, in [0.0, 1.0], lower < upper". A bare tuple cannot report validation
  errors. Update §3.3 to `-> anyhow::Result<(f64, f64)>` to match the implemented
  (and correct) signature in `src/estimate.rs:54`.
- **F-4 → design §6.1:** `SliceDoc` derive list still shows `Eq`, but the struct
  now carries `Option<EstimateFacet>`/`Option<ValueFacet>` whose `f64` fields are
  `PartialEq` but not `Eq`. Update §6.1 to derive only `PartialEq` and add a
  one-line note ("`Eq` dropped — f64 facets are not `Eq`"). No code change; impl
  is already correct (`src/slice.rs:989`).

### Governance/spec (REV)

- None. The SPEC-020 amendments flagged in SL-101's inquisition (RV-082) —
  default unit `high_caffeine_hours` → `espresso_shots`, Value facet coverage —
  are tracked on the inquisition's standing list and are the reconcile-stage
  spec edits, not audit findings. They surface here for handoff continuity.

### Code fixes applied within audit (fix-now, on `candidate/101/review-001`)

- **F-1 / F-2** — `dtoml::parse()` no longer eagerly validates estimation config;
  the shared reader is decoupled from estimation validity. Regression test added
  (`dtoml::tests::malformed_estimation_confidence_does_not_block_config_read`).
  Commits `8598dbca` (fix) + `a50f7092` (fmt) on `candidate/101/review-001`.

## Reconciliation Outcome

### Direct edits applied

- **design.md §3.3 (RV-085 F-3):** `resolve_confidence` signature corrected from
  bare `(f64, f64)` → `anyhow::Result<(f64, f64)>` to match the implemented (and
  correct) `Result` return.
- **design.md §6.1 (RV-085 F-4):** `Eq` dropped from `SliceDoc` derive; one-line
  note added ("Eq dropped — f64 facets are PartialEq but not Eq"). Impl already
  correct.

### REVs completed

- **REV-001** (`reconcile-sl-101`): **done** — SPEC-020 amended with two changes:
  - Default estimation unit `high_caffeine_hours` → `espresso_shots` (RV-082 F-1)
    in responsibilities, prose, and acceptance criteria.
  - Value facet coverage added (RV-082 F-2): `ValueFacet` model, validation, and
    unit resolution (`magic_beans`) in responsibilities, MD prose, and D5 decision;
    new requirements FR-007/FR-008/FR-009 introduced.
  Rationale in `revision-001.md`.

### Withdrawn / tolerated

- None — all four RV-085 findings were `verified` with `fix-now` or `verified`
  dispositions and are now resolved.
