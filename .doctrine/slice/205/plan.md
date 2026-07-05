# Implementation Plan SL-205: Ambient memory surfacing via harness hooks

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, layered strictly per ADR-001 (leaf ← engine ← command) and driven
red/green/refactor. The design (`design.md`) is canon; this plan only sequences
its build. The neutral query seam lands first, the pure command core next, the
impure shell + wiring third, and the live battery last — each phase compiles and
is verifiable before the next depends on it.

## Sequencing & Rationale

**Why this order.** The dependency arrows point one way: the adapter consumes
`retrieve_rows`, which consumes the existing query core. Building bottom-up means
every phase is exercised by real tests before the layer above leans on it, and no
phase carries a stub it later revisits.

- **PHASE-01 (engine seam).** `SurfaceRow` + `retrieve_rows` in `retrieve.rs`,
  plus exposing `severity_rank` `pub(crate)`. This is the only touch to the
  retrieve module and it is *additive* — `retrieve_rows` composes the existing
  `load_query`/`query`/`check_retrievable` path (no parallel query; DRY). The
  behaviour-preservation gate (EX-4) is the proof the composition changed nothing.
  It lands first because both later code layers depend on the row type and the
  exposed ordinal.

- **PHASE-02 (pure helpers).** The adapter's decision core — `admits`,
  `dedup_diff`, `cap`, `format_block` over `Vec<SurfaceRow>` — as pure functions
  in `memory.rs`, unit-tested with synthetic rows (the jail's `decide`/`render`
  discipline). Isolating the pure logic here keeps PHASE-03's shell thin and lets
  the admission/format rules be pinned without any IO. The severity gate reuses
  PHASE-01's `severity_rank` — one severity scale, no string list.

- **PHASE-03 (shell + dispatch + guard).** The impure `run_surface` and the
  `MemoryCommand::Surface` variant, plus the `guard.rs` `Read` classification.
  This phase owns the **RV-254 penances** — the fail-open / IO-ordering contract
  the external inquisition confessed: emit swallows a stdout `Err` via the
  injectable `emit_surface` seam (never `?` — F-1); seen-set/log append only
  after a successful non-empty emit (F-3, INV-6); absent/empty `session_id`
  disables dedup for the fire (F-2). All of `run_surface` is VT-gated through
  synthetic stdin — no harness needed. At green, `doctrine memory surface` is a
  working, fully-tested command; only the wiring remains.

- **PHASE-04 (wiring + live).** The `hooks.json` matcher entries + the rebuild/
  install that re-embeds the RustEmbed `plugins/` root, then VA-1: the live
  battery a unit test cannot reach (a real `PreToolUse` fire, main-thread vs
  subagent). Deferred to last because it is the only non-unit-testable step and
  it depends on a working command to wire.

**The fail-open spine.** Every phase preserves the advisory contract: PHASE-01/02
are pure (cannot fail a hook); PHASE-03 makes `run_surface` exit 0 on every path
and emit only `additionalContext`; PHASE-04's live battery asserts no tool call is
ever blocked. A regression here is a blocker, not a nit.

## Notes

- **Test homes.** Unit tests live in `#[cfg(test)] mod tests` in the production
  file (retrieve.rs, memory.rs) per the codebase norm — hence the VT `test_file`
  points at the source, not a separate test file. PHASE-04's VT-1 targets the
  config artifact (`hooks.json`) directly (substring gate over the wiring).
- **`emit_surface` is a named contract.** PHASE-03 EX-3 names the injectable emit
  seam so VT-5 can drive it with a failing writer — the impl must use that seam,
  not an inline `writeln!`, or the F-1/F-3 penances are untestable.
- **No new gitignore/embed wiring.** `.doctrine/state/` is already gitignored
  wholesale (runtime tier) — the new `mem-surface*` files need no negation.
  `plugins/` is an existing RustEmbed root — PHASE-04's rebuild re-embeds it; no
  `flake.nix` `srcWithDist` graft is needed (hooks.json is not a new embed root).
  This is the design-target completeness the RV-254 reviewer cleared, restated as
  a build note so PHASE-04 doesn't rediscover it.
- **`emit_surface` gates the record (C3).** The seen-set/log append MUST be
  conditioned on `emit_surface`'s success result — not an inline `writeln!`
  followed by an unconditional append. Shape it so a test can assert "emit failed
  ⇒ zero uids recorded" (e.g. `emit_surface` returns whether a non-empty block was
  delivered, and the shell appends only then). Without this seam VT-5/VT-7 cannot
  be written.
- **PHASE-04 execution landmines (C2) — resolve at `/phase-plan`, not by
  surprise:**
  - **`DOCTRINE_BIN`.** The shipped hook calls `${DOCTRINE_BIN:-doctrine}`. In the
    jail the PATH `doctrine` (`~/.cargo/bin`, readonly) is the *old* binary with no
    `memory surface` verb. VA-1 must point `DOCTRINE_BIN` at the freshly-built
    `./target/debug/doctrine`, or install the new binary first — else the hook
    invokes a command that does not exist and (fail-open) silently does nothing,
    which would read as a false "no surfacing" failure.
  - **Tear down the prototype probes.** The two prototype hooks are still live in
    the gitignored `.claude/settings.local.json`. Remove them before VA-1, or the
    shipped path and the prototype double-fire and confound the observation.
  - **Registration may need a full restart.** `mem.fact.claude.reload-plugins-registers-pretooluse`
    is low-trust and contradicted on macOS; do not trust `/reload-plugins` alone —
    budget for a session restart (or fresh-session handoff) to observe live firing,
    and confirm the hook actually fires (a log line in `mem-surface.log`), never the
    "N hooks" count.
