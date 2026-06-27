# Implementation Plan SL-163: check command proxy verb

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split along the design's two concern-clusters: **the verb** (a new
CLI surface that proxies project-declared check commands from the owned
`[verification]` contract) and **the corpus** (rewriting the shipped skills off
this repo's `just` convention onto that verb, plus a regression guard). The
boundary is also a dependency edge — the skills can only point at `doctrine
check gate` once it exists — so the phases are serial, not parallel.

## Sequencing & Rationale

### PHASE-01 — the verb (leaf + shell + wiring)

One phase, not three, despite spanning the pure leaf and the impure shell. The
alternative (a leaf-only PHASE-01) would land `resolve_check` / `CheckPlan` with
no caller until a later phase, tripping `dead_code` under `just gate` (clippy,
bins/lib only) and forcing a temporary `#[allow]` blanket. Folding the leaf and
its sole consumer into one phase keeps every intermediate state green without
that dance, and the unit (pure `resolve_check`) vs e2e (spawn + exit forwarding)
split still gives clean TDD layers inside the phase.

Build order inside the phase follows ADR-001 (leaf → command): the pure
`verify.rs` resolution first (red/green on `VT-1`/`VT-2`), then the
`commands/check.rs` shell and CLI/guard wiring (`VT-3` e2e). The
behaviour-preservation gate (INV-1) is load-bearing: the new `quick`/`commit`/
`gate` fields and `resolve_check` must not perturb the existing `command` (VT
base) parse or `verify::resolve` — the existing suites are the proof and stay
green unedited (`EX-4`, `VT-2`).

The proxy posture is deliberately the *opposite* of the existing
`coverage_verify::run_argv` (which pipes, captures, and caps for VT matching):
`check` **inherits** stdio for a live stream and runs **without** a timeout (an
interactive dev gate). The three sharp edges the codex pass surfaced are all
pinned by exit criteria: a configured-empty override is `Empty(kind)` → keyed
error rather than an empty spawn (`EX-2`, CR-F2); unconfigured `quick` is an
**owned** `Noop` (doctrine prints + exits 0, no host `echo`) so the default path
names no host binary (`EX-2`, CR-F3); and a signal-killed child forwards
`128+signo` rather than a flattened `1` (`EX-3`, CR-F5).

### PHASE-02 — the corpus (sweep + scrub + guard + re-embed)

Gated on PHASE-01 so no skill points at a non-existent verb. The sweep is **not**
a blind grep-replace (CR-F4): the four genuine phase/close-boundary instructions
(execute, close, audit, notes) become `doctrine check gate` (`EX-1`), while the
two `worktree` sites get *illustrative-example-only* edits (`EX-2`) — their
"project-provided, never a hardcoded `cargo …`" and "orchestrator-supplied verify
command" semantics carry intentional caller-control that a fixed-gate rewrite
would erase. `VA-1` exists specifically to confirm that distinction in review,
because a test can see the string changed but not whether the *meaning* survived.

The uid scrub (`EX-3`) replaces the dangling `mem_019ec65ecbc7` citation with
portable prose describing the base==B mechanism. The new
`tests/e2e_no_shipped_couplings.rs` guard (`VT-1`) makes the coupling
non-regressable — it rides the `e2e_no_baked_paths.rs` pattern, assembling its
needles from fragments so the guard file does not match its own scan. Re-embed +
reinstall (`EX-5`) is the materialisation step: `cargo build` re-bakes the
RustEmbed asset, `doctrine claude install` regenerates the gitignored `.agents/`;
`VH-1` eyeballs a scratch install as the client-surface check.

## Notes

- **No registry yet** — `[specs]` / `[requirements]` stay empty (v1); the slice's
  governance link (`governed_by POL-002`) and concerns (`SPEC-013`, `SPEC-010`)
  live on `slice-163.toml`, not here.
- **CR-F1 (typed-key parse-surface change) is deferred, not solved** here — see
  design R5. No client projects exist yet, so claiming `quick`/`commit`/`gate` as
  typed keys in the owned `[verification]` table breaks nothing today; revisit
  with a tolerant-parse migration note if/when external clients adopt.
- **Selectors** — the 13 `design-target` selectors recorded at design time are the
  conformance touch-set; `slice conformance` diffs git actuals against them at
  audit. PHASE-02 adds no files beyond those already declared.
