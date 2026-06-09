# Dispatch worktree creation: detection and creation paths with guards

## Context

Implements **IMP-003's lifecycle half** under the now-accepted **ADR-006**
(worktree posture, orchestrator-sole-writer). ADR-006 fixes the *policy*; this
slice builds the worktree *lifecycle* mechanism + the optional `/execute`
isolation path — **solo, no funnel, no IMP-002 dependency** (design OQ-1
slice-split). The orchestrator funnel + `/dispatch` are a separate follow-up
slice (IMP-002-dependent). `/dispatch` stays a placeholder here.

**Design resolved (`design.md`):** OQ-1 → slice-split; OQ-3 → CLI owns `worktree
provision` + `check-allowlist` (exclusion enforced *at the copy seam*); OQ-2 →
`provision` is the framework-neutral copy rung. The lifecycle lives in a
standalone **`/worktree`** skill invoked by `/execute` (and the future
`/dispatch`).

The governing decisions mechanised here: detect-don't-prescribe isolation (D1);
the creation **preference ladder** native→`git worktree add`→work-in-place
(D5/D9), with `git worktree add` + `provision` as the *blessed tested default*
(native rung opportunistic); fork **provisioning** = regenerate-derivable + copy
only the irreducible gitignored files via a project-owned `.worktreeinclude`
allowlist, withholding the coordination tier (D9); commit-before-spawn (D5).

**Assumed, not built — IMP-002** (worker-mode guard `DOCTRINE_WORKER=1` (D2a) +
trunk-ref minting / reseat (D3)). It is a prereq for the *funnel slice*, **not**
for this one: solo `/execute` isolation never mints ids, so SL-029 carries no
IMP-002 dependency.

## Scope & Objectives

- **Detection (D1).** `GIT_DIR != GIT_COMMON` with the submodule guard; adapt to
  observed isolation rather than prescribe it. The solo trunk-based path stays
  untouched (no worktree required).
- **Creation ladder (D5/D9, skill-prose).** Detect existing isolation → harness
  native tool (opportunistic) → **`git worktree add` + `doctrine worktree
  provision` (the blessed tested default)** → degrade to work-in-place (solo, no
  funnel) on sandbox denial. Reinvent `git worktree` only at the fallback rung.
- **Fork provisioning (D9, CLI).** `doctrine worktree provision <fork>` copies only
  the irreducible *gitignored* files matching a project-owned `.worktreeinclude`
  allowlist, **enforcing the exclusion invariant at the copy seam** — coordination/
  runtime tier (`.doctrine/state/`, `phases`, `handover.md`, memory caches) is
  withheld even under a broad `**` pattern (skip+warn). Regenerate-from-source +
  baseline-verify green via the project-configured command (this repo: `just
  check`) is the skill step before handoff.
- **Guards (D5).** Commit-before-spawn — an exact `git status --porcelain -z` gate
  (abort on tracked-dirty or untracked-non-ignored; the fork sees only committed
  HEAD). Branch-point check (HEAD pre/post-create compare) is **in scope** — cheap,
  ADR-D5-mandated, no IMP-002 needed; the funnel slice only extends it to the
  concurrent case.
- **Worker vs solo (D6a).** This slice lands the **solo** side only: `/execute`
  isolation, worker-mode OFF, writing doctrine state directly. Worker-mode-ON funnel
  workers + `/dispatch` are the follow-up slice.

## Non-Goals

- **Orchestrator funnel + `/dispatch` (OQ-1 split half)** — the
  import→verify→commit→record discipline (D2/D6/D7), worker-mode-ON workers, and
  filling the `/dispatch` placeholder. Separate follow-up slice; depends on IMP-002.
- **IMP-002 machinery** — the worker-mode guard (D2a) and trunk-ref minting /
  reseat (D3). Prerequisite for the funnel slice, not this one.
- **Raw-tree confinement (D2b)** — OS-enforced worker confinement; deferred to
  ADR-008's bwrap spike. This slice rests on the CLI guard + prompt contract.
- **Project-local jail concerns (ADR-008)** — per-worktree `CARGO_TARGET_DIR`,
  bwrap, `sccache`. This slice provides the framework seam; the jail instance is
  ADR-008's.
- **Adversarial-review ledger (ADR-007)** — the orthogonal single-tree primitive.

## Affected surface

- `plugins/doctrine/skills/worktree/SKILL.md` — **new** lifecycle skill (mode
  contract; detection, creation, guards, baseline; invokes the CLI verbs).
  **Authored in `plugins/`, not the gitignored `.doctrine/skills/` install copy.**
- `src/worktree.rs` — **new**: pure core (`Allowlist`, structured `WITHHELD`,
  `is_withheld`, `select_copies`, `allowlist_violations`) + impure `provision` +
  canonicalize-guarded copy.
- `src/main.rs` — new `Worktree { Provision, CheckAllowlist }` subcommand.
- `src/fsutil.rs` — recursive, canonicalize/symlink-safe copy helper.
- `plugins/doctrine/skills/execute/SKILL.md` — thin optional solo-isolation thread.
- `.worktreeinclude` — **not installed**: project-owned; `provision` tolerates
  absence; the `/worktree` skill documents the template (design F2).
- `plugins/doctrine/skills/dispatch/SKILL.md` — **untouched** (placeholder; funnel).

## Risks, assumptions, open questions

- **OQ-1 (altitude / sprawl) — RESOLVED → slice-split.** Lifecycle (this slice)
  vs orchestrator funnel (follow-up slice). See `design.md`.
- **OQ-2 (harness specificity) — RESOLVED.** `doctrine worktree provision` is the
  framework-neutral copy rung, reading the project-owned `.worktreeinclude`
  allowlist; rung 3 (`git worktree add` + provision) is the tested default. (No
  native-hook allowlist parity assumed — native rung is opportunistic, design F1.)
- **OQ-3 (CLI vs skill boundary) — RESOLVED.** CLI owns `provision` +
  `check-allowlist` (exclusion enforced at the copy seam); detection / ladder /
  commit-before-spawn / baseline stay skill-prose.
- **A-1 — DOWNGRADED.** IMP-002 is a prereq for the funnel slice, not SL-029; this
  slice has no IMP-002 dependency (solo isolation never mints ids).
- **R-1.** Worker self-verify (D6) degradation is a funnel-slice concern; N/A here.
- **R-2 (native rung, design F1).** The Claude Code `WorktreeCreate` hook is a
  GitHub-discussion proposal, unconfirmed-shipped; rung 3 is the tested default, so
  an absent hook costs only a dead optimisation, not correctness.

## Verification / closure intent

Done when: detection adapts to isolation correctly (incl. submodule guard); the
creation ladder degrades cleanly through its rungs (`git worktree add` + provision
as the tested default); `provision` copies per allowlist and the **coordination-
tier exclusion invariant is enforced *at the copy seam* and tested** (incl. the
broad-`**` withhold case); `check-allowlist` rejects a tier-naming pattern;
baseline-verify (`just check`) gates handoff; commit-before-spawn + branch-point
guards (D5) hold; the `/worktree` skill ships (with its `mode=solo|worker` contract,
solo implemented) and `/execute` gains the solo optional-isolation thread (D6a). The
relevant subset of ADR-006's Verification bullets is the conformance basis (funnel /
D7 / D2a / branch-point-under-concurrency deferred to the follow-up slice).

## Follow-Ups

- **Orchestrator funnel slice (OQ-1 split half)** — import→verify→commit→record
  (D2/D6/D7), worker-mode-ON workers, branch-point check, fill `/dispatch`. Depends
  on IMP-002. Reuses the `/worktree` skill this slice lands.
- ADR-008 bwrap spike (discharges D2b) — the OS-enforced confinement this slice
  defers to.
- Anchor-stability seam (ADR-006 Open) if squash-orphaning proves common.
