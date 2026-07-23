# SL-225 — Funnel false-red elimination — Design

> Cluster 1 / move B of RFC-016. Two false-reds — a green worker delta reported
> as a failure it cannot distinguish from its own damage. Both are **project-tier**
> dogfooding artifacts (POL-002): the engine ships one thin generic affordance and
> is otherwise untouched. See DEC-003.

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

## Fix #1 — gate resolves a fork-consistent `doctrine`

### Current vs target

Current (`justfile:28`):
```
validate:
  doctrine prompt check      # → PATH ~/.cargo/bin/doctrine (stale)
  doctrine doctor
```

Target — **layered** resolution, project-tier, PATH kept as the generic-host tail:
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

### Why `$DOCTRINE_BIN` must be the first rung (not just the local build)

`validate` runs **before** the gate's `build` leg (`check: … validate test build`),
so on a **non-Rust phase** the fork has no freshly-built `./target/debug/doctrine`
and the existence-check alone silently falls back to the stale PATH binary —
re-masking the very P14 repro. `$DOCTRINE_BIN` points at the coord/orchestrator
build (on `dispatch/<slice>`, carries the earlier phase's rules, guaranteed present)
and needs no fork build. See DEC-003 "why not the alternatives".

### Engine affordance (the one platform change)

`worker_commit`'s gate spawn (`run_commit_gate`, `worker_commit.rs:151`) exports
the running server's own binary path so the recipe's first rung is reliably set:

```rust
// current_exe() = the doctrine binary serving this MCP session. In dogfooding the
// operator serves MCP from the coord-built binary (AGENTS.md), so this carries the
// in-flight rules. A generic host's gate simply ignores $DOCTRINE_BIN. POL-002:
// we publish "the doctrine I am", never a cargo path.
let self_bin = std::env::current_exe().ok();
// …
let mut cmd = std::process::Command::new(program);
cmd.args(rest).current_dir(dir).env_remove("DOCTRINE_WORKER");
if let Some(bin) = &self_bin {
    cmd.env("DOCTRINE_BIN", bin);
}
```

This is the sole `.rs` edit for #1 — an env publish, no resolution *policy* in the
engine. `current_exe()` correctness rides the coord-served-MCP convention → ASM
(DEC-003).

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
| `src/mcp_server/worker_commit.rs` | publish `DOCTRINE_BIN=current_exe()` into the gate env (env only) | #1 |
| `src/test_support.rs` | `under_worker_marker()` + `WORKER_MARKER_REL` const | #2 |
| `tests/common/mod.rs` | re-export `under_worker_marker` | #2 |
| `tests/e2e_worker_guard.rs`, `tests/e2e_dispatch_sync.rs`, `tests/e2e_doctor_golden.rs`, … | marker-guard early-return in authored-write goldens | #2 |

## Verification alignment

- **VT-1 (#1, engine):** a `run_commit_gate` unit/e2e test asserts the spawned gate
  child sees `DOCTRINE_BIN == current_exe()` in its environment.
- **VT-2 (#1, recipe):** with `DOCTRINE_BIN` set to a fork/coord binary that knows a
  rule the PATH binary doesn't, `just validate` is green where bare `doctrine` reds.
  (Behavioural: the layered recipe honours `$DOCTRINE_BIN` over PATH.)
- **VT-3 (#2):** an authored-write golden returns early (skips) when
  `DOCTRINE_WORKER` is set or the marker file is present, and runs normally when
  neither is — proving marker-gated, never masking a real regression on the main arm.

## Invariants & boundary conditions

- **POL-002.** No cargo/`./target` layout enters engine code. The engine publishes a
  value (`current_exe()`); the *recipe* (project) owns resolution. (DEC-003.)
- **Coverage preserved.** #2 skips only when marked; the gate clears the marker, so
  the goldens still run in the gate. The skip is strictly marker-gated — the main
  (unmarked) arm always runs them, so a real authored-write regression cannot hide.
- **Generic-host no-op.** A non-dogfooding host ignores `$DOCTRINE_BIN` (PATH tail is
  correct) and never sets the marker (its tests don't spawn doctrine authored writes).
- **Idempotent env.** `DOCTRINE_BIN` is published unconditionally when `current_exe()`
  resolves; on failure the recipe degrades to local→PATH (no hard dependency).

## Design decisions & residual open questions

- **DEC-003** — layered recipe + engine publishes `DOCTRINE_BIN`; rejected the
  engine-baked path and the zero-engine existence-check (non-Rust-phase hole).
- **OQ-1 (STD-001 single-sourcing).** `WORKER_MARKER_REL` (`.doctrine/state/dispatch/worker`)
  duplicates `marker.rs:114`'s `marker_path`. The dual-compilation seam (CHR-014)
  blocks a shared `crate::` const from `test_support.rs` (included into both the lib
  and the separate test crate). Resolve in `/plan`: either host the const in
  `test_support.rs` and have `marker.rs` reference *it*, or accept one documented
  carve-out with a cross-pointer comment (same class CHR-014 already tolerates).
