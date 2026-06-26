# Review RV-171 — reconciliation of SL-162

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Reviewed surface.** Dispatched slice (claude arm). Audit ran against the
candidate interaction branch `candidate/162/review-001`
(`cand-162-review-001`, tip `5b313b68`) — the impl_bundle of `review/162`
3-way-merged onto `refs/heads/main`. `review/162` and the worker branches
(`dispatch-worker/162/PHASE-01..02`) are immutable evidence refs (R2).

**What this audits.** SL-162 extends the CHR-014 runtime-resolve pattern from
`CARGO_MANIFEST_DIR` to `CARGO_BIN_EXE_doctrine`: a new
`test_support::doctrine_bin()` resolver, its `tests/common/mod.rs` re-export, a
59-file `const BIN`→`bin()` sweep, and a generalised reintroduction guard.

**Lines of attack / invariants held:**
1. **Behaviour-preservation gate (F4)** — the e2e goldens must stay
   byte-identical and every swept suite green. The sweep is the risk (R1: 59-file
   transcription error). Proof = run the suite.
2. **INV-1 / F3 — zero baked path.** No `env!("CARGO_BIN_EXE…")` or
   `env!("CARGO_MANIFEST_DIR")` in compiled *code* (comments exempt; guard skips
   `//`). Resolver body uses `current_exe()`, never a macro.
3. **Guard integrity (D3/F2)** — rename `e2e_no_baked_manifest_dir`→
   `e2e_no_baked_paths` (git mv), both needles fragment-assembled so the guard
   never self-trips, comment-skip intact.
4. **Conformance** — `slice conformance 162`: undeclared/undelivered cells.
5. **Single source / layering (F5/F6)** — resolver lives once in
   `src/test_support.rs`; test-only, no ADR-001 impact.

## Synthesis

**Verdict: clean. No code changes required; reconcile is cosmetic.** SL-162
delivered exactly what the design specified, and the behaviour-preservation gate
— the slice's own stated proof — holds.

**Evidence.**
- *Behaviour preservation (F4 / VT-1 / VT-2).* The full integration suite on the
  candidate (`candidate/162/review-001`, tip `5b313b68`) runs with exactly **one**
  failing test across all binaries, and that failure
  (`e2e_estimate_non_blocking::no_facet_symbol_outside_allowlist`) is inherited
  from `main` and untouched by this slice (F-2). Every one of the 59 swept e2e
  suites is green and the goldens are byte-identical — a golden assertion would
  fail on any stdout/JSON drift, so green *is* the byte-identity proof. The
  formerly-failing suites (`e2e_adr_cli_golden`, `e2e_backlog_filter_alias`) pass.
- *Zero baked path (INV-1 / F3 / VA-1).* No `env!("CARGO_BIN_EXE…")` or
  `env!("CARGO_MANIFEST_DIR")` survives in compiled code. Every residual mention is
  a doc-comment (`///`/`//!`), which the guard's comment-skip exempts; the `.disabled`
  spike that still names `CARGO_MANIFEST_DIR` is not a `.rs` file and is not compiled
  (CHR-014's domain, not this slice's). The resolver body uses `current_exe()`.
- *Resolver (VT-4 / IMP-185).* `test_support::doctrine_bin()` matches design §5.2
  verbatim (pop exe, pop `deps/`, push `doctrine`+`EXE_SUFFIX`). Its constructor
  test (`doctrine_bin_returns_existing_executable`) passes: resolves an existing,
  non-zero, `doctrine`-named file.
- *Guard (D3 / F2 / VT-3).* `e2e_no_baked_manifest_dir.rs` → `e2e_no_baked_paths.rs`
  via `git mv` (old gone, new present); both needles are fragment-assembled so the
  guard never self-matches; `no_baked_paths` passes.
- *Conformance.* 0 undeclared, 1 undelivered — a benign selector-overlap (F-1), not
  drift. Single source and layering (F5/F6) intact: resolver lives once in
  `src/test_support.rs`, re-exported through the existing CHR-014 seam.

**Standing risks / tradeoffs consciously accepted.**
- **VH-1 cross-namespace proof is unexercised in-jail** (a single namespace cannot
  demonstrate it). Accepted at design time (review F4) and marked VH; in-jail
  correctness is by construction (no baked path). The only direct proof is a human
  cross-namespace run. Not a blocker.
- **Lost `CARGO_BIN_EXE_*` build-graph link** (§5.4): a missing bin now surfaces as
  a runtime spawn error rather than a link error. Mitigated by running via
  `cargo test`. Accepted.

## Reconciliation Brief

All findings are `aligned`. Nothing routes to a governance/spec REV. Two items are
optional per-slice cosmetic tidies; one is out-of-scope work harvested to backlog.

### Per-slice (direct edit) — optional, non-gating
- **design.md §5.2** (F-4): the `tests/common/mod.rs` inner-attribute snippet shows
  `#![allow(dead_code)]`; the delivered code is `#![allow(dead_code, unused_imports)]`.
  Design R4 already sanctions the wider allow — update the §5.2 snippet to match so
  prose tracks code. Cosmetic.
- **design-target selectors** (F-1): the literal `tests/e2e_no_baked_paths.rs`
  selector overlaps the `tests/e2e_*.rs` glob, producing a spurious "undelivered"
  cell. Optionally drop the redundant literal so `slice conformance` reads clean.
  Cosmetic; no correctness impact.

### Governance/spec (REV)
- None.

### Out of scope → backlog (harvested, not reconciled here)
- **F-2**: `main` is red on `no_facet_symbol_outside_allowlist` — `src/knowledge.rs`
  names a facet symbol outside the NF-001 allowlist. Pre-existing, unrelated to
  SL-162. Captured as a backlog issue.
- **F-3**: dispatch candidate worktree provisioning does not seed gitignored build
  assets (`web/map/dist/`), so a fresh worktree cannot compile the bin until copied.
  Harvested as a durable gotcha.
