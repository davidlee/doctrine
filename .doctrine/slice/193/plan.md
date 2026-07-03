# Implementation Plan SL-193: Exposed-slot self-replaces projection

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases. The design (locked, REV-019 mechanism; SPEC-023 REQ-322/323/329)
puts the entire fix in the projector `src/install.rs` — the engine
`src/hymns.rs` is unchanged (D4). PHASE-01 makes the producer correct and gives
it a live call site; PHASE-02 proves the resolved behaviour, verifies corpus-wide
legality, and reconciles the 5 existing orphan twins.

## Sequencing & Rationale

### Why PHASE-01 is one atomic install.rs change

The obvious split — "emit the sidecar" then "wire the producer" — is **not
available**. Three things are coupled by the compile gate (design F1):

1. `project_starters` is `#[expect(dead_code)]`. Removing that attr requires a
   **non-test** caller; a `#[cfg(test)]` reference does not satisfy `dead_code`
   under a plain `cargo build`. So the fn goes live only when wired into
   `run_forward_steps`.
2. `HymnsSection.expose` carries its **own** `#[expect(dead_code)]`
   (install.rs:117). It goes live only when `embedded_expose_set` reads it —
   which happens in the same wiring.
3. `Cargo.toml` sets `warnings = "deny"`, so an **unfulfilled** lint expectation
   is a hard compile error, not a warning. Leaving either attr in place after its
   target became live fails the build; removing either before its target is live
   also fails (the `dead_code` it suppressed re-fires).

So producer logic + `embedded_expose_set` + forward-step-4 wiring + both attr
removals must land together or the tree does not build at the phase boundary.
TDD still applies **within** the phase: the seven `project_starters` unit tests
(sidecar emission, backfill, idempotent, preserve-edits, seal-respected, dry_run,
fresh-dir) drive the producer red→green; `project_starters` is untested dead code
today, so these are net-new, not an extension.

The **fresh-dir** VT (VT-7) is the F2 regression guard: `write_atomic`
(`fsutil.rs:52`) resolves the parent but never `create_dir_all`s, so a fresh
projection into a not-yet-existing `<band>/` dir `ENOENT`s today. VT-7 is red
until the `create_dir_all(parent)` lands. (The normal `doctrine install` flow
doesn't hit this — base install pre-materialises every embedded `.md` and its
band dir before forward-steps — but the unit path and the user-reset self-heal
path both do.)

### Why the goldens + reconcile are PHASE-02, after the producer

PHASE-02 does three things that all depend on the wired producer:

- **Resolver symmetry goldens** (src/hymns.rs, tests only) lock the expose-side
  mirror of the existing seal golden (`sealed_slot_drops_user_disk_twin…`,
  :1006). These characterise the **unchanged** engine — REV-019 already
  pressure-tested the self-`replaces` path against `hymns.rs`; the goldens make
  that a standing safety net. D4's behaviour-preservation gate is proved by an
  empty `hymns.rs` production diff (VA-2).
- **Corpus-wide legality + single-emit E2E** closes the design's F3 gap. The
  file-I/O unit tests in PHASE-01 prove the sidecar is *written* but never that
  it is *legal*: a self-`replaces` that is not the unique-most-specific active
  snippet of its slot is a `NonTopReplacer`/cycle error that makes `prompt
  resolve` fail outright. VT-3 runs `prompt check` (⇒ `validate_replaces`) over
  the projected corpus — the only test that exercises INV-3 end-to-end. VT-4
  drives **all 5** exposed slots, not just `role/worker`, because the defect is
  corpus-wide. INV-3 legality was re-verified during design: no embedded sidecar
  exists for any of the 5 slots, so each framework twin uses the pure
  `default_selector` and the user twin (same selector + `replaces`) is the strict
  unique top by provenance.
- **Reconcile** (D5) runs the wired producer against the repo to backfill the 5
  orphan sidecars, commits them as authored data, and confirms the live corpus
  no longer doubles. No bespoke migration — the reconcile *is* the producer,
  which is exactly the point of wiring it.

### Corpus exhaustiveness (why "5 slots" is complete, not a sample)

Verified during design: embedded `install/hymns/**` = `preamble/core` (sealed) +
the 5 exposed slots + `README.md` (non-slot, loader skips). `seal ∪ expose`
covers every embedded hymn slot, so base install copies no non-sealed/non-exposed
twin that would double outside the projector's reach. This is a **maintenance
invariant**: a future embedded hymn that is neither sealed nor exposed would be
copied to disk as a doubling User twin the projector never sidecars.

## Notes

- **Mechanism is locked** (REV-019, approved). This plan does not relitigate
  self-`replaces` vs implicit same-slot override; it implements the projector.
- **Sidecar-repair gap** (design F4, accepted): write-if-absent is self-healing
  only for a *missing* sidecar; a user who hand-edits a projected `.toml` and
  drops the `replaces` line gets a permanently doubling slot the projector won't
  repair. Out of scope; `prompt check` surfaces it.
- **STD-001 / POL-002**: the sidecar `replaces` string is single-sourced off
  `Slot::path()` (no magic string, D3); the sidecar is doctrine-owned mechanism,
  host-agnostic.

## Risks & mitigations (critical pass)

- **Reconcile side effects (PHASE-02 EX-3).** The named reconcile path is
  "run the wired producer", but the only live call site is forward-step 4 inside
  `doctrine install`, which runs **all** forward steps against the repo. Risk: a
  full install pass touches more than the 5 sidecars. Mitigation: run the rebuilt
  `./target/debug/doctrine` install, `git status --porcelain` first, review the
  diff, and commit **path-limited** to `.doctrine/hymns/**/*.toml` (AGENTS.md
  path-limit-commit rule) — never a pathless commit. If install proves to touch
  unrelated authored state, fall back to invoking `project_starters` directly
  from a one-shot (still "running the producer", not hand-authoring). phase-plan
  decides the exact invocation.
- **VT-4 context activation (assumption).** VT-4 asserts each of the 5 exposed
  slots single-emits, which requires the E2E to construct a `prompt resolve`
  context that *activates* each — the model-band slots
  (`model/anthropic/claude-sonnet-4`, `model/deepseek/_default`) need a matching
  `--model` context, `harness/claude` a `--harness`, the role slots a `--role`.
  Assumption: the `prompt resolve` flags can pin every axis needed. Low risk
  (that is the verb's contract); if a slot proves unreachable via flags, the
  golden in VT-1/VT-2 plus VA-1's in-repo `prompt explain` still cover it.
- **PHASE-01 size.** Seven unit VTs + wiring + two attr removals in one phase is
  larger than typical, but the F1 compile-gate coupling forbids splitting it and
  the surface is a single file (`src/install.rs`). Accepted.
