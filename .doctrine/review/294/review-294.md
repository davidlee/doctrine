# Review RV-294 — reconciliation of SL-225

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation self-audit). **Surface reviewed:** the
landed source on `edge` (solo, not dispatched) — commits `ad1fff949` + `575757955`
(PHASE-01, fix #1 + ii) and `613af9c51` (PHASE-02, fix #2); lifecycle at `837801a5`.

**What this audit probes** — the two false-reds the slice charters to dissolve
(ISS-218 stale-binary gate red; CHR-044 marker-poisoned goldens) and the invariants
DEC-003 pins the fix to:

1. **Engine surface is one neutral signal line.** `worker_commit` gains only
   `.env("DOCTRINE_DISPATCH_GATE","1")` on the gate spawn; no binary resolution, no
   cargo/`target` layout, no gate-logic change (POL-002). The *project recipe* decides
   what to skip.
2. **The skip loses no signal.** `doctor`/`prompt check` read authored `.doctrine/`
   state a worker cannot write, so in a fork their verdict = coord's already-green
   verdict up to binary version — skipping removes only the false-red.
3. **The residual is closed, not conceded.** A phase mutating `doctor`'s own logic is
   caught at close by a fresh-binary corpus gate — belt-enforced via `build`-before-
   `validate` in `check`/`gate`, not an operator-remembered step.
4. **Both fixes are strictly worker/marker-gated** — a generic host / the main
   (unmarked) arm runs the checks unchanged, so no real regression can hide (the
   no-mask negatives VT-1b / EX-3).
5. **(ii) is entirely project-tier** — zero engine, zero shipped-skill surface, inert
   for client repos.

**Lines of attack / where bodies may be buried:**

- **Conformance registry is polluted** — PHASE-01's solo auto-binding
  (`boundaries.toml` `[886153a5f..30f2b82a8]`) swallows 8 interleaved IMP-306/chore
  commits because sibling work landed on shared `edge` between SL-225's two
  non-contiguous PHASE-01 commits. `slice conformance` reports 55 undeclared (mostly
  foreign). Verified manually via per-commit `--against` (F-3).
- **Design-premise drift, both phases** — flagged in the phase sheets; the design's
  illustrative surface lists (CLAUDE.md edit target; three named goldens) diverge from
  what single-sourcing / empirical enumeration actually delivered (F-1, F-2).
- **Charter-edge inclusion** — `e2e_check_regression` guarded as a PHASE-01 fingerprint
  interaction, not a write-refusal (F-4).
- **Guard verbosity** — ~95 explicit early-returns vs a macro (F-5).

**Evidence run (all green):** `doctrine check gate` → exit 0 (fresh binary,
`build`→`validate`, full `test-all`); the IMP-306 handover-SKILL foreign redness the
phase-01 sheet flagged is resolved (IMP-306 closed). `e2e_worker_gate_skip` 7/7
(VT-1/1b/1c/1d). `worker_marker_at` unit green (VT-2). worker_commit suites green
unchanged (VT-1e / behaviour-preservation). Implementation matches design verbatim:
`validate` body (3-leg skip + `${DOCTRINE_BIN:-./target/debug/doctrine}` + `command -v`
fallback + exact `="1"`); belt reorder; the one engine env line; governance.md reframe;
helper + 27 guarded goldens with `e2e_worker_guard` correctly **un**guarded.

## Synthesis

**Closure story.** SL-225 dissolves both charter false-reds and holds every DEC-003
invariant. Fix #1 makes the worker gate stop re-running coord's governance
self-checks: `just validate` skips `doctrine doctor` / `prompt check` under a
three-leg worker-context signal (`DOCTRINE_DISPATCH_GATE` | `DOCTRINE_WORKER=1` |
the marker file), because those checks read authored `.doctrine/` state a worker
cannot write and so carry zero worker-delta signal in a fork — they could only
stale-binary false-red (ISS-218). The engine surface is exactly one neutral line
(`.env("DOCTRINE_DISPATCH_GATE","1")` on the gate spawn); the *project recipe*
decides what to skip — POL-002 holds, contrast the three rejected designs that tried
to make the engine or a git inference resolve *which binary*. Fix #1 (ii) closes the
one residual — a phase mutating `doctor`'s own logic — at close, via a fresh-binary
corpus gate that is **belt-enforced**: `check`/`gate` run `build` before `validate`,
so `doctrine check gate` validates the landed corpus with a this-invocation-fresh
binary, no operator-remembered step. Fix #2 gives the authored-write e2e goldens a
`under_worker_marker()` early-return so a worker's own `cargo test` reflects delta
health, not the guard's correct refusals; coverage is preserved because the
server-side gate clears the marker (SL-199 F2) and the unmarked/main arm always runs
them.

Evidence is green throughout: `doctrine check gate` exit 0 (fresh binary, full
`test-all`); the discriminating VT-1 suite 7/7 (the green-vs-red pivot IS the skip);
VT-1d proves (ii)'s enforced closure (belt reorder + `DOCTRINE_BIN` resolution);
VT-2 the predicate; worker_commit suites green unchanged (behaviour-preservation).
Implementation matches design verbatim. The IMP-306 handover-SKILL foreign redness
the phase-01 sheet flagged as blocking has since resolved (IMP-306 closed with the
de-hash/de-colon fixes) — the gate is now clean.

**Standing risks (all bounded, none blocking).**
- *Stale-marker bound (low).* A leftover `.doctrine/state/dispatch/worker` from a
  crashed dispatch would skip the governance legs on a later manual `just validate` in
  that tree. Bounded: gitignored runtime state, the skip echoes a visible line (not
  silent), and the property is shared with fix #2's existing predicate — no new
  persistence risk.
- *`quick` fast-path bound (documented).* `quick` may validate against a prior build;
  a conscious low-severity carve-out, not the close path.
- *OQ-1 carve-out.* `WORKER_MARKER_REL` duplicates `marker.rs:114` with a cross-pointer
  comment — the same CHR-014 dual-compilation class the repo already tolerates;
  single-sourcing is untractable across the external test-crate boundary without a
  `pub` leak.

**Tradeoffs consciously accepted.** (F-4) The `e2e_check_regression` guard is
in-charter — a false-red born of the slice's own PHASE-01 mechanism, marker-gated so
no coverage is lost. (F-5) The ~95 explicit early-returns are kept over a macro to
preserve greppability (the property VA-1's set-membership audit leans on) and avoid
hiding control-flow. (F-3) PHASE-01's boundary registry is polluted by shared-edge
solo auto-binding; conformance was verified manually instead, and boundaries.toml is
disposable runtime state.

**Design-premise drift (F-1, F-2) — documentary, not defect.** The design's
illustrative surface lists lag what single-sourcing (CLAUDE.md) and empirical
enumeration (the three goldens) actually delivered. The delivered code is correct and
complete; the design prose + three stale selectors need reconciliation. No behaviour
impact.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §"What this dissolves" F-3 bullet (L134-137) + §"Code impact" table
  CLAUDE.md row (L236) + the EX-5 premise** (F-1): drop `CLAUDE.md` as a *distinct
  authored edit target* for the reframed DOCTRINE_BIN rule. State that the rule is
  single-source in `.doctrine/governance.md` (STD-001) and reaches the agent-facing
  CLAUDE.md surface via the `@.doctrine/state/boot.md` inline, refreshed by
  `doctrine boot`. EX-5's *intent* was met; only its premise was wrong.
- **design.md §"Which goldens" (L224) + §"Code impact" table golden row (L239)**
  (F-2, prose mirror): replace the three illustrative filenames
  (`e2e_worker_guard`/`e2e_dispatch_sync`/`e2e_doctor_golden`) with a pointer to the
  empirical CHR-044 target set enumerated at implementation (27 marker-poisoned
  goldens; the PHASE-02 sheet's Findings pin it). Note that `e2e_worker_guard` is
  deliberately *excluded* (it exercises the guard).

### Governance/spec (REV)

- None. The slice's governance touch (`.doctrine/governance.md` DOCTRINE_BIN reframe)
  was authored in-phase and is correct; no ADR/policy/standard/spec requires a REV.

### Selector registry (names the verb, not the prose)

- **`doctrine slice selector rm`** the three undelivered design-target selectors
  `tests/e2e_worker_guard.rs`, `tests/e2e_dispatch_sync.rs`, `tests/e2e_doctor_golden.rs`
  (F-2). This is the load-bearing change that clears conformance's `undelivered` cells
  — `slice conformance` reads the selector registry (`slice-225.toml`), of which the
  §239 design-table row is only the human mirror. The remaining four selectors
  (justfile, worker_commit.rs, test_support.rs, common/mod.rs) are conformant and
  stay. The ~27 delivered goldens remain conformance-`undeclared` by design (the
  slice's real fix-#2 surface, documented here rather than enumerated as 27 selectors
  — conformance is necessary, not sufficient).

### Not a reconcile surface (recorded for completeness)

- **F-3 boundary-registry pollution** — `boundaries.toml` is gitignored/disposable
  runtime state, and PHASE-01's non-contiguous two-commit landing admits no clean
  contiguous re-record. No authored surface to write; conformance was verified
  manually. The durable *tooling* hazard (solo auto-binding uses a HEAD-range, not
  per-commit) is **already tracked** — no new mint: **IMP-175** (solo phase-binding
  stamps stale `code_start_oid` when edge advances before land — the identical
  mechanism, with candidate fixes sketched) and **IMP-292** (audit-time conformance
  signal degradation, incl. authored-metadata pollution). SL-225 PHASE-01 is a fresh
  repro of IMP-175.
