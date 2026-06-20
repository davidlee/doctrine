# Claude dispatch arm fail-closed base integrity

## Context

Source: **ISS-034 Defect A** (Defect B is folded into ISS-011). Discovered
dogfooding `/dispatch` (claude arm) on SL-121, 2026-06-20.

The claude `/dispatch` arm (`/dispatch-agent`) places each worker at base==B by
`cd`-ing the Bash cwd into the coordination worktree before spawning an `Agent`
with `isolation: worktree` — which forks the Bash-cwd HEAD under
`worktree.baseRef: "head"`. That placement is **correct only while `main` is
static**. Under a busy shared single clone (80+ live worktrees), concurrent
`git worktree add` / `prune` / `checkout` contend on git's repo-global locks
(`index.lock`, `HEAD.lock`, `.git/worktrees`); when a worker's worktree creation
loses that race the Claude Code subagent **silently falls back to the main
worktree** (`/workspace/<repo>`), where `baseRef:"head"` then tracks a **moving
`main`**. Net: the worker runs on a wrong, dirty, moving base instead of B — and a
worker even started on the correct fork and was **clobbered to `main` mid-run**
when `main` moved (SL-121 PHASE-02, failed 3× consecutively).

The parallel-work model assumes `isolation: worktree` yields a *stable* fork
pinned to B for the worker's whole lifetime. That invariant does not hold under
real multi-agent load. Today the arm fails *closed* only because an **ad-hoc**
base-guard was hand-added to each worker prompt; with no guard a worker would
author a phase atop the wrong base and commit it, leaving the funnel's
`verify-worker` / delta checks the only backstop.

Governing canon: **PRD-015** (Dispatch & worktree), **ADR-006** (worktree
posture: orchestrator-sole-writer), **ADR-011** (harness-agnostic spawn; the D6
"not pre-worker fail-closable" residual class names exactly this gap), **ADR-012**
(dispatch integration topology). Mechanism memories:
`mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head`,
`mem.pattern.dispatch.agent-worktree-forks-bash-cwd-head`,
`mem.signpost.doctrine.dispatch-claude-arm-wrong-base`.

## Scope & Objectives

Make the claude dispatch arm **fail closed** against silent fallback-to-main /
wrong-base — so a contention-induced wrong base is *always* caught loudly before
import, not dependent on a hand-added prompt guard.

In scope (the doctrine-owned surface):

1. **Standardise the worker base-guard** as a first-class part of the
   `dispatch-worker` spawn (the `/dispatch-agent` prompt template, not ad hoc): the
   worker's first action asserts `git status` clean, greps the prerequisite seams,
   and checks `git merge-base --is-ancestor B HEAD`, STOP-and-report on mismatch.
   (ISS-034 remedy 2.)
2. **Harden `doctrine worktree verify-worker`** (`src/worktree.rs`) to detect the
   fallback explicitly and refuse: worker worktree path == the main worktree, or
   `merge-base --is-ancestor B HEAD` false, or a missing-isolation signal. Turn a
   silent wrong-base into a loud funnel halt **independent of** the prompt guard.
   (ISS-034 remedy 3.)
3. **Orchestrator post-spawn detection** of a missing `worktreePath:` footer in the
   Agent return (no isolated tree was created) — treat as a red flag and halt.
   (Overlaps **IMP-052**; reconcile with it.)

Objective bar: two independent belts (prompt guard + `verify-worker`) each
sufficient to catch the fallback, so the arm fails closed even if one is absent.

## Non-Goals

- **Defect B** (SubagentStart `(deleted)` stamp-hook path) — tracked in **ISS-011**.
- **Harness-internal lock-retry / backoff** on worktree creation (ISS-034 remedy
  4) — Claude Code internals, not doctrine-owned; out of scope.
- **Switching the dispatch default to the subprocess arm** (remedy 6) — the
  subprocess arm remains the recommended fallback under heavy churn, but
  re-defaulting is a separate decision.
- **Repo-wide `baseRef`→branch/SHA pinning** (remedy 1) — `settings.local.json` is
  repo-wide and affects every agent; viability is an open question for `/design`,
  not committed here.
- **`WorktreeCreate` pre-worker hook** (**IMP-072**) — the true pre-worker
  fail-closed mechanism per ADR-011 D6, but deferred; `/design` decides whether
  this slice pulls it in or stays at the prompt-guard + `verify-worker` belts.

## Affected surface (concrete)

- `plugins/doctrine/skills/dispatch-agent/SKILL.md` — worker base-guard block +
  pre-funnel footer gate (claude-arm only).
- `src/worktree.rs` — `verify-worker` `not-isolated` belt (primary-tree fallback).
- `tests/e2e_skills_dispatch_shrinkage.rs` — budget bump + content presence asserts.
- Tests: `classify_worker_verify` goldens + `run_verify_worker` integration.

NOT modified: `plugins/doctrine/skills/dispatch/SKILL.md` (router funnel). Per the
codex adversarial pass (design §10), the footer parse is a claude `Agent` artifact
handled as an arm-level pre-funnel gate; the funnel's own belts (import `S^==B`,
head-moved) are unchanged and are the existing backstop this slice leans on for the
mid-run-clobber / misplacement residuals.

## Risks / Assumptions / Open Questions

- **OQ-1:** does Claude Code `worktree.baseRef` accept a branch/SHA, or only
  `"head"`? (Gates remedy 1.) Verify empirically — do not infer.
- **OQ-2:** is the prompt-guard + `verify-worker` belt sufficient, or does durable
  fail-closure require `WorktreeCreate` (IMP-072)?
- **OQ-3:** can `verify-worker` reliably identify "the main worktree" across jail
  layouts (`/workspace/<repo>`) without false positives on legitimate forks?
- **Assumption:** the funnel already calls `verify-worker --base B` pre-import
  (ADR-011 §8.4 belt); this slice strengthens that belt, not adds a new stage.
- **Risk:** prompt-template changes are unverifiable by `cargo test`; their VT must
  be VA/VH (guard text present + a contention dry-run), not a unit test.

## Verification / Closure intent

- Under simulated contention (worker cwd forced to the main worktree, or B not an
  ancestor of HEAD), `verify-worker` **refuses with a clear cause** before import —
  proven by test.
- Every `/dispatch-agent` spawn carries the base-guard by template (VA: template
  inspection).
- A missing `worktreePath:` footer halts the funnel (VA/VH on the skill cadence).
- ISS-034 Defect A reconciled (resolved/promoted) at close; IMP-052 overlap
  reconciled.

## Summary

Harden the claude `/dispatch` arm so a contention-induced silent fallback-to-main
(wrong/moving worker base) always fails closed: standardise the worker base-guard
in the spawn template and harden `verify-worker` to detect the fallback, two
independent belts. Scopes ISS-034 Defect A; Defect B lives in ISS-011.

## Follow-Ups

- IMP-072 (WorktreeCreate hook) — if `/design` defers it again, leave it open with
  this slice as the trigger context.
