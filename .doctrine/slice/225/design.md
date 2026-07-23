# SL-225 — Funnel false-red elimination — Design

> Cluster 1 / move B of RFC-016. Two false-reds — a green worker delta reported
> as a failure it cannot distinguish from its own damage. Both are **project-tier**
> dogfooding artifacts (POL-002): the fixes live in the `justfile`, governance, and
> this repo's test suite — **the engine (`src/mcp_server/**`) is untouched**. See
> DEC-003.

## Problem

A dispatch worker iterates in an isolated fork and must reach a green local suite
before `worker_commit`. Two recurring reds are **not** the worker's delta:

- **#1 (ISS-218, 6+ repros).** The commit gate's `just validate` shells bare
  `doctrine` → PATH `~/.cargo/bin/doctrine`, built from edge/main and RO in the
  jail. When an earlier phase shipped a binary-level rule change (new role /
  allowlist / check) on `dispatch/<slice>`, the PATH binary doesn't know it →
  `doctrine doctor` false-reds a correct fork. Independent of the current phase's
  delta (recurs on a pure-JS phase — SL-206-P14).
- **#2 (CHR-044, 7+ repros).** ~30 e2e goldens spawn the CLI for authored writes
  (`backlog new`, `install`, …). In a worker fork the worker-mode guard correctly
  refuses those writes under the marker. The server-side gate already clears the
  marker for its own run (SL-199 F2, `worker_commit.rs:147`), so the residual burn
  is the **worker agent's own** `cargo test` / `just check` — run to check its
  work — hitting the marker and reading the refusal as own-delta red.

Neither is a real regression; both burn tokens on diagnosis of noise.

## Fix #1 — gate resolves a fork-consistent `doctrine` (zero-engine)

The commit gate is `check commit` → `DEFAULT_COMMIT = ["just","check"]`
(verify.rs:149) → the `check` recipe runs `validate`, which shells `doctrine`. So
`validate` is the seam. The fix is **project-tier only — the engine is untouched**
(DEC-003; the engine-publish first considered here was rejected, see below).

### Current vs target

Current (`justfile:28`):
```
validate:
  doctrine prompt check      # → PATH doctrine (stale in a dispatch fork)
  doctrine doctor
```

Target — **layered** resolution, PATH kept as the generic-host tail:
```
validate:
  #!/usr/bin/env bash
  set -euo pipefail
  bin="${DOCTRINE_BIN:-}"
  [ -z "$bin" ] && [ -x ./target/debug/doctrine ] && bin=./target/debug/doctrine
  [ -z "$bin" ] && bin=doctrine
  "$bin" prompt check
  "$bin" doctor
```
Resolution order: **`$DOCTRINE_BIN` → local `./target/debug/doctrine` → PATH**.
The PATH tail is *correct* for a generic host (their installed binary is not stale);
the earlier rungs are the dogfooding override.

### Why the engine-publish was rejected (adversarial reversal)

`.mcp.json` launches the server via `${DOCTRINE_BIN:-doctrine}`, and the gate is a
child of the server process, so it **already inherits** the server's `DOCTRINE_BIN`.
Publishing `DOCTRINE_BIN=current_exe()` from `worker_commit` is therefore redundant
when the var is set, and **harmful when it is unset** — `current_exe()` is then the
stale server binary and pinning it pre-empts the fall-through to the fork's fresh
local build. So no engine change; `worker_commit` is not in this slice's surface.
Full reasoning: DEC-003.

### The precondition (governance, not code)

`$DOCTRINE_BIN` must point at the **coord build** for a dispatch session — it is on
`dispatch/<slice>`, so it carries earlier phases' rules and is the canonical
correct binary. The forwarding path already exists (`flake.nix try-fwd-env` +
`.mcp.json`). This is recorded in **`.doctrine/governance.md` (§ orchestration)**,
per the User's direction that the precondition live outside any skill. The residual
corner — a non-Rust phase with `DOCTRINE_BIN` unset (fork unbuilt → PATH fallback,
the SL-206-P14 repro) — is closed by that governance rule, **not** by mechanism
(resolving the coord path in the engine would re-bake cargo layout → POL-002).

## Fix #2 — marker-aware skip in the authored-write goldens

### Detection helper (project test-support, SL-162 pattern)

Integration tests are a separate crate — `pub(crate) marker_present` (marker.rs:118)
is unreachable and the fork-root path must be resolved test-side. Add to
`src/test_support.rs` (single source, `#[path]`-included by `tests/common/mod.rs`
alongside `doctrine_bin`/`repo_root`):

```rust
/// True when running inside a dispatch worker fork — the env leg (subprocess arm)
/// OR the marker file (claude arm, marker-file-only; case-note #6). Authored-write
/// e2e goldens skip on this so a worker's own `cargo test` reflects delta health,
/// not the worker-mode guard's (correct) refusals. The server-side commit gate
/// CLEARS the marker before its run (SL-199 F2), so the goldens still execute there
/// — coverage is preserved; only the worker's manual run skips.
pub(crate) fn under_worker_marker() -> bool {
    std::env::var_os("DOCTRINE_WORKER").is_some()
        || repo_root().join(WORKER_MARKER_REL).exists()
}
```

Each authored-write golden gains a guarded early-return:
```rust
if common::under_worker_marker() { return; }  // SL-225 #2: skip in a worker fork
```

### Which goldens

The authored-write spawners only (those the marker guard refuses) — `e2e_worker_guard.rs`,
`e2e_dispatch_sync.rs`, `e2e_doctor_golden.rs`, and the ~30 marker-poisoned suites
enumerated at implementation from the CHR-044 case-note list. Read-only goldens are
untouched.

## Code impact (design-target selectors)

| Path | Change | Fix |
|---|---|---|
| `justfile` | `validate` recipe → layered `$DOCTRINE_BIN`→local→PATH resolution | #1 |
| `.doctrine/governance.md` | § orchestration — the `DOCTRINE_BIN`→coord-build precondition | #1 |
| `src/test_support.rs` | `under_worker_marker()` + `WORKER_MARKER_REL` const | #2 |
| `tests/common/mod.rs` | re-export `under_worker_marker` | #2 |
| `tests/e2e_worker_guard.rs`, `tests/e2e_dispatch_sync.rs`, `tests/e2e_doctor_golden.rs`, … | marker-guard early-return in authored-write goldens | #2 |

**The engine (`src/mcp_server/**`) is untouched.** `.doctrine/governance.md` is an
authored-tier orchestrator edit (already partly landed for this slice), not a
worker code-delta, so it is not a `design-target` selector.

## Verification alignment

- **VT-1 (#1, recipe resolution):** `just validate` invokes the binary named by
  `$DOCTRINE_BIN` when set (over PATH), the local `./target/debug/doctrine` when
  `$DOCTRINE_BIN` is unset but the local build exists, and bare `doctrine`
  otherwise. Exercised by pointing `$DOCTRINE_BIN` at a stub that records its argv.
- **VT-2 (#2):** an authored-write golden returns early (skips) when
  `DOCTRINE_WORKER` is set or the marker file is present, and runs normally when
  neither is — proving marker-gated, never masking a real regression on the main arm.

No engine VT: `src/` is unchanged, so the existing suites must stay green
untouched (the behaviour-preservation gate).

## Invariants & boundary conditions

- **POL-002.** No cargo/`./target` layout — nor any resolution policy — enters
  engine code. The engine is untouched; the *recipe* (project) owns resolution and
  the *governance rule* (project) owns the precondition. (DEC-003.)
- **Coverage preserved.** #2 skips only when marked; the gate clears the marker, so
  the goldens still run in the gate. The skip is strictly marker-gated — the main
  (unmarked) arm always runs them, so a real authored-write regression cannot hide.
- **Generic-host no-op.** A non-dogfooding host ignores `$DOCTRINE_BIN` (PATH tail is
  correct) and never sets the marker (its tests don't spawn doctrine authored writes).
- **Idempotent env.** `DOCTRINE_BIN` is published unconditionally when `current_exe()`
  resolves; on failure the recipe degrades to local→PATH (no hard dependency).

## Design decisions & residual open questions

- **DEC-003** — zero-engine layered recipe + `DOCTRINE_BIN`→coord-build governance
  precondition; rejected the engine-baked path *and* the engine-publish
  (`current_exe()` redundant-or-harmful) *and* the bare existence-check
  (non-Rust-phase hole).
- **OQ-1 (STD-001 single-sourcing).** `WORKER_MARKER_REL` (`.doctrine/state/dispatch/worker`)
  duplicates `marker.rs:114`'s `marker_path`. The dual-compilation seam (CHR-014)
  blocks a shared `crate::` const from `test_support.rs` (included into both the lib
  and the separate test crate). Resolve in `/plan`: either host the const in
  `test_support.rs` and have `marker.rs` reference *it*, or accept one documented
  carve-out with a cross-pointer comment (same class CHR-014 already tolerates).
