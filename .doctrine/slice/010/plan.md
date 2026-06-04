# Implementation Plan SL-010: Symlink skills from a canonical .doctrine/skills tree (Claude-first)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Five phases replace the Claude-path copy-and-skip with a canonical
`.doctrine/skills/<id>` tree plus relative symlinks, governed by ownership-by-
target-equality. The spine is the design's two load-bearing fixes from the codex
passes ([design.md](design.md) §10): **never clobber foreign data** (a link is
ours iff `read_link == ../../.doctrine/skills/<id>` — anything else, foreign
symlink or real dir, is kept; D3/F2) and **honest atomicity** (the leaf-link
write is genuinely atomic; the canonical *directory* swap is not — `rename` can't
replace a non-empty dir, no `renameat2` in std — so it is a staged, minimal-window
remove-then-rename, crash-healed by an idempotent re-run; D5/pass-2 F4).

The phases are sized so each ends green and is a clean review unit, building
inert seams bottom-up (gitignore helper → canonical writer → pure classifier)
before the one phase that flips behaviour (execute + list), then sealing with an
end-to-end refresh proof. The `npx` delegate path and the entity engine are
untouched throughout — the existing `delegate_argv` and entity/slice/state suites
are the behaviour-preservation gate.

## Sequencing & Rationale

- **PHASE-01 (gitignore + manifest) first — smallest, fully inert.** Adds the
  `.doctrine/skills/*` manifest entry and extracts the shared `ensure_gitignored`
  helper, with the copy mechanism untouched. It lands F4 (skills install self-
  enforces its derived-tree ignore, not order-dependent on `doctrine install`) and
  the manifest test with zero risk to the Claude path. Front-loaded because it has
  no dependency on any later seam and de-risks the storage-rule invariant early.
- **PHASE-02 (materialise) before classification.** The canonical writer is the
  riskiest imperative piece — the staged temp + minimal-window swap from pass-2 F4
  — so it is isolated and unit-tested standalone (atomic-overwrite + crash-heal)
  with a tempdir path argument, no plan model involved. It reuses `copy_skill`
  unchanged (no parallel copier). Landing it before the plan model means execute
  (PHASE-04) only has to *call* a proven primitive.
- **PHASE-03 (pure classification) is the disk-free decision core.** `canonical_dir`,
  `relative_target`, and `classify_link` are pure functions, so the ownership
  trichotomy — the design's load-bearing correctness (D3) — is asserted in tempdir
  states with no mutation. It replaces the `Step{Install,Skip}` model with
  `Link{Create,Relink,KeepForeign}` + `AgentPlan::Claude{canonical, links}`. Pure-
  before-imperative keeps the risky ownership logic testable without the seam.
- **PHASE-04 (execute + list) is the single behaviour-flip.** It wires
  materialise + `write_link` (the genuinely-atomic leaf write) behind the classified
  plan, switches `run_list` to `lexists` (F5), and deletes the copy-and-skip path.
  This is the only phase that changes observable behaviour and rewrites
  `claude_steps`' tests (R4) — kept to one phase so the diff that retires the old
  mechanism is a single, contained review unit. The ownership invariant (only
  missing-or-ours links mutated) is proven here against foreign symlink + real dir.
- **PHASE-05 (e2e + docs + close-out)** seals it: the end-to-end test proves the
  whole point — refresh-on-reinstall — against the built binary, and confirms a
  real-dir override survives. Docs describe the symlink model + override hatch;
  close-out harvests durable findings.

The dependency graph is a strict chain (01 → 02 → 03 → 04 → 05); each phase's
entrance criterion is the prior phase merged. The inert seams (01–03) could in
principle be reviewed together, but are sequenced linearly for a single reviewer
and so the behaviour-flip (04) lands against fully-tested primitives.

## Notes

- **Behaviour-preservation gate.** The `npx` delegate path (`delegate_argv`,
  `AgentPlan::Delegate`) and the entity engine are untouched all slice; their
  suites must stay green unchanged. The only intended behaviour change is the
  Claude install/list path (PHASE-04) — whose own `claude_steps`/list tests update.
- **Ownership spelling (accepted limitation, design §5.5/pass-2 F5).** Ownership is
  raw `read_link == <relative target>`. A differently-spelled equivalent link
  (absolute, or normalised) classifies foreign → kept + warned, never clobbered.
  v1 emits exactly one spelling so the case does not arise in-version; normalising
  is a rejected-for-v1 future hardening. The plan does **not** attempt it.
- **Out of scope (design §5.4/Q4, F3).** `--global` auto-detection and a global
  mode for `skills list` pre-exist SL-010; only the explicit `--global install
  --agent claude` path is supported here. Orphan pruning (a skill dropped from the
  embed) is a deferred follow-up (Q2). The `DELEGATE_SOURCE` typo and the opt-in
  `just` module are separate follow-ups (slice-010.md).
- **The gate.** No code before this plan is approved. After approval:
  `doctrine slice phases 010` to materialise tracking, then detail each phase in
  its `state/.../phase-NN.md` just prior to execution; flip status with
  `doctrine slice phase 010 PHASE-NN --status in_progress`, end each phase green.
- **`src/memory.rs` stays dirty** — pre-existing SL-007 WIP, not part of this slice.
