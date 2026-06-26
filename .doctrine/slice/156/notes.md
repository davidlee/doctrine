# SL-156 — implementation notes

Durable execution decisions (survive the disposable handover). Narrative only;
queried data lives in the TOMLs.

## Execution posture (set PHASE-01)

- **Solo on one fork.** All phases run in `.worktrees/SL-156` (branch
  `slice/SL-156-cargo-isolation`), forked off edge. Build/test use an isolated
  `CARGO_TARGET_DIR=/home/david/.cargo/doctrine-target-jail/wt/slice/SL-156-cargo-isolation`
  so they don't thrash SL-154's shared-target builds (live pre-fix mechanism, R5).
  Use the fork-built binary `$CARGO_TARGET_DIR/debug/doctrine` for doctrine CLI
  (RO jail binary is stale per AGENTS.md).
- **Landing is deferred to slice close** — one fork carries all four phases; land
  via `git merge --no-ff` (never squash) at close, not per-phase. The /execute
  skill's per-phase land/gc framing assumes single-phase-per-fork; overridden here.

## Scoped baseline (accepted by user)

- Full `just check` cannot go green on edge: SL-154's `d82ec4b8` (commit-boundaries
  conformance check) leaves **22 `e2e_dispatch_candidate` tests red** vs stale
  fixtures. Pre-existing, SL-154's domain, **out of SL-156 scope**. SL-156's gate
  is its own suites (worktree fork/coordinate/gc/stamp/create_fork — all green).
  Do not try to fix the 22; flag to SL-154.
- Fixed one unrelated trunk-red on the way: relation-contiguity in `slice-156.toml`
  (`085d351e` appended `related→REV-011` after the `references` rows) — reordered,
  committed on edge `f6833c68`.
- `src/ledger.rs` shows persistent fmt drift after every `just check` (edge has
  committed-unformatted SL-154 code; `just check` runs `cargo fmt` in-place). SL-154's
  file — **leave it**, path-limit commits.

## Verification model (PHASE-01, generalises)

- The flake change (and any flake change) is **launch-time** (R5): inert in-session.
  In-session proof is the `.env_remove` e2e simulation; the live mechanism + flake
  eval are **VH**, discharged at reconcile post-relaunch. nix is absent inside the
  jail (`mem.fact.jail.nix-absent-no-flake-eval`), so flake eval cannot run here at all.
- VT-1 is honestly **fallback-path/simulation only** (codex 019f0214 rescope); VH-1
  is the sole proof of final `<wt>/target` semantics. See plan.toml / design §9 / §8 R5.
