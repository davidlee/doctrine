# SL-038 — audit (conformance)

Post-implementation conformance audit of the cordage scale harness slice. Mode:
**conformance** (all three phases implemented; reconciling observed evidence
against `design.md`, `plan.toml` criteria, and SPEC-001). Hand-authored — no
audit scaffold yet.

Audit date 2026-06-11. Evidence is a fresh re-run of the committed harness, not
the phase-sheet numbers re-quoted.

## Evidence gathered

- `just check` — **green** (full lib/bin suite + fmt + clippy).
- `cargo test -p cordage -- --ignored --nocapture --test-threads=1` — **4 passed**
  in 52.99s (debug, jail target). Fresh measured values this run:
  - `explain_path_count_is_exponential_in_diamond_depth` — pass (exact `1<<18`).
  - `deep_chain_overflows_inside_target_scale` — pass (`fatal runtime error:
    stack overflow, aborting`, child non-success).
  - `eviction_fixpoint_scales_superlinearly` — `eviction ratio 18.5x for 4x edges`.
  - `evaluate_scales_quadratically_in_node_count` — `evaluate ratio 4.2x for 2x nodes`.
- Default gate `cargo test -p cordage` — reds are `#[ignore]`d, off the gate (VT-2).
- `git diff crates/cordage/Cargo.toml` — empty (zero-dep held, EX-4).

## Findings

### F1 — notes.md cited wrong `--exact` test names — FIX NOW (reconciled)

- **Expected** (PHASE-03 EX-1): each measurement carries a *working*
  reproduce-command.
- **Observed**: notes.md §1 cited four fabricated `--exact` fn names
  (`explain_path_count_is_exponential`, `build_overflows_stack_on_deep_chain`,
  `eviction_is_quadratic`, `evaluate_is_quadratic`) — none match the actual reds in
  `tests/scale_cliffs.rs`. Every `cargo test … --exact <name>` would have selected
  zero tests.
- **Evidence**: real fn names from `scale_cliffs.rs:111/123/145/161`.
- **Disposition**: **fix now** — corrected the four names in notes.md §1 inside
  this slice. The `--cliff` example names were verified correct
  (`scale_harness.rs:30`), no change.

### F2 — characterization ratios drift run-to-run — ALIGNED

- **Expected**: eviction ~18.5×, evaluate ~4× (PHASE-01/02 pins).
- **Observed**: audit re-run gave 18.5× / 4.2×; PHASE-02 pinned 18.7× / 4.3×.
- **Evidence**: the two runs above.
- **Disposition**: **aligned** — these are wall-clock characterization ratios under
  a loose bound (≪ 120s); single-digit-percent drift is scheduler noise, and both
  runs clear the quadratic-signal threshold unambiguously. notes.md §1 now records
  the provenance range (pin + audit re-run) rather than a single fragile number.

### F3 — eviction red labels "4× edges", notes framed "2× nodes" — ALIGNED

- **Observed**: the red prints `for 4x edges`; `dense_evict(50,50)`→`(100,100)` is
  2× nodes / 4× edges (dense). Both framings describe the same pair.
- **Disposition**: **aligned** — notes.md §1 now states both ("4× edges / 2× nodes").

### F4 — provenance precision (RSK-004 first-measured-here) — ALIGNED

- **Expected** (PHASE-03 EX-2): RSK-004 stated analytical-only / first-measured-here;
  RSK-002/003 = probe's numbers re-confirmed.
- **Observed**: notes.md §4 states the distinction unambiguously, anchored to
  `query.rs:256` and design D5.
- **Disposition**: **aligned**.

### F5 — fixes filed, not patched — ALIGNED

- **Expected** (PHASE-03 EX-3): Fix A/B/C/D filed, harness unpatched.
- **Observed**: RSK-002 (explain/Fix C), RSK-003 controls (overflow/Fix A +
  quadratic/Fix B), RSK-004 (evaluate/Fix D) — all **open**; slice §Follow-Ups
  enumerates all four. No cordage `src/` change in the slice diff.
- **Disposition**: **aligned** — cite, don't re-file. No new backlog items opened.

### F6 — SPEC-001 H1 wording reconcile deferred — FOLLOW-UP (already owned)

- **Observed**: H1 holds only after Fix A/B/D (explain needs C). The trailing H1
  *wording* reconcile is gated on the fixes landing — out of SL-038 (measure-and-red
  only).
- **Disposition**: **follow-up slice** — already owned in slice §Follow-Ups
  ("Trailing SPEC-001 H1 wording reconcile once the fixes land") and bound to the
  open RSK fixes. No new item required.

## Criteria reconciliation (all phases)

- **PHASE-01** EN-1/2, EX-1..4, VT-1, VA-1 — met (example compiles public-API-only,
  four CSV cliffs, evaluate pair pinned (2000,4000), zero-dep, clippy-clean examples).
- **PHASE-02** EN-1, EX-1..3, VT-1/2/3 — met (four `#[ignore]`d reds, self-re-exec
  overflow via `var_os`, `Along`+`Flag(true)` evaluate seed, default gate untouched,
  75 existing tests intact).
- **PHASE-03** EN-1, EX-1..3, VA-1 — met *after F1 fix*. **VH-1 (user accept)** —
  the one open gate; carried to `/close`.

## Durable harvest

- Fix A/B/C/D homes: RSK-002, RSK-003 (both controls), RSK-004 — all open. No new
  backlog work surfaced by the audit.
- Memory: debug-vs-release ~10× scale timing already captured
  (`mem.pattern.testing.debug-vs-release-scale-timing`); opaque-id capture-from-
  builder already captured (`mem.pattern.cordage.opaque-ids-capture-from-builder`).
  No new durable pattern from this audit beyond F1's lesson (reproduce-commands must
  cite the real `#[ignore]`d fn name) — too thin to record standalone.

## Closure story

Harness is committed, reproducible, and green-as-characterization; the findings note
consolidates four cliffs with *working* reproduce-commands (post-F1), an honest H1
position, precise RSK-004 provenance, and the OQ-2 gap. All findings dispositioned;
the only open gate is VH-1 (user acceptance). **Audit-ready for `/close`.**
