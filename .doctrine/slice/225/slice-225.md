# Funnel false-red elimination

## Context

Descends from RFC-016 (zero-rescue dispatch), **Cluster 1 / move B** — stop the
funnel showing the worker a *false red*: a green delta reported as a failure the
worker cannot distinguish from its own damage. Two of the RFC-011 case-notes' top
burners are exactly this class, and both are reds that **survive Cluster 2** —
read-verbs / no-shell-git do not touch how the gate resolves its binary or how the
worker's own test run trips the marker:

- **worker_commit gate false-red (6+ repros, ISS-218).** The gate belt's
  `just validate` → `doctrine doctor` runs the stale PATH `~/.cargo/bin/doctrine`
  (built from edge/main, RO in the jail), which lags `dispatch/<slice>` content.
  A phase that adds a role / allowlist / check false-reds `commit-gate-red` — even
  a *pure-JS* delta — against a correct fork.
- **e2e authored-write goldens fail under the worker marker (7+ repros,
  CHR-044).** ~30 e2e binaries spawn the CLI for authored writes, which the
  worker-mode guard correctly refuses under the marker. The server-side gate
  already clears the marker for its own run (SL-199 F2, `worker_commit.rs:147`),
  so the residual burn is the **worker agent's own** `cargo test` / `just check`
  (run to check its work), which hits the marker and reads the refusal as
  own-delta red.

Both burn tokens as diagnosis of noise. The coord-worktree reverse-diff footgun
(case-note #2, a *different* seam and not a false-red) was considered here and
**routed to Cluster 2 as ISS-234** — its durable fix is read-verbs + no-shell-git,
which an interim auto-sync in this slice would only pre-empt then get retired.

## Scope & Objectives

1. **Gate resolves a fork-consistent `doctrine`, not stale `$PATH`.** This repo's
   `just validate` (reached by the `check commit` gate → `DEFAULT_COMMIT`) resolves
   the `doctrine` it shells in layered order — `${DOCTRINE_BIN}` → local
   `./target/debug/doctrine` → PATH — so `doctrine doctor` matches the fork's actual
   rules, not edge/main's (ISS-218; IMP-270 dup-closed here). **Zero-engine** — the
   fix is the `justfile` recipe plus a `DOCTRINE_BIN`→coord-build precondition in
   `.doctrine/governance.md`; `worker_commit`/the engine are untouched (DEC-003).
2. **Marker-aware skip for authored-write e2e goldens.** The e2e goldens that drive
   authored writes skip when the worker marker is present, so the worker agent's
   own suite run reflects delta health. This composes with the existing gate
   marker-clear: marker present (worker's manual run) → skip; the gate clears the
   marker → the goldens still run → **coverage preserved in the gate** (CHR-044).

Closure intent: a worker in a marked fork sees its own `cargo test` and the
server-side `worker_commit` gate both green on a correct delta — no environmental
red, no recalled "this red is a rig artifact" idiom.

## Non-Goals

- **Coord-worktree reverse-diff (case-note #2 → ISS-234).** Re-homed to RFC-016
  Cluster 2: the funnel is deliberately working-tree-free (ADR-012 / design §B) and
  the durable fix is read-verbs + no-shell-git-in-funnel, not an interim auto-sync.
- Refusal legibility / plan-time selector lint (move C) — that is **SL-224**.
- The `dispatch next` state machine, no-shell-git prohibition, and the memory-blind
  benchmark — RFC-016 Cluster 2 / move A.
- The `CARGO_MANIFEST_DIR`-baked stale-test-binary flake
  ([[mem_019f376e2f6b7571af71290f8ea994c2]]) — a *separate* false-red class
  (env!-baked absolute path from a reaped fork), SL-206-deferred to its PHASE-13.
  Distinct from the stale-PATH-binary mechanism this slice fixes.
- Reworking the worker-mode guard's semantics — the guard is correct; we stop
  *conflating* its refusal with delta damage.
- ISS-219 (295k-char transcript in the refusal) and the architecture-layering
  ratchet-red handoff (IMP-293) — adjacent false-red ergonomics, not in this cut.

## Affected surface (see design.md code-impact for the locked touch-set)

- `justfile` — `validate` recipe layered resolution (#1).
- `.doctrine/governance.md` — § orchestration `DOCTRINE_BIN` precondition (#1, authored tier).
- `src/test_support.rs` + `tests/common/mod.rs` — `under_worker_marker()` helper (#2).
- `tests/e2e_*.rs` authored-write goldens (`e2e_worker_guard.rs`,
  `e2e_dispatch_sync.rs`, `e2e_doctor_golden.rs`, and the ~30 marker-poisoned
  suites) — marker-aware skip (#2).
- **Engine (`src/mcp_server/**`) untouched** — the zero-engine reversal (DEC-003).

## Risks / Assumptions / Open questions

- **A:** the server can resolve the workspace / coord binary for #1 — the coord
  tree's `./target/debug/doctrine`, `current_exe()` of the MCP server, or a config
  path. Confirm the resolution seam and its jail-safety in `/design`.
- **OQ:** #1 — resolve the coord tree's `./target/debug/doctrine`, or reuse the
  MCP server's own binary (`current_exe`)? The two differ when the server predates
  an in-flight binary change; the coord build is the more current.
- **OQ:** #2 — skip the goldens under marker (test-side), or route worker_commit's
  gate to exclude authored-write goldens inside a marked fork (gate-side)? The
  test-side skip composes with the existing gate marker-clear for free; confirm.
- **Risk:** #2 marker-skips must be strictly marker-gated — never mask a *real*
  authored-write regression on the main (unmarked) arm.

## Verification / closure intent

VT per fix: a gate test showing green on a conformance-rule-changing fork under the
workspace binary (where the stale PATH binary would red); a marked-fork run of an
authored-write golden that skips under the marker yet still runs (green) once the
marker is cleared. Closes when both land green and the stale-PATH false-red is
demonstrably gone.

## Follow-Ups

- ISS-234 (coord-worktree reverse-diff) rides RFC-016 Cluster 2.
- Adjacent, not in scope: ISS-219 (refusal transcript cap), IMP-293 (ratchet-red
  handoff signal), IDE-028 (phase-status push), and the `CARGO_MANIFEST_DIR` flake.
