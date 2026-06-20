# SL-128 — notes

## Audit (RV-118, 2026-06-21)

Reconciliation audit ran against the dispatched candidate surface
(`candidate/128/review-001` → repaired → `review-002`). Conformance clean on all
load-bearing axes (R3 behaviour-preservation, I2 `--integrate` untouched, ADR-001
layering, D3 precedence, codex-F3 failure ordering, VT-1 prose hygiene). Default
`refs/heads/main` unchanged; live `dispatch deliver-to` verifies default +
override.

Two fix-now findings, both code-hygiene, fixed on the impl bundle `review/128`
(commit `72527bbd`):

- **F-1 (minor)** — bundle not rustfmt-clean (`cargo fmt --check` failed on
  slice.rs / main.rs / dtoml.rs / e2e_dispatch_sync.rs). **Cause:** the dispatch
  funnel skipped the prescribed scoped `rustfmt --check <touched files>` verify
  step (workers can't run `just check` — lint-js node_modules gap), so fmt drift
  reached the bundle and surfaced only at audit. The fix is already documented in
  `mem.pattern.dispatch.pi-arm-worker-ops` ("scope verify… + rustfmt --check");
  this was that guidance not being followed, not a new gap. **Practice note:**
  audit a dispatched bundle with `cargo fmt --check` on the candidate — it catches
  funnel verify scoping gaps.
- **F-2 (nit)** — `dtoml.rs` module doc claimed the file read "lives in the shell"
  after this slice co-located the impure `load_doctrine_toml` there; doc corrected.

No reconcile write surface owed by the findings; design.md matches the
implementation. IMP-124 resolves on close (slice scope §6); IMP-129 is the
downstream follow-up.
