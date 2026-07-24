# Implementation Plan SL-225: Funnel false-red elimination

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two false-reds burn tokens in dispatch — a green worker delta reported as a
failure it cannot distinguish from its own damage (RFC-016 Cluster 1 / move B).
Both are project-tier dogfooding artifacts (POL-002): the fixes live in the
`justfile`, governance prose, and this repo's test suite; the engine is touched
by a **single neutral signal line**. DEC-003 (accepted) carries the reasoning
and its three-pass adversarial history; this plan sequences the build.

- **PHASE-01 — fix #1 + (ii).** The `worker_commit` commit gate's `just validate`
  false-reds because it shells the stale PATH `doctrine` (ISS-218). Rather than
  chase a fork-consistent binary (impossible from a fork — RV-291/292), `validate`
  **skips** its governance legs in a worker fork, where they carry no worker-delta
  signal, and — off the fork path — resolves the fresh coord build so `close`'s
  gate validates the landed corpus with a source-consistent binary (ii). The belt
  reorder (`build` before `validate`) makes that freshness belt-enforced, not prose.
- **PHASE-02 — fix #2.** ~30 authored-write e2e goldens spawn the CLI for writes
  the worker-mode guard correctly refuses under the marker; a worker's own
  `cargo test` reads those refusals as own-delta red. The goldens skip under the
  marker. The gate already clears the marker (SL-199 F2), so coverage is preserved.

## Sequencing & Rationale

**Why two phases, in this order.** Fix #1 and (ii) both edit the one `validate`
bash body (skip legs + fresh-binary resolution) plus the shared `check`/`gate`
belt order — splitting them would strand a mid-recipe seam across a phase
boundary, so they fold into PHASE-01. PHASE-02 is **file-disjoint** from PHASE-01
(`src/test_support.rs` + `tests/*` vs `justfile` + `src/mcp_server/worker_commit.rs`
+ governance prose), with no hard ordering dependency — the two could dispatch in
parallel. PHASE-01 leads because it is the headline ISS-218 fix and carries the
one engine line; PHASE-02 is the sibling ergonomic fix.

**The one engine line, and the boundary it must not cross (PHASE-01).** The only
`src/` change is `.env("DOCTRINE_DISPATCH_GATE","1")` on the gate spawn — a neutral
context signal, no cargo layout / path / binary-resolution policy (POL-002). It is
required specifically because `run_commit_gate` clears the marker and removes
`DOCTRINE_WORKER` for the gate's own run (so fix #2's goldens execute), leaving
neither of the other two signal legs visible in that window. Everything else in
PHASE-01 — the skip predicate, the fresh-binary resolution, the belt reorder — is
project-recipe territory; the behaviour-preservation gate (VT-1e) holds the engine
change to *exactly* that one line by requiring the existing `worker_commit` suites
to stay green unchanged.

**The (ii) residual and why the belt reorder is load-bearing (PHASE-01).** The
fork-skip's one coverage residual — a phase that changes `doctor`/`prompt check`'s
*own logic* — is closed at the orchestrator's close, where coord is a full built
tree (the fork-side location/existence blockers do not apply). The blocker that
*does* reach close is the belt's own validate-before-build order (RV-292 F-2's
ghost: `gate` runs `validate` before `build`), which would validate against a
prior build or silently fall back to stale PATH. Reordering `build` before
`validate` neutralises it in the belt itself — VT-1d is the discriminating proof.
This is why the `DOCTRINE_BIN` governance rule is retained and reframed, not
deleted: it now documents the coord-side close-time build (belt-enforced, so the
rule is documentation/override, not the load-bearing guarantee).

**OQ-1 resolved — the `WORKER_MARKER_REL` carve-out (PHASE-02).** `test_support.rs`
is `#[cfg(test)] mod` in the lib *and* `#[path]`-included into the separate
integration-test crate. `marker.rs::marker_path` is production (not `cfg(test)`),
so it cannot reference a `cfg(test)` const in `test_support.rs`; and the external
test crate sees only `pub` items, so it cannot reference `marker.rs`'s `pub(crate)`
const without leaking an internal path to public API. Single-sourcing is therefore
untractable without a `pub` leak. The plan takes the documented **carve-out**:
`WORKER_MARKER_REL` in `test_support.rs`, cross-pointing `marker.rs:114` as
canonical — the same CHR-014 dual-compilation class already tolerated. (The
`justfile` bash carries a third, cross-language copy no Rust const can absorb.)

**Verification shape.** PHASE-01's VTs drive the *real* seam: VT-1 the discriminating
gate green (skip pivots green-vs-red), VT-1b the generic-host no-mask negative,
VT-1c the signal-legs unit incl. the exact-`=1` negative, VT-1d the close-gate
freshness (build→validate pivots green-vs-red on a landed doctor-rule delta),
VT-1e the engine behaviour-preservation. VA-1 covers the governance prose reframe.
PHASE-02: VT-2 pins `under_worker_marker()`'s predicate (marker-gated, never masks
the unmarked arm); VA-1 enumerates the golden set against the CHR-044 list, which
no single VT can cover.

## Notes

- Immutable ids: `PHASE-NN` and criterion ids never renumber — edits append.
- The `justfile` bash reorder and the `validate` rewrite are code-impact executed
  at `/execute`; this plan authors the target, not the edit.

### Execution guidance (from the plan-critique pass)

- **VT-1 / VT-1b need no second binary.** The skip is binary-agnostic, so the
  discriminating proof uses a **broken-authored-state fixture** (a repo whose
  `.doctrine/` state the *current* binary's `doctrine doctor` reds) and pivots on
  the signal: present ⇒ `validate` skips ⇒ green (VT-1); absent ⇒ `validate` runs
  ⇒ red (VT-1b). Do NOT build a separate "stale" binary in-test — a broken-state
  proxy faithfully exercises the mechanism that dissolves ISS-218.
- **VT-1d — resolution leg is the tractable core.** `DOCTRINE_BIN` → a stub that
  reds/greens the corpus proves `validate` runs the *resolved* binary. The
  belt-order proof (fresh-beats-stale) should stay light — a stub whose
  freshly-`build`-produced variant differs from a pre-placed `./target/debug/doctrine`,
  or a recipe-order assertion — not a full in-test `cargo` rebuild. Settle the exact
  discriminator at `/phase-plan`.
- **PHASE-02 golden set is discovered, not guessed.** Enumerate the authored-write
  goldens by grepping for CLI spawners of writes the worker-mode guard refuses
  (`backlog new`, `install`, doctor-golden writers, …), cross-checked against the
  CHR-044 case-note list; read-only goldens stay untouched. This is VA-1's method.
- **PHASE-01 is the heavier phase** (real-`worker_commit`-seam e2e fixtures);
  `/phase-plan` owns its task decomposition. Keeping fix #1 + (ii) unified is
  deliberate — a split would make both halves edit the `justfile` and forfeit the
  file-disjoint, parallel-dispatchable boundary with PHASE-02.

### Low-severity documented bounds (not blockers)

- A leftover marker from a crashed dispatch would skip governance legs on a later
  manual `just validate` (gitignored runtime state, and the skip echoes a visible
  line).
- `quick` may validate against a prior build (it is the fast inner loop, not the
  close path).
