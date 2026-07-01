# Subprocess-arm Seatbelt confinement (macOS jail parity)

## Context

Discharges the **subprocess (pi/codex) residual of IMP-045** that SL-183 scoped
OUT as a Non-Goal + Follow-Up. SL-183 delivered the macOS Seatbelt
write-confinement floor for the **claude PreToolUse arm** — the pure
`seatbelt_profile` / `sandbox_exec_argv` builders, the `Seatbelt` `Jailer`, and
the `Decision`/`Target`/policy/funnel machinery in `src/worktree/jail.rs`, wired
to the claude arm via `src/worktree/pretooluse.rs` (`materialize_seatbelt_profile`
writes the `.sb` body to disk before `sandbox-exec` runs).

The **subprocess arm confines differently**: `scripts/pi-spawn-confined.sh`
wraps the worker `pi`/`codex` exec in a **nested bwrap** directly in shell — it
does NOT route through `jail.rs`. On macOS bwrap is absent, so the subprocess
arm is **unconfined today** (no parity floor). This slice closes that gap by
giving the subprocess arm a Seatbelt confinement on macOS, reusing SL-183's
builders — the fork is the launcher/argv shell (subprocess spawn vs the claude
PreToolUse hook rewrite).

Governed by ADR-008 (project-local jail build isolation + worker confinement)
and ADR-006 (worktree posture), like SL-183. POL-002 (platform independence) is
the load-bearing intent: parity, not a mac-shaped special case.

## Scope & Objectives

1. **Subprocess-arm Seatbelt confinement on macOS** — the worker exec runs under
   `sandbox-exec -f <profile>` with a write-floor identical to the claude arm:
   allow-default, deny `file-write*`, re-allow writes under the worker worktree
   (+ validated `extra_rw`). Bwrap parity: rw-bind only the worker's own tree,
   ro everything else.
2. **Reuse SL-183's pure machinery unchanged** — `seatbelt_profile`,
   `sandbox_exec_argv`, `resolve_inputs`/`ResolveEnv`, `Decision`/`Target`,
   `validate_policy`, `select_jailer`. No parallel builder. The change is the
   *launcher seam* only.
3. **Profile materialization on the subprocess path** — the `.sb` body must exist
   on disk before `sandbox-exec` runs (the command-tier obligation SL-183 met via
   `materialize_seatbelt_profile`; the subprocess arm needs the equivalent hook —
   see OQ-1 for where it lives).
4. **Linux-side development, deferred mac enforcement** — the pure layer
   (profile/argv/policy) is platform-agnostic and TDD'd red/green/refactor here on
   Linux. Actual enforcement (does `sandbox-exec` block a write) is a mac-only
   gate (see Verification).

## Non-Goals

- **Linux subprocess arm** — bwrap confinement via `pi-spawn-confined.sh` already
  ships; not re-touched except at the shared seam if OQ-1 lands in Rust.
- **The claude PreToolUse arm** — SL-183, done. Only reused, not modified.
- **Reads / confidentiality, network egress wall, the `launchd`/`launchctl
  submit` IPC residual** — inherited Non-Goals from SL-183; Seatbelt leaves reads
  open and the network knob stays coarse.
- **New probe battery** — SL-183 already ran the falsification-first
  `sandbox-exec` probe (nesting, canonicalization, child inheritance, escape
  vectors). Reuse its findings; re-probe only if the subprocess launcher shape
  changes an assumption (e.g. spawn nesting depth).
- **Self-commit / funnel / `.git` posture** — unchanged; the worker gitdir sits
  outside the worktree → write-denied by the floor, identical to SL-183/SL-182.

## Affected surface

- `src/worktree/jail.rs` — the `Seatbelt` `Jailer` + builders (reused; touched
  only if the subprocess launcher needs a new entry point).
- `scripts/pi-spawn-confined.sh` — the subprocess launcher; the macOS branch (or
  a sibling `pi-spawn-confined-macos.sh` / os-branch) that calls `sandbox-exec`
  instead of `bwrap`.
- Whatever Rust subcommand the shell calls to emit the argv/profile (OQ-1) — e.g.
  a `doctrine worktree ...` seam analogous to the claude `pretooluse` subcommand.
- `validate_policy` — unchanged; the platform-agnostic parity proof.

## Risks / assumptions / open questions

- **OQ-1 (the seam — for `/design`):** the subprocess arm confines in *shell*
  today, not through `jail.rs`. Two shapes: (a) expose a doctrine subcommand that
  emits the `sandbox-exec` argv + materializes the `.sb` profile, which the spawn
  script invokes (keeps confinement in shell, mac parity for the existing path);
  or (b) move subprocess confinement into Rust. (a) rides the existing shell seam
  with least disruption; (b) unifies the two arms but is larger. `/design`
  decides. Reuse-first (CLAUDE.md, ADR-001) leans (a).
- **OQ-2:** profile-write location + lifecycle on the subprocess path — who writes
  `<wt>/.tmp/jail.sb`, when, and GC. SL-183's `materialize_seatbelt_profile` is
  the claude-arm reference; the subprocess arm needs the equivalent, wherever
  OQ-1 lands it. See [[mem_019f1d4568797f73962303244c9838c8]] (the `.sb` must be
  written by the command tier — bwrap parity doesn't cover it).
- **RISK — cfg-rot:** the `Seatbelt` `Jailer` and any new launcher glue are behind
  `#[cfg(target_os = "macos")]`; Linux `cargo build` does not compile or
  type-check that branch. Mitigation: wire `cargo check --target
  aarch64-apple-darwin` into the plan (needs the target installed; type-checks
  without a mac). Do not treat Linux-green as complete.
- **ASSUMPTION:** SL-183's probe findings (nesting, canonicalization, child
  inheritance) hold for the subprocess spawn shape. Re-probe only if the launcher
  changes the nesting/exec assumptions.
- **OQ-3 / IDE-025 adjacency (for `/design`):** [[IDE-025]] proposes a
  selector-sourced write-allowlist *jail mode*. It stays a separate slice (distinct
  config-surface feature, currently an unscoped idea) — but its mechanism IS
  hostable on this arm, contrary to IDE-025's own "mount-ns can't express glob
  sets" framing. Two corrections that bear on the schema:
  - **Seatbelt CAN express a glob allowlist at the profile floor.** SL-183's
    `seatbelt_profile` already emits a regex path filter
    (`(allow file-write* (require-all (subpath …) (regex XCRUN_DB_REGEX)))`,
    `jail.rs:506`). A selector allowlist is the same shape per-selector:
    `(allow file-write* (require-all (subpath WT) (regex <glob→regex>)))`. It even
    covers **new-file creation** under a glob (file-write* matches the new path
    against the regex — no pre-existing inode, unlike a bwrap bind). And it governs
    **every** write syscall (Bash included), so it is *stronger* than IDE-025's
    claude-arm form (a per-path Edit/Write-tool predicate a `Bash` write can slip).
  - **bwrap** can approximate via per-file rw-binds of the point-in-time set, but
    degrades: new-file-under-glob needs parent-dir write (subtree), and
    atomic-rename saves cross the file's own bind mount → `EXDEV`. Partial, brittle.
  Consequence for THIS slice: the shared seam is not just the per-worker **policy
  schema** but the **profile builder** itself. Design the seatbelt policy this slice
  stamps so a future `mode = "selector-strict"` / `write_allowlist` field can drive
  a per-selector regex allow-rule — leave the room, don't implement it here.

## Verification / closure intent

- **VT (Linux, here):** unit tests on the reused pure layer for the subprocess
  path — `sandbox_exec_argv` shape, profile body, policy validation, `extra_rw`
  round-trip, `select_jailer(Seatbelt)`. Red/green/refactor.
- **VA:** conformance that no parallel builder was introduced — the subprocess
  arm rides SL-183's `seatbelt_profile`/`sandbox_exec_argv`/policy unchanged.
- **VH (mac-only, deferred):** on a real mac — profile materializes, worker exec
  runs under `sandbox-exec`, a write outside the worktree is denied, a write
  inside succeeds, child processes inherit the sandbox. This is the enforcement
  gate; it CANNOT be discharged on Linux and closes only after mac verification.

## Summary

## Follow-Ups
