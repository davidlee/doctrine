# Dispatch worktree creation: detection and creation paths with guards

## Context

Implements IMP-003 under the now-accepted **ADR-006** (worktree posture,
orchestrator-sole-writer). ADR-006 fixes the *policy*; this slice builds the
*mechanism*: the worktree lifecycle and orchestrator funnel for `/dispatch` (and
the optional `/execute` isolation path). The `/dispatch` skill is currently a
placeholder.

The governing decisions: detect-don't-prescribe isolation (D1); the creation
**preference ladder** native→`git worktree add`→work-in-place (D5/D9); fork
**provisioning** = regenerate-derivable + copy-irreducible via a project-owned
`.worktreeinclude` allowlist, withholding the coordination tier (D9); the
commit-before-spawn and branch-point guards (D5); the import→verify→commit→record
funnel (D7); worker-vs-solo write rules (D6a).

**Hard dependency: IMP-002** (worker-mode CLI guard `DOCTRINE_WORKER=1` (D2a) +
trunk-ref minting / reseat (D3)) is a prerequisite and is **not** in this slice.
This slice *assumes* that machinery and is sequenced after it.

## Scope & Objectives

- **Detection (D1).** `GIT_DIR != GIT_COMMON` with the submodule guard; adapt to
  observed isolation rather than prescribe it. The solo trunk-based path stays
  untouched (no worktree required).
- **Creation ladder (D5/D9).** Prefer the harness native tool (Claude Code
  `WorktreeCreate`/`WorktreeRemove` hooks) → fall back to `git worktree add` →
  degrade to work-in-place (solo, no funnel) on sandbox denial. Delegate the
  mechanism; reinvent `git worktree` only at the fallback rung.
- **Fork provisioning (D9).** Regenerate derivable prerequisites (`cargo build`);
  copy only irreducible gitignored files via a project-owned `.worktreeinclude`
  allowlist; **enforce the invariant** that the allowlist excludes the
  coordination/runtime tier (`.doctrine/state/`, `phases`, `handover.md`, memory
  caches). Baseline-verify the fork builds+tests green before dispatch.
- **Guards (D5).** Commit-before-spawn (a fork sees only committed HEAD);
  branch-point check (HEAD pre/post-spawn mismatch → re-dispatch).
- **Funnel discipline (D2/D6/D7).** Worker returns a structured report + source
  delta; orchestrator pre-distills worker context (D6) and persists incrementally
  in strict order **import delta → verify → commit → record knowledge** on the
  coordination branch. Crash/overflow recovery = rebuild from coordination branch
  + `git worktree list`.
- **Worker vs solo (D6a).** Worker-mode ON for funnel workers; OFF for solo
  agents. Land the `/dispatch` skill; thread the optional isolation path into
  `/execute`.

## Non-Goals

- **IMP-002 machinery** — the worker-mode guard (D2a) and trunk-ref minting /
  reseat (D3). Prerequisite, separate slice.
- **Raw-tree confinement (D2b)** — OS-enforced worker confinement; deferred to
  ADR-008's bwrap spike. This slice rests on the CLI guard + prompt contract.
- **Project-local jail concerns (ADR-008)** — per-worktree `CARGO_TARGET_DIR`,
  bwrap, `sccache`. This slice provides the framework seam; the jail instance is
  ADR-008's.
- **Adversarial-review ledger (ADR-007)** — the orthogonal single-tree primitive.

## Affected surface

- `.doctrine/skills/dispatch/` — the placeholder skill, filled here.
- `.doctrine/skills/execute/` — optional isolation path (D6a).
- `.worktreeinclude` default + the `WorktreeCreate`/`WorktreeRemove` hook wiring
  (Claude Code settings / `install/`).
- `install/manifest.toml` — ship the new authored/skill files.
- `src/` — only if the branch-point check or baseline-verify needs CLI support
  (TBD at `/design`; much may be skill-level prose, not Rust).

## Risks, assumptions, open questions

- **OQ-1 (altitude / sprawl).** IMP-003 bundles two coherent units: the
  **worktree lifecycle** (detect / create-ladder / provision / baseline / guards)
  and the **orchestrator funnel** (import→verify→commit→record, worker-vs-solo).
  `/design` to decide: one slice phased into two, or split into a second slice.
- **OQ-2 (harness specificity).** The `WorktreeCreate` hook + `.worktreeinclude`
  is Claude-specific (the native rung). The framework-neutral rungs
  (`git worktree add`, work-in-place) must be fully specified so a non-Claude
  harness still works — D1 framework-neutrality.
- **OQ-3 (CLI vs skill boundary).** Which guards are CLI-enforced (testable Rust)
  vs skill-prose orchestration? Branch-point check is orchestrator-side; the
  allowlist-exclusion invariant could be a CLI assertion. Resolve at `/design`.
- **A-1.** IMP-002 lands first; this slice assumes `DOCTRINE_WORKER=1` and
  trunk-ref minting exist.
- **R-1.** Worker self-verify (D6) is degraded if provisioning is incomplete, but
  the orchestrator's authoritative re-verify (D7) runs where prereqs exist, so
  this is an efficiency risk, not a correctness break.

## Verification / closure intent

Done when: detection adapts to isolation correctly (incl. submodule guard); the
creation ladder degrades cleanly through all rungs; provisioning regenerates +
copies per allowlist and the **coordination-tier exclusion invariant is enforced
and tested**; baseline-verify gates dispatch; commit-before-spawn and
branch-point guards hold; the funnel persists in strict D7 order; `/dispatch`
ships (no longer a placeholder) and worker/solo write rules (D6a) are respected.
ADR-006's Verification bullets are the conformance basis.

## Follow-Ups

- ADR-008 bwrap spike (discharges D2b) — the OS-enforced confinement this slice
  defers to.
- Anchor-stability seam (ADR-006 Open) if squash-orphaning proves common.
