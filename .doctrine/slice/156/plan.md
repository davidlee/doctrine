# Implementation Plan SL-156: Per-worktree CARGO_TARGET_DIR for dispatch workers

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases deliver the B1 design: per-worktree build isolation by **retiring**
the shared `CARGO_TARGET_DIR`, and a platform made **build-tool-agnostic** by
removing the cargo coupling (POL-002). The work splits along a hard seam — the
**project-side mechanism** (PHASE-01) is the correctness change; the
**platform removal** (PHASE-03) is the POL-002 cleanup; the **codex-skill
migration** (PHASE-02) bridges them; the **ritual/memory cleanup** (PHASE-04) is
hygiene the mechanism enables.

The ADR-008 mechanism change is **not** carried by a phase — it rides REV-011
(`revises` ADR-008), authored in design and approved+applied at reconcile, when
the code that makes the new mechanism real has landed.

## Sequencing & Rationale

The order is the design's migration order (design §8 R2), kept for
**reviewability and final-semantics validation** — *not* regression avoidance.
EAP-2 corrected the earlier framing: `project_env_contract` fails *closed* to
isolation (`fork.join("target")` fallback when `CARGO_TARGET_DIR` is unset), so
no ordering strands a codex worker on a shared/unset env. The order is still
worth keeping because it lets each contract change be reviewed in isolation and
lets PHASE-01 validate the mechanism before the skill stops re-deriving a
`wt/<branch>` subdir.

- **PHASE-01 first** because retiring the flake export *is* the correctness
  mechanism — once the shared env is gone, every worktree (both arms, `just` and
  raw `cargo`) defaults to its own in-tree `target/`. Everything downstream is
  removal of now-dead machinery. The flake change is launch-time, so it is inert
  in the authoring session (R5): validate in-session by simulating with
  `.env_remove("CARGO_TARGET_DIR")` (the e2e pattern already in the suite); the
  true end-to-end check is a `VH` after a jail relaunch.

- **PHASE-02 before PHASE-03** so the codex worker stops re-deriving
  `wt/<branch>` from the stdout contract *before* that contract is deleted. Even
  reversed it is safe (EAP-2), but this keeps the rollout clean and the
  intermediate state honest.

- **PHASE-03** removes the platform coupling and is the POL-002 payload. The
  behaviour-preservation gate here is **assertion-grained, not test-grained**
  (EAP-1): the env-contract assertions are *blocks inside* the worktree creation
  tests, so they are excised surgically while every creation/provision/marking
  assertion stays green. The gc target-base scaffold is the one whole-test
  deletion. EAP-5: `run_fork`'s stdout was *only* the env contract (the created
  path goes to stderr), so its stdout becomes empty — there is no other output to
  keep. EAP-4: the stale stdout-contract advertisements (`mod.rs` help,
  `provision.rs` comment, generic `/worktree` skill) are refreshed here too.

- **PHASE-04 last** because the stale-target rituals only become safe to remove
  once the in-tree model is the production reality. R3/OQ-3: re-evaluate each
  ritual and memory against the host and non-jail flows before deleting — mark a
  memory superseded only when confirmed no longer true, not by blanket sweep.

## Notes

- **Behaviour-preservation gate** (AGENTS.md): the worktree creation/coordination
  suites are the proof — they stay green unchanged. Only the deliberately-removed
  env-contract assertions change (design §9).
- **Verification modes.** `VH` is used where a test cannot judge in-session: the
  flake effect needs a jail relaunch (PHASE-01 VH-1, PHASE-03 VH-1). Do not
  silently skip these — they are checked downstream at reconcile.
- **REV-011** is the governance counterpart; its apply at reconcile lands the
  ADR-008 D-B1/D-B5 mechanism edit. Plan execution must not edit ADR-008 directly.
