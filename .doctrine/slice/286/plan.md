# Implementation Plan SL-286: Codex MCP registers at user level with an absolute path

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Five phases: one independent server change, then the codex user MCP leg built
pure-first, shell second, wired third, and proven end-to-end last.

```text
PHASE-01 rootless serve ──────────────────────────────┐
PHASE-02 codex_mcp pure plan ─▶ PHASE-03 execute shell ─▶ PHASE-04 wire + retire ─▶ PHASE-05 e2e + VH
```

## Sequencing & Rationale

- **PHASE-01 first** because it is independent of the leg and is the safety net
  for it: once a machine-wide entry exists, `serve --mcp` runs in every codex
  folder, so it should be harmless before anything can register it.
- **Pure before shell (PHASE-02 → 03).** The plan matrix is the bulk of the
  design's logic and is testable without I/O. The shell then only maps a settled
  plan to effects.
- **Constants move in PHASE-02.** `codex_mcp` takes ownership of the wrapper
  forms and their recogniser at once; `boot.rs` imports them back until PHASE-04
  deletes its arm. This avoids both a parallel implementation and a temporary
  `codex_mcp → boot` edge.
- **No `codex_mcp → boot` edge at all.** `boot` and `install` import
  `codex_mcp`, and the command-tier tangle baseline (`.doctrine/adr/001/layering.toml`
  `[tangle_baseline]`) may not grow. The design's code-impact note
  ("→ boot for `resolve_exec`/`Harness`") would create a cycle; callers pass
  `exec` and applicability in instead. This changes a dependency note, not the
  module boundary.
- **PHASE-04 starts with the test-environment guard** (RV-427 F-3): the moment
  install can run the leg, every e2e suite would otherwise reach the developer's
  real codex config.
- **PHASE-04 also retires the old arm** in the same phase that wires the new leg,
  so no commit leaves codex with both or neither registration path. Existing
  suites that asserted the project wrapper are updated there to keep the phase
  green; PHASE-05 adds the new binary-driven coverage.
- **Consent reuses `install::prompt_confirm`**, and the leg sits in
  `run_forward_steps` rather than inside a leg `install::regenerate` composes,
  so DEC-411 (explicit install only) holds by construction.
- **`codex_mcp` returns its own outcome type**; install maps it to its private
  `LegOutcome`, keeping the dependency one-way.
- **VH last.** The interactive-codex check (EVD-032 recurrence) and ASM-014 need
  the installed binary and a real daemon.

## Reconcile-time work (not a phase)

Carried to `/reconcile`, per design § Revision and reconcile:

- one REV (Revision) against SPEC-011: rewrite REQ-484; add a rootless
  `serve --mcp` requirement where `serve --mcp` is specified;
- `doctrine supersede`: DEC-323 and DEC-332 by DEC-406; DEC-325 by DEC-407;
- README: user-level entry and scope, consent and `-y`, project override slot,
  codex's per-folder trust prompt, codex CLI as a declared dependency, drop the
  `sh` dependency declaration.

## Notes

- RV-427 routed findings all carry `route = review`; none is
  `demonstrate`/`probe`/`control`, so none is transcribed as a criterion.
- Research advisory reports drift (design.md added, scope edited during design);
  expected post-design, not a stale premise. Design references re-grepped
  against the tree at plan time: all resolve.
