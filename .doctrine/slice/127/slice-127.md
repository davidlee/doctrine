# Dispatch base freshness: ancestor-dominant ladder and mid-drive refresh

## Context

A `/dispatch` coordination branch forks off a base captured **once** at `setup`.
The base is a frozen snapshot of trunk; nothing advances it during the drive and
nothing inspects its drift until the very end. Two failure modes follow, both
witnessed live (SL-122, SL-125 drives) and recorded as backlog:

- **Wrong base at start (ISS-036).** `git::trunk_ladder` (`git.rs:1042`) hard-
  prefers `origin/HEAD` over local `main`. In this repo's commit-on-main local-
  first workflow `origin/HEAD` routinely lags local `main` by tens of commits. The
  coordination tree forks a stale trunk; when that trunk predates the slice's own
  authored `slice/NNN/plan.toml`, `worktree::coordinate` → `slice::run_phases` hard-
  aborts (`Plan for slice N not found …`) and rolls back. Workaround today is an
  env prefix on *every* dispatch command: `DOCTRINE_TRUNK_REF=main`.

- **Base drifts stale mid-drive (RSK-010).** Even with a correct base, a long drive
  (SL-122 ran 5 phases over hours) lets `main` advance underneath the branch. The
  drift is **invisible until candidate-create time**, when `dispatch candidate
  create` 3-way-merges the impl bundle onto current `main` and conflicts on a
  merge-*base* divergence (both sides rewrote the same block; here a `chore: fmt`).
  A fix on the dispatch branch cannot resolve it — the **merge-base itself** must
  advance. The terminal position (post-audit-prep) is the most disruptive place to
  discover it.

The shared root is **the base is a snapshot, not a tracked reference**. This slice
makes trunk-base selection ancestor-correct at setup and makes base-freshness an
observable, advanceable thing across the drive — turning a silent terminal
conflict into a small, early, in-context one.

## Scope & Objectives

Two axes, one shared merge-base seam (`git::trunk_ladder` / `merge-base
--is-ancestor`):

1. **Ancestor-dominant trunk ladder (ISS-036).** When two ladder candidates are
   ancestor-related, select the most-advanced (the descendant) rather than fixed
   `origin/HEAD`-first order. Framework-neutral by construction (ADR-006): no
   opinion about *which named ref* is trunk, only "never fork a base that an
   equally-valid candidate strictly dominates." Plan-presence is the tie-break
   safety net — refuse/warn loudly (with the `DOCTRINE_TRUNK_REF` hint) when the
   chosen base's tree lacks the dispatched slice's `plan.toml`, rather than aborting
   deep inside phase-sheet regen.

2. **Mid-drive base refresh + drift visibility (RSK-010).** Promote the SL-122
   manual fix (merge current trunk into `dispatch/<slice>`, regen bundle) to a
   first-class CLI verb, runnable per phase-conclude rather than once at the end, so
   the merge-base advances incrementally and each conflict is one phase's delta in
   context. Surface drift as a number (`merge-base(dispatch,trunk)` vs `trunk HEAD`)
   on dispatch status/sync output. When candidate-create *does* conflict, classify
   base-divergence vs content-conflict and name it, pointing at the refresh verb.

Per ADR-011 the mechanism lands as **tested CLI verbs/gates**, not skill prose;
skills only route to them.

## Non-Goals

- **Integrate-side phantom (ISS-038 / IMP-122).** Dirty-trunk integrate leaving a
  phantom reverse-deletion index is a *different seam* (stage-2 ref CAS vs the
  shared checkout) and a different risk class. Tracked separately; out of scope.
- **`[dispatch] deliver_to` / trunk-ref config (IMP-124 / IMP-101).** A durable
  single-source trunk ref is complementary but a distinct config surface; this
  slice fixes ladder *ordering*, not config plumbing. May become a follow-up.
- **`import --allow-reanchor` (IMP-043).** Moved-HEAD 3-way re-anchor at import is a
  separate funnel-stage concern.
- Rebase-vs-merge for the refresh verb is an **open question** for `/design`, not a
  settled non-goal — see below.

## Affected Surface

- `src/git.rs` — `trunk_ladder` (ordering + ancestor dominance), `trunk_commit`,
  a `merge-base --is-ancestor` / drift-count helper.
- `src/worktree.rs` — `coordinate` (~1700) base-selection gate + plan-presence
  check before phase-sheet regen.
- Dispatch sync/status surface (the refresh verb + drift reporting).
- Skills: `/dispatch`, `/dispatch-subprocess`, `/dispatch-agent` — route to the
  new verb; remove the `DOCTRINE_TRUNK_REF=main` env-prefix ritual.

## Risks / Assumptions / Open Questions

- **OQ-1 — refresh = merge or rebase?** The coordination branch's bundle is
  *regenerated* by prepare-review, so its commit history is semi-disposable, which
  favours rebase-onto-fresh-trunk (clean merge-base, no merge-commit noise). Merge
  is safer/idempotent and preserves history. Decide in `/design`; pressure-test
  against worker-commit sha references and the candidate-create bundle path.
- **OQ-2** — does ancestor-dominance fully subsume the plan-presence check, or are
  both needed (non-ancestor-related divergent refs)? Likely both.
- **A-1** — local `main` is the de-facto trunk here (commit-on-main, origin
  unpushed); the fix must not *assume* that for consuming projects (ADR-006).
- **R-1** — reordering the ladder could regress a consumer who genuinely wants
  `origin/HEAD` as the integration target; ancestor-dominance (not "prefer main")
  is the mitigation, plus `DOCTRINE_TRUNK_REF` stays the explicit override.

## Verification / Closure Intent

- Ladder selection unit-tested: ancestor pairs pick the descendant; non-ancestor
  divergence falls through deterministically; explicit `DOCTRINE_TRUNK_REF` still
  wins; plan-absent base refused with the hinted message — all without env prefix.
- Refresh verb test: a drive whose trunk advanced past the fork resolves cleanly
  via the verb, then candidate-create admits — reproducing the SL-122 scenario as a
  green test.
- Drift reporting surfaces a non-zero count when trunk moved past the merge-base.
- `just gate` green; the `DOCTRINE_TRUNK_REF=main` workaround retired from the
  dispatch skills.

## Summary

(to be written at close)

## Follow-Ups

(harvested at reconcile/close — ISS-038/IMP-122 integrate seam, IMP-124 deliver_to
config, IMP-043 reanchor remain open and adjacent.)
