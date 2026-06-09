# Dispatch orchestrator funnel: worker-mode workers and import-verify-commit-record

## Context

This slice realises **IMP-003's funnel half** — the OQ-1 split deferred by
**SL-029** — under **ADR-006** (worktree posture, orchestrator-sole-writer).
SL-029 shipped the *lifecycle* (detection D1, creation ladder D5/D9,
`doctrine worktree provision`/`check-allowlist`, the `/worktree` skill's **solo**
side, commit-before-spawn + single-tree branch-point). This slice builds the
**worker-mode-ON funnel** on top of that lifecycle: the orchestrator-sole-writer
discipline (D2/D6/D7), worker-mode workers (D6a), and filling the `/dispatch`
placeholder.

The governing decisions mechanised here: the **worker-sole-writer invariant**
(D2) — workers mutate source only, return a structured report + source delta,
never commit doctrine state; **orchestrator pre-distill** (D6); **funnel
discipline** (D7) — incremental per-batch persistence in strict order
*import delta → verify → commit → record knowledge* on the coordination branch
(D8); the **branch-point check extended to the concurrent case** (D5); and
**worker-mode ON** for funnel workers (D6a, the half SL-029 left stubbed).

**Execution depends on IMP-002** (worker-mode guard `DOCTRINE_WORKER=1` (D2a) +
trunk-ref minting / reseat (D3)) — still `open`. IMP-002 is the funnel's
prerequisite, named as such by both ADR-006 and SL-029. The dependency blocks
*execution*, not scoping or design: this slice's design assumes IMP-002's D2a
guard surface and D3 minting exist, and proceeds. Reuses the `/worktree` skill
SL-029 landed; fills `plugins/doctrine/skills/dispatch/SKILL.md` (placeholder).

## Scope & Objectives

- **Worker contract (D2/D6a).** Implement the `mode=worker` side of the
  `/worktree` skill (SL-029 landed solo only): worker-mode ON, worker mutates
  **source only**, returns a **structured report + source delta** (a
  branch/worktree diff or patch — not a prose description), never commits
  doctrine state. Doctrine-mediated authored writes refuse under worker-mode
  (honouring IMP-002's D2a guard).
- **Orchestrator pre-distill (D6).** The worker receives a self-contained prompt
  — policy digest, design excerpts, pre-fetched memories, task spec, mandatory
  verification command. Workers do **not** read boot/governance or run `/boot`.
- **Funnel discipline (D7).** The orchestrator persists incrementally per batch
  in strict order **import delta → verify → commit → record knowledge** on the
  coordination branch (D8); knowledge always trails confirmed code; crash/overflow
  recovery = rebuild from coordination branch + `git worktree list`.
- **Branch-point check under concurrency (D5).** Extend SL-029's single-tree HEAD
  pre/post compare to the concurrent-batch case: a HEAD mismatch at import time →
  re-dispatch rather than silently merge against a moved base.
- **`/dispatch` skill (D5).** Fill the placeholder: mandatory isolation via the
  harness `Agent` isolation mechanism, orchestrator-sole-writer remit, the funnel
  loop above, and the recovery prose.
- **Deterministic worker provisioning at the harness seam (IMP-003 A-6, candidate
  — design-gated).** A **Claude-only, opt-in** `WorktreeCreate` hook that runs
  `doctrine worktree provision <fork>` when a worker spawns with
  `isolation: "worktree"`, closing the "relies on the worker remembering to
  provision" gap. **Provision stays the sole copier** — the hook only *guarantees
  it runs*, it is never a second copy path. An *optimisation over* the portable
  rung-3 fallback (`git worktree add` + provision), never a dependency. Whether
  this lands here or defers is OQ-1.

## Non-Goals

- **IMP-002 machinery** — the worker-mode guard (D2a) and trunk-ref minting /
  reseat (D3). Prerequisite, owned by IMP-002; this slice *consumes* it.
- **The SL-029 lifecycle half** — detection, creation ladder, `provision`/
  `check-allowlist`, solo `/execute` isolation. Already shipped; reused, not
  rebuilt.
- **Raw-tree confinement (D2b)** — OS-enforced worker confinement; deferred to
  ADR-008's bwrap spike. The funnel rests on the CLI guard (IMP-002) + prompt
  contract.
- **Project-local jail concerns (ADR-008)** — per-worktree `CARGO_TARGET_DIR`,
  bwrap, `sccache`.
- **Adversarial-review ledger (ADR-007)** — the orthogonal single-tree primitive.
- **Anchor-stability seam (ADR-006 Open)** — moving the memory anchor off the
  volatile branch sha; deferred until squash-orphaning proves common.

## Affected surface

- `plugins/doctrine/skills/dispatch/SKILL.md` — **fill placeholder**: orchestrator
  funnel, worker spawn via `Agent` isolation, the import→verify→commit→record
  loop, branch-point-under-concurrency, crash recovery. Authored in `plugins/`,
  not the gitignored `.doctrine/skills/` install copy.
- `plugins/doctrine/skills/worktree/SKILL.md` — implement the `mode=worker`
  contract (SL-029 stubbed it; solo shipped).
- `plugins/doctrine/skills/execute/SKILL.md` — clarify the worker-vs-solo boundary
  only if the funnel contract requires it (D6a already drawn in SL-029).
- **Possible CLI surface (design to settle, OQ-2)** — whether any of structured-
  report / source-delta import / funnel ordering is a CLI verb (`src/worktree.rs`
  / a new `doctrine worktree …` subcommand) or stays orchestrator skill-prose.
  SL-029's precedent: mechanics in skill-prose, thin tested CLI verbs at the seam.
- **Possible install/boot wiring** — for the Claude-only `WorktreeCreate` hook
  (A-6), if OQ-1 resolves to ship it. Design-gated.
- **Tests** — funnel-order conformance (import→verify→commit→record); branch-point
  check under the concurrent-batch case.

## Risks, assumptions, open questions

- **A-1 — IMP-002 is an execution prerequisite.** Design proceeds assuming its
  D2a worker-mode guard and D3 trunk-ref minting surfaces exist; execution is
  blocked until IMP-002 lands.
- **OQ-1 — WorktreeCreate hook (A-6): ship here or defer?** Sharp edges to decide
  in `/design`: **interception scope** (the hook replaces *all* Claude worktree
  creation in the project, not just doctrine's → needs opt-in/scoping, not blanket
  install); **portability** (Claude-only; a non-Claude funnel agent has no hook →
  must fall back to rung-3 `git worktree add` + provision; the hook is an
  optimisation, never a dependency); **force-copy reconciliation** (a
  doctrine-authored hook body must be **provision-only** — if it copies, the
  SL-029 sole-copier invariant degrades to `check-allowlist` only).
- **OQ-2 — CLI vs skill-prose boundary for the funnel.** How much of import /
  verify / commit / record is a tested CLI verb vs pure orchestrator skill-prose
  (mirrors SL-029 OQ-3).
- **OQ-3 — source-delta representation.** Branch diff vs patch vs handed-back
  worktree path; how the orchestrator imports it onto the coordination branch.
- **R-1 — D2b residual gap.** A worker can still raw-edit main (the harness does
  not confine it to its worktree, ADR-006 D2b); the funnel rests on the CLI guard
  (IMP-002) + prompt contract. Known, deferred to ADR-008.
- **R-2 — squash-merge orphans coordination-branch memory anchors** (ADR-006
  Consequences). Convention: record on trunk/coordination branch. Deferred seam.

## Verification / closure intent

Done when: the `mode=worker` contract is implemented and a worker returns a
structured report + source delta with doctrine-mediated writes refusing under
worker-mode (D2a, via IMP-002); the orchestrator funnel executes
**import → verify → commit → record** in strict order on the coordination branch
with incremental per-batch persistence (D7), and the ordering is test-asserted;
the branch-point check holds under the concurrent-batch case (D5); the `/dispatch`
skill ships **filled** (no longer a placeholder) with the orchestrator-sole-writer
remit and crash/recovery prose; and the WorktreeCreate-hook decision (OQ-1) is
resolved — installed-and-scoped (provision-only) or explicitly deferred with
rationale. ADR-006's funnel / D7 / D2a / branch-point-under-concurrency
Verification bullets are the conformance basis.

## Follow-Ups

- **Close IMP-003** once both SL-029 (lifecycle) and this slice (funnel) land;
  link IMP-003 ↔ SL-029 / SL-031 (backlog→slice relations are empty in v1 — the
  registry does not yet exist).
- **ADR-008 bwrap spike** — discharges the D2b raw-tree-confinement gap this slice
  rests on.
- **Anchor-stability seam** (ADR-006 Open) — if squash-orphaning of coordination-
  branch memory proves common.
