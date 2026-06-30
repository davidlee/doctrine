# macOS Seatbelt write-confinement arm

## Context

SL-182 installs the Linux/bwrap write-containment floor for claude
`isolation:worktree` subagents. On macOS that arm is today a fail-closed stub
(`deny "bwrap-unavailable"`, SL-182 §5.5 / POL-002): no `bwrap`, so confined
fan-out simply refuses. This slice discharges **IMP-045** for the **claude arm**
— a *real* write-containment arm on macOS via Seatbelt (`sandbox-exec` + a
generated `.sb` profile), reusing the **same** `Decision`/`Target`/policy/funnel
so only the argv/profile builder forks. It closes the macOS gap RFC-012 named as
unscoped, and is the cross-platform completion of the per-arming policy floor.

Note IMP-045's original framing was the *subprocess* jail seam (from SL-056); the
design seed re-aims it at **SL-182 `jail.rs` parity** for the claude PreToolUse
arm. The full design-ahead brief lives in
[`seatbelt-seam-brief.md`](./seatbelt-seam-brief.md) (relocated from gitignored
scratch) — `/design` consumes it.

## Scope & Objectives

1. **Seatbelt profile builder.** `seatbelt_profile(policy) -> String` —
   *allow-default, deny `file-write*`, re-allow writes under the worktree* (and
   per validated `extra_rw`). Inverse of bwrap's mount-namespace logic; a write
   floor, **not** SBPL default-deny (the footgun is sidestepped).
2. **`sandbox-exec` argv builder.** `sandbox_exec_argv(wt, policy)` — opaque
   base64 command body, `-D` **realpath** params (never string-splice), children
   inherit the sandbox.
3. **Single `Jailer` seam.** Reuse all of SL-182's `jail.rs`
   (`resolve_target`, `decide_bash`, `decide_write`, `pathcheck`, `opaque_wrap`,
   `validate_policy`) unchanged; fork **only** the argv/profile builder behind a
   trait or runtime-os branch. ADR-001 layering intact; no parallel pipeline.
4. **No-tmpfs resolution.** Seatbelt cannot mount a private `/tmp`. Default:
   `TMPDIR=<wt>/.tmp` + deny `/private/tmp` (collapses scratch into the rw scope,
   GC'd with the tree); `mktemp -d` + explicit teardown GC is the fallback for
   hardcoded-`/tmp` tooling.
5. **Network knob.** `network=false ⇒ (deny network*)` with a stated coarseness
   caveat (syscall-deny, not iface removal). Egress remains a non-goal.
6. **Probe-first phase (gating).** A disposable `sandbox-exec` shell probe
   **before any Rust**, falsification-first (mirrors RSK-014/SL-182): nesting,
   floor+canonicalization, child inheritance, the 11-vector escape battery + the
   macOS-only `launchctl submit`/`at` vectors, `-D` round-trip, `updatedInput`
   honoured. Abort/degrade contract: nesting-refused or canonicalization-leaky ⇒
   macOS arm = `deny worktree-subagent Bash`, **never** unwrapped pass-through.

## Non-Goals

- Reads / confidentiality (Seatbelt leaves reads open by design — parity: reads
  are out of scope on both arms).
- Network egress as a hard wall (the `network` knob is coarse; egress wall is
  IPC/egress territory, not this floor).
- The `launchd` IPC residual (`launchctl submit` mach-service) — *measured* by
  the probe, then assigned to the IPC/egress wall (macOS sibling of the
  postgres/nix-daemon reachable-peer residual, RSK-014). Closing it would mean
  mach-lookup default-deny — the rabbit hole this floor avoids.
- The subprocess (pi/codex) seatbelt backend — IMP-045's original axis; can adopt
  the same `seatbelt_profile` builder later but is not this slice.
- Re-opening `.git` posture / self-commit / funnel (inherited from SL-182: the
  worktree gitdir at `<main>/.git/worktrees/<name>` is outside `wt` → write-denied
  by the floor; no-self-commit consequence and funnel are identical).

## Affected surface

- `src/.../jail.rs` (SL-182's module) — the `Jailer` seam + profile/argv fork.
- The worktree `pretooluse` subcommand dispatch (os-branch selection).
- `validate_policy` — **unchanged** (platform-agnostic; reused as the parity
  proof).
- A disposable probe harness (shell, `.doctrine/backlog/.../probe/` or slice
  scratch) — not shipped Rust.

## Risks / assumptions / open questions

- **OQ-mac1 — nesting vs Claude's own Seatbelt** (top risk, hard gate).
  **CLOSED (RSK-014 H2 pass 2, 2026-07-01): SUPPORTED.** Nested `sandbox-exec`
  composes inside a real `isolation:worktree` subagent under bypassPermissions;
  every external vector denied, `updatedInput` honoured. Probe gate discharged.
- **OQ-mac2 — launchd IPC residual** — **MEASURED-LOW:** `launchctl submit` / `at`
  denied by Seatbelt default. Assigned to the IPC/egress wall (non-goal), not open.
- **OQ-mac4 — second temp surface (`/var/folders/$USER/T`)** — **RESOLVED:** narrow
  `xcrun_db` cache-file allow; rest denied (documented cross-subagent caveat).
- **Canonicalization footgun (INV-5 twin):** macOS aliases `/tmp→/private/tmp`,
  `/var→/private/var`, `/etc→/private/etc`; `subpath` matches the *resolved* path.
  Feed realpaths into every `-D` param; prove symlink/hardlink containment in the
  battery, don't assume.
- **Vanish risk:** Seatbelt deprecated since ~10.10, SBPL undocumented; mitigated
  by Anthropic's own sandbox-runtime + system `.sb` profiles depending on it. Low,
  not zero.
- **Hard dependency on SL-182.** Parity reuses SL-182's `jail.rs` seams; SL-182 is
  **`ready`** (design locked) but **not yet implemented** — `jail.rs` doesn't exist
  on disk. SL-182 already upstreamed the cross-arm `Jailer` seam + capability-as-data
  `select_jailer` fork point; SL-183 slots the Seatbelt argv/profile builder in
  as-is (OQ-mac3 resolved, no SL-182 refactor). **Implementation blocked until
  SL-182 lands.** Tracked as `needs SL-182`.
- **Execution host:** the probe + verification require a macOS machine — cannot
  run inside the Linux/bwrap jail. The operator ships this slice to macOS to
  execute.

## Verification / closure intent

- Probe pass criterion (identical to RSK-014 H1): every external escape vector
  `denied` (`Operation not permitted` / sandbox violation), worktree writable,
  `sandbox-exec` wrapper confirmed applied via `updatedInput`.
- `validate_policy` behaviour-preserved (shared, unchanged) — SL-182's suites
  stay green.
- End-to-end: a jailed claude worktree subagent on macOS contained at the OS
  floor, funnel import unchanged.
- Degrade contract proven: nesting-refused ⇒ explicit `deny`, asserted.

## Follow-Ups

- Subprocess (pi/codex) seatbelt backend reusing `seatbelt_profile` (IMP-045's
  original axis).
- The `launchd` IPC residual, if/when an IPC/egress wall is built.
