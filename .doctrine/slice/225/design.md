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

Target — **layered** resolution that **self-locates the fork-consistent binary at
gate-run time**, PATH kept as the generic-host tail:
```
validate:
  #!/usr/bin/env bash
  set -euo pipefail
  bin="${DOCTRINE_BIN:-}"                                            # 1. explicit override (optional)
  [ -z "$bin" ] && [ -x ./target/debug/doctrine ] && bin=./target/debug/doctrine   # 2. fork's own build (Rust phase)
  if [ -z "$bin" ]; then                                            # 3. coord build — git-derived, fork-consistent
    coord="$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"
    [ -x "$coord/target/debug/doctrine" ] && bin="$coord/target/debug/doctrine"
  fi
  if [ -z "$bin" ] && [ "$(git rev-parse --git-dir)" != "$(git rev-parse --git-common-dir)" ]; then
    echo "coord build missing: run 'cargo build' in the coord tree" >&2; exit 1   # 3a. refuse in a fork, don't fall to stale PATH
  fi
  [ -z "$bin" ] && bin=doctrine                                     # 4. PATH tail (correct for a generic host)
  "$bin" prompt check
  "$bin" doctor
```
Resolution order: **`$DOCTRINE_BIN` → fork's own `./target/debug/doctrine` →
git-derived coord build → PATH**, with a fork-gated refusal before the PATH tail.

- **Rung 3 is the load-bearing fix.** The gate runs with CWD = the **worker fork**,
  always a linked git worktree under the coord tree. `dirname $(git rev-parse
  --path-format=absolute --git-common-dir)` resolves the **coord root** from any
  linked fork (and the repo root from a main worktree — where rung 3 coincides with
  rung 2). So the gate finds the coord build — which is on `dispatch/<slice>` and
  carries *every* prior phase's rules — **without** any launch-time env, state file,
  or restart. This is what makes F-1's precondition temporally achievable (below).
- **Rung 2 before rung 3** so a Rust phase that rebuilt the fork validates against
  its *own* freshest binary (which knows the in-flight delta's rule change); a
  non-Rust phase (fork unbuilt) falls to the coord build — closing the
  SL-206-P14 residual the earlier draft *accepted* rather than solved.
- **Refusal (3a)** fires only inside a linked worktree (`--git-dir ≠
  --git-common-dir`) with no resolvable coord build — a diagnostic beats a silent
  stale-PATH false-red. A generic host (main worktree, dirs equal) never refuses;
  its PATH tail is correct.

### Why the engine-publish was rejected (adversarial reversal)

`.mcp.json` launches the server via `${DOCTRINE_BIN:-doctrine}`, and the gate is a
child of the server process, so it **already inherits** the server's `DOCTRINE_BIN`.
Publishing `DOCTRINE_BIN=current_exe()` from `worker_commit` is therefore redundant
when the var is set, and **harmful when it is unset** — `current_exe()` is then the
stale server binary and pinning it pre-empts the fall-through to the fork's fresh
local build. So no engine change; `worker_commit` is not in this slice's surface.
Full reasoning: DEC-003.

### The precondition — temporally achievable, run-time-verified (F-1)

An earlier draft demanded `$DOCTRINE_BIN` point at the coord build for a dispatch
session. That precondition is **not achievable**: `.mcp.json` launches the server
via `${DOCTRINE_BIN:-doctrine}` (boot.rs:549) and the client resolves that
expansion **at server launch** (boot.rs:544-549), *freezing* the server env — but
the coord worktree/build is created by `dispatch setup` (dispatch SKILL.md:13)
**after** the session is already running. A later operator `export` cannot reach
the running server, and the `worker_commit` gate is that server's child. So no
env-set-before-launch contract can name a binary that does not yet exist
(RV-291 F-1).

The self-locating rung 3 dissolves the cycle: the gate resolves the coord build
from **git at run time**, not from a launch-frozen variable. The only surviving
precondition is that **the coord tree is built** (`cargo build` in coord) — a
natural, already-expected part of dispatch setup, established *after* setup and
*before* phases run, and **verified at gate time** by rung 3's `-x` test with an
explicit refusal (3a) when absent. No restart, no rebind, no env-forwarding
dependency.

`$DOCTRINE_BIN` survives only as an **optional override** (rung 1) for an operator
who wants to force a specific binary; it is no longer required for correctness. The
governance note (`.doctrine/governance.md` § orchestration + the CLAUDE.md dispatch
precondition) softens accordingly: "set `DOCTRINE_BIN` or false-red" → "the gate
self-locates the coord build; `DOCTRINE_BIN` is an optional override." Resolving the
coord path stays **out of the engine** (that would re-bake cargo layout → POL-002);
`target/debug` lives in the project recipe, coord-location uses generic git plumbing.

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
| `justfile` | `validate` recipe → self-locating `$DOCTRINE_BIN`→fork-build→git-derived-coord→PATH resolution + fork-gated refusal | #1 |
| `.doctrine/governance.md` | § orchestration — soften the `DOCTRINE_BIN` note: gate self-locates; env var is now an optional override | #1 |
| `CLAUDE.md` | § "Dispatch precondition" — same softening (self-locating gate; `DOCTRINE_BIN` optional) | #1 |
| `src/test_support.rs` | `under_worker_marker()` + `WORKER_MARKER_REL` const | #2 |
| `tests/common/mod.rs` | re-export `under_worker_marker` | #2 |
| `tests/e2e_worker_guard.rs`, `tests/e2e_dispatch_sync.rs`, `tests/e2e_doctor_golden.rs`, … | marker-guard early-return in authored-write goldens | #2 |
| `tests/e2e_*` (new) | VT-1 discriminating gate fixture + VT-1b refusal + VT-1c precedence unit | #1 |

**The engine (`src/mcp_server/**`) is untouched.** `.doctrine/governance.md` and
`CLAUDE.md` are authored-tier orchestrator/prose edits, not worker code-deltas, so
they are not `design-target` selectors.

## Verification alignment

- **VT-1 (#1, discriminating end-to-end — the scope's promised proof).** A fixture
  builds a coord tree whose binary knows a rule the **stale PATH `doctrine` does
  not** (e.g. a role/allowlist/check the coord build carries), forks a worker off it
  (non-Rust delta, so the fork itself is *not* rebuilt), and drives the **real
  `worker_commit` gate seam**. The gate must go **green** — proving `just validate`
  resolved the git-derived coord build (rung 3) and `doctrine doctor` saw the rule —
  where the same gate run against a stale PATH binary would `commit-gate-red`. This
  proves *binary identity and lifecycle reachability*, not argv shape (RV-291 F-3).
- **VT-1b (#1, refusal / precondition negative).** Inside a linked fork with **no
  resolvable coord build**, `just validate` exits non-zero with the coord-build
  diagnostic (rung 3a) rather than silently falling through to the stale PATH binary.
- **VT-1c (#1, recipe precedence unit — the narrow stub).** The former argv-stub
  test is retained *only* as a unit proof of rung order: `$DOCTRINE_BIN` (set) wins
  over the fork build, which wins over PATH. It backstops precedence; it does **not**
  stand in for VT-1's end-to-end elimination of ISS-218.
- **VT-2 (#2):** an authored-write golden returns early (skips) when
  `DOCTRINE_WORKER` is set or the marker file is present, and runs normally when
  neither is — proving marker-gated, never masking a real regression on the main arm.

No engine VT: `src/` is unchanged, so the existing suites must stay green
untouched (the behaviour-preservation gate).

## Invariants & boundary conditions

- **POL-002.** No cargo/`./target` layout — nor any binary-resolution policy —
  enters engine code. The engine is untouched; the *recipe* (project) owns
  resolution: `target/debug` is project (cargo) knowledge in the project justfile,
  and coord-location uses **generic git plumbing** (`--git-common-dir`), never the
  dispatch `.worktrees` layout. (DEC-003.)
- **Gate resolution is run-time, not launch-time.** `worker_commit` publishes and
  mutates **no** environment; the gate child inherits only the server env fixed at
  launch, and the recipe self-locates the binary at gate-run time (rungs 2–3). No
  `current_exe()` publish, no `DOCTRINE_BIN` write — the rejected engine-publish
  leaves no live invariant here (RV-291 F-2).
- **Own-`target/` assumption.** Rung 3 names `$coord/target/debug/doctrine` because
  each worktree builds into its **own in-tree `target/`** (AGENTS.md — no shared
  `CARGO_TARGET_DIR`). Were that ever redirected to a shared target, rung 3 would
  coincide with rung 2 (the fork's build) — degraded but not incorrect.
- **Coverage preserved.** #2 skips only when marked; the gate clears the marker, so
  the goldens still run in the gate. The skip is strictly marker-gated — the main
  (unmarked) arm always runs them, so a real authored-write regression cannot hide.
- **Generic-host no-op.** A non-dogfooding host is a main worktree (`--git-dir =
  --git-common-dir`), so rung 3a never refuses and the PATH tail (rung 4) — correct
  there — is reached; it never sets the #2 marker either.

## Design decisions & residual open questions

- **DEC-003** — zero-engine **self-locating** recipe: the gate resolves the
  fork-consistent binary from git at run time (`$DOCTRINE_BIN` override → fork build
  → git-derived coord build → PATH), with a fork-gated refusal. Rejected the
  engine-baked path, the engine-publish (`current_exe()` redundant-or-harmful), the
  bare existence-check (non-Rust-phase hole), **and** the launch-frozen
  `DOCTRINE_BIN`→coord-build *precondition* (RV-291 F-1: not temporally achievable —
  server env freezes at launch, coord build is created after). `DOCTRINE_BIN` demotes
  to an optional override; the only precondition is a built coord tree, verified at
  gate time.
- **OQ-1 (STD-001 single-sourcing).** `WORKER_MARKER_REL` (`.doctrine/state/dispatch/worker`)
  duplicates `marker.rs:114`'s `marker_path`. The dual-compilation seam (CHR-014)
  blocks a shared `crate::` const from `test_support.rs` (included into both the lib
  and the separate test crate). Resolve in `/plan`: either host the const in
  `test_support.rs` and have `marker.rs` reference *it*, or accept one documented
  carve-out with a cross-pointer comment (same class CHR-014 already tolerates).
