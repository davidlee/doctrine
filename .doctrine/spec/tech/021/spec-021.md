# SPEC-021: Dispatch orchestrator process

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

This component is the **orchestrator-facing process** layer of the dispatch &
worktree container (`parent: SPEC-012`). SPEC-012 owns the *mechanism* — the verb
family (`provision`/`fork`/`import`/`land`/`gc`), the worker-mode guard, the
branch-point check, the born-frame git seam — each cited here by its `REQ-NNN`.
This spec owns the *process that wields them*: the **funnel cadence** (the order of
acts and the halt discipline), the **routing and concurrency decision logic**, the
**per-harness altitude contract**, the **two-stage integration projection**, and the
**load-bearing operational gotchas** that recur across real dispatch runs. It descends
from **PRD-015** (the product intent for concurrent isolated dispatch) through its parent
container SPEC-012; ADR-006 (policy-agnostic worktree posture), ADR-011 (harness-agnostic
spawn), and ADR-012 (integration topology) are the governing decisions.

The container's keystone (SPEC-012) is that *the funnel is enforced CLI mechanism, not
prose an LLM may skip*. The complement this spec records is the irreducibly-prose
residue the mechanism cannot absorb: **what order to invoke the verbs in, which arm to
route to, when to halt versus proceed, what each harness can and cannot enforce, and
the environmental footguns** (proxy-masked git, base-by-placement, patch corruption)
that a fresh orchestrator would otherwise rediscover. It is the synthesis of truth
currently scattered across the four dispatch/worktree skills (the runbook), ADR-006/
008/011/012 (the decisions), and the dispatch memory corpus (the gotchas) — collated
into the durable spec tier, citing each source, restating no verb internal.

> **Posture.** Largely **retrospective** — the cadence, routing, arm split, and the
> codex/pi and claude altitudes are shipped (SL-056, SL-064; ISS-029 resolved
> 2026-06-19). Two pieces are **forward-intent**, named where they bear: the positive
> coordination-tree marker (IMP-065, today rests on marker-*absence*) and the in-verb
> import re-anchor / content-base assertion (IMP-043, today deferred to sync-time
> report). Requirements stay `pending`; coverage is reconciled, never inferred.

## Responsibilities

Mirrors the structured `responsibilities` list. Verb *internals* are SPEC-012's and
are not restated; what follows is the process that sequences and decides among them.

### The funnel cadence — an ordered contract, halt-on-breach

Per batch, the orchestrator captures `B = git rev-parse HEAD` **before any spawn
window**, then after workers return runs, in exact order (ADR-006 D7; SL-056 §7,
SL-064 §3):

1. **Precondition** — coordination tree clean, `HEAD == B` (`branch-point-check`,
   SPEC-012 REQ-191).
2. **Delta-check** — net `B..S` is a single non-merge commit, `S^ == B`.
3. **R-5 belt** — reject any `.doctrine/` or `.claude/` touch in the `B..S` tracked
   diff (`import`'s `doctrine-touch`/`claude-touch`, SPEC-012 REQ-249).
4. **Import** — apply the surviving net delta onto `B`, **non-committing**
   (SPEC-012 REQ-249).
5. **Verify** — run the project's combined-tree verify; if RED, isolate the offender
   per delta.
6. **Branch-point guard** — coordination HEAD still `B`? (demoted under coordination-
   tree isolation, DD-5/SL-064 §3 — within a run HEAD moves only at the orchestrator's
   own commit; its real job relocates to integration-sync, where trunk may have moved).
7. **Commit** — exactly **one** commit on the coordination branch.
8. **Record** — knowledge (memory, AC evidence, notes) trails the confirmed commit.

The discipline is **report-and-halt** on conflict, moved HEAD, authored-tree touch, or
unrecoverable RED — **never auto-resolve**. The invariant is *knowledge always trails
confirmed code*: the coordination branch is the durable store, orchestrator context is
disposable, so crash ≡ handover ≡ resume-from-coordination-branch (ADR-006 D7).

### Routing and concurrency decision logic

**Arm selection** (`/dispatch` router, ADR-011 D3):

1. `doctrine.toml [dispatch] claude-force-subprocess-dispatch` (default `false`):
   `true` ⇒ `/dispatch-subprocess` (codex/pi; default `pi` until
   `preferred-subprocess-harness` wired, IMP-101).
2. otherwise route by env-marker: `.claude/` present ⇒ `/dispatch-agent` (claude
   `Agent` tool), else `/dispatch-subprocess`.
3. a self-belief↔env-marker **mismatch refuses, naming the cause** — never a blind
   spawn.

**Serial vs parallel** — `plan-next` plans **parallel batches only when file-disjoint**;
default serial (one worker per phase). The asymmetry is load-bearing: parallel
*execution* is first-class, but v1 lands **one worker per base** under the stationary-
head precondition (importing+committing worker A moves HEAD `B→B+1`, so worker B —
also forked at `B` — refuses `head-moved`). Serial-dependent phases **self-base**: the
orchestrator advances coordination HEAD to phase N's integrated tip before spawning
N+1, so the next worker forks the dependency for free (SL-056 §7c, SL-064 §7c).

### Coordination-tree placement and lifecycle

The orchestrator always runs on a dedicated, **markerless** `dispatch/<slice>`
coordination worktree, provisioned per run via `worktree coordinate` (ADR-012 D1,
SL-064 §2). Markerless because *mode, not location, decides who may write* (ADR-006
D6a): marker-absent ⇒ not worker-mode ⇒ the orchestrator may write. It is created
**inside the project root** (convention `.dispatch/SL-<n>`): under a cwd-confining jail
a `cd` to an outside sibling silently reverts to root on the next Bash call, leaving
the session on `main` and forking workers off `main` instead of `B` (ISS-029, resolved
2026-06-19; `mem.pattern.dispatch.claude-arm-coord-placement`). Bash cwd stays **parked
in the coordination tree for the whole drive loop** — this *is* how claude's
`isolation: worktree` reaches `base==B` (it forks the Bash cwd's HEAD, not the session
root; `mem.pattern.dispatch.agent-worktree-forks-bash-cwd-head`). Concurrent same-slice
dispatch is refused at creation; the worktree directory is removed at conclude while
the `dispatch/<slice>` branch is kept as a deliverable (worktree-life < branch-life,
DD-6).

### Per-harness altitude — uniform contract, honest non-uniform reach

The trust-bearing core (create-or-mark + provision + marker + per-wt env *emission*) is
harness-identical (ADR-011 D2); the reachable *altitude* is not (ADR-011 D3). Delivery
of the per-worktree env contract is subprocess-only — claude's `Agent` path has no
worker env channel and cannot consume it.

| Axis | **codex / pi** (`/dispatch-subprocess`) | **claude** (`/dispatch-agent`) |
|---|---|---|
| Spawn | subprocess (`codex exec` / pi RPC), cwd bound via `env -C "$D"` / bwrap `--chdir` | in-session `Agent` tool, `isolation: worktree` — first-class, not a degraded rung |
| Identity | disk marker (primary) **+** `DOCTRINE_WORKER` env (optimisation) | **disk marker only** — no env channel |
| Marker writer | `fork --worker` (orchestrator-owned, before any spawn window) | matcher-scoped `SubagentStart` hook `marker --stamp-subagent` (claude creates the worktree; hook provisions+stamps into `cwd`) |
| Base | **explicit `fork --base B`** | base==B by **placement** (cwd==coord tree, `baseRef='head'`) + post-spawn `verify-worker` (`merge-base --is-ancestor B HEAD`) |
| Pre-dispatch baseline-verify | **yes** (orchestrator owns `fork`) | **no** — unbuildable fork caught late at `import → verify` (a wasted worker run) |
| Worker-on-main catch | **yes** (env leg) | **no** — deferred D2b residual, mitigated by always-isolating + the hook-stamped marker |
| Build isolation | per-worktree `CARGO_TARGET_DIR` (ADR-008 D-B1) | none — shares the jail-wide target |
| OS confinement | nested bwrap (ADR-008 D-B3, marker ro-overlay *after* the rw worktree bind; never ro-bind `settings.local.json`) | none — `Agent` is not a subprocess to wrap |
| Fail-closability | full mechanism floor | SubagentStart is a **read-only event** — the stamp is **not fail-closable**; an unstamped worker is contained by the marker-absent fail-closed privilege rule + the `import` belt, not by the hook |

### Two-stage, audit-gated integration projection

The coordination branch is the funnel's SSoT; the sync verb reads the completed
`dispatch/<slice>` and projects outward in two stages (ADR-012 D4/D5, SL-064 §4):

- **Stage 1 — `dispatch sync --prepare-review`**: materialise the reviewable refs —
  `review/<slice>` (impl bundle), `phase/<slice>-NN` (code, cut from `dispatch/<slice>`
  at sync time on the claude arm so the deliverable is arm-universal, ADR-012 D3) — and
  a **journal committed to `dispatch/<slice>` before any external ref mutation**. Every
  ref update is a compare-and-swap on `expected_old_oid`. **No trunk write.**
- **Audit** runs from the parent/root context against the prepared refs (RV review
  verbs refuse on a worktree fork; the coordination worktree is removed at conclude).
- **Stage 2 — `dispatch sync --integrate`**: optional projection to trunk/`edge`,
  **opt-in, fast-forward-only, expected-tip-CAS**; a moved/non-ff target ⇒ **report,
  never auto-resolve, never force-push**.

Default routing is to `review/<slice>`, **never trunk-by-default**. The impl bundle
holds together by default; only knowledge explicitly marked slice-orthogonal projects
ahead independently (the four-bucket temporal classifier, ADR-012 D2).

### Operational gotchas as durable constraints

These are environment-and-harness realities the orchestrator must honour; each is a
durable constraint, not a single run's accident. See **Concerns** for the catalogue.

## Concerns

- **Output-rewriting proxies (rtk) silently corrupt the git plumbing the funnel reads.**
  `git diff` is stat-proxied (returns a `path | N +++` summary, not a patch), inner git
  exit codes are masked in piped/chained invocations, and `--name-only`/`rev-parse`/
  `ls-tree` can return phantom hits. Funnel guards that branch on a chained exit code or
  pipe `git diff` into `git apply` take the wrong branch silently. **Constraint:** read
  decisions from **printed output** (`ls-tree --name-only`, `diff --name-only | grep`,
  `git cherry`), capture `rc=$?` on its own line when an exit code is genuinely needed,
  and bypass the proxy with `rtk proxy git …` for any blob-level query. The **combined-
  tree project verify is the real gate** — it, not import's own fidelity check, catches
  a reverted wiring. (`mem.pattern.tooling.git-cat-file-e-exit-masked-use-ls-tree`,
  `mem.pattern.dispatch.rtk-masks-git-plumbing-during-funnel-reanchor`,
  `mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import`.)
- **`git apply` patch corruption / proxying → the checkout-import idiom.** When
  `worktree import` (or a raw `git apply --3way`) fails `corrupt patch` / `No valid
  patches in input`, substitute, running the verb's belts by hand on the trusted side:
  prove `HEAD==B`, clean tree, `S^==B`, single non-merge, R-5 clean, then
  `git checkout S -- $(git diff --name-only B..S)` (valid because the batch is disjoint
  and coord==B, so S's blobs *are* the net delta), verify `git diff S -- <paths>` empty,
  and continue the cadence. (`mem.pattern.dispatch.worktree-import-corrupt-patch-use-checkout`,
  `mem.fact.doctrine.import-corrupt-patch`.)
- **Never widen a worker's delta when integrating.** Stage **exact declared paths**;
  `git add -A` / `commit -a` sweep foreign untracked/WIP files on a shared tree (a
  foreign untracked slice TOML was swept once and amended out). The dedicated
  coordination worktree removes most of this contention *by construction* (SL-064), but
  the staging discipline stands. (`mem.system.dispatch.orchestrator-on-shared-main-contention-cost`,
  `mem.pattern.dispatch.glob-add-sweeps-foreign-untracked-on-shared-main`.)
- **Re-anchor only on a proven-disjoint HEAD move.** When HEAD legitimately moves
  between capture and import, prefer re-anchoring `B → current HEAD` over re-dispatch
  (which reproduces an identical delta) — but **only** after a per-path byte-identical
  disjointness proof (`git diff --stat <oldB>..<newHEAD> -- <each delta path>` empty,
  read raw under a proxy) and intervening commits touching only unrelated trees. A
  moved path needs a real `git apply --3way`, not checkout-import. The in-verb re-anchor
  is deferred (IMP-043). (`mem.pattern.dispatch.reanchor-base-on-disjoint-head-move`,
  `mem.pattern.dispatch.three-way-import-onto-moved-shared-main`.)
- **The landed oracle is durable git state, never a runtime receipt.** `import`'s
  `apply --3way` severs ancestry, so `branch --merged` and delta-emptiness are unsound
  reap oracles, and a gitignored "landed" flag survives a crash-before-commit and lies.
  `gc` reaps only on `git cherry` (ancestry **or** every-commit patch-id `-`); a squash
  is indistinguishable from never-landed, which is *why solo `land` must be non-squash*
  (SPEC-012 REQ-250/251). (`mem.pattern.dispatch.landed-oracle-needs-import-receipt`.)
- **Claude integration can collapse the worktree onto the parent.** The `Agent` tool's
  `isolation: worktree` may integrate the worker commit onto the parent branch on
  completion rather than leaving an isolated fork — so pre-commit gates must run
  **post-landing** on the orchestrator side, on the proven `B..S` delta, not on worker
  self-report (cross-worktree LSP/reads are stale).
  (`mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent`.)
- **Record durable memory on trunk, not in a fork.** Memory committed inside a worker
  branch is orphaned by a squash/sever (content survives, the git anchor points at a
  commit that never lands; staleness fires). (`worktree` skill; SPEC-012 NF-002 context.)
- **Identity gap, consciously accepted.** v1 rests coordination-tree write-permission on
  marker-*absence*, indistinguishable from an unstamped worker; the D2b fence (R-5 belt,
  IMP-052 post-spawn check, env worker-on-main catch, bwrap-no-push) is defence-in-depth,
  **not a coverage proof** for the full Orchestrator verb class. The real close is the
  positive marker (IMP-065). (ADR-012 OQ-D, ADR-006 D2b.)

## Hypotheses

- **The mechanism cannot absorb the order, the routing, or the footguns.** SPEC-012
  moved each funnel *step* into a verb; what stays prose is the *sequence*, the *arm
  choice*, and the *environmental gotchas*. Collating that residue into one spec — citing
  the verbs, never restating them — is higher-value than leaving it spread across four
  skills, four ADRs, and twenty memories.
- **Placement, not a ref-redirect, controls the claude base.** Because `Agent`
  `isolation: worktree` forks the Bash cwd's HEAD, parking cwd on the coordination tree
  (HEAD==B) yields `base==B` without any orchestrator-supplied base reaching a hook —
  empirically confirmed (SL-064 §8.6, controlled marker-commit test).
- **Isolation by construction beats trust.** A dedicated coordination worktree makes the
  shared-main contention surfaces (dirty foreign index, foreign-WIP collisions) *unreach-
  able*, rather than defending against them per batch (SL-064, vs the SL-060 retrospect).
- **Parallel execution, serial landing.** File-disjoint phases run concurrently for
  throughput, but landing stays one-per-base under stationary-head — cheaper and crash-
  safe versus a parallel-landing re-anchor the orchestrator would have to prove each time.

## Decisions

- **D1 — the funnel cadence is a fixed ordered contract, report-and-halt.** The
  eight-step per-batch sequence and the no-auto-resolve discipline are owned here; the
  verbs are SPEC-012's. Knowledge records only after the confirmed code commit.
- **D2 — arm routing is deterministic and refuses on disagreement.** `doctrine.toml`
  override, then env-marker; a self-belief↔env-marker mismatch refuses naming the cause.
- **D3 — parallel execution is first-class, landing is one-per-base (v1).** Serial-
  dependent phases self-base by advancing coordination HEAD before the next spawn.
- **D4 — the orchestrator runs on a dedicated markerless coordination worktree inside
  the project root.** Always-on, per-run, concurrent same-slice refused; cwd parked
  there for the drive loop; worktree-life < branch-life.
- **D5 — per-harness altitude is a uniform contract with honest non-uniform reach.**
  codex/pi reach the full floor (explicit base, env catch, pre-dispatch verify, bwrap);
  claude reaches base==B-by-placement + post-spawn `verify-worker`, marker-only, fail-
  open SubagentStart stamp, no pre-dispatch verify — confessed residuals, not parity.
- **D6 — integration is two-stage and audit-gated.** Stage-1 materialises review refs +
  a CAS journal (no trunk write); audit gates; stage-2 is opt-in, ff-only, expected-tip-
  CAS, report-never-resolve.
- **D7 — the operational gotchas are durable constraints, not run accidents.** Proxy-safe
  git reads, checkout-import on patch corruption, never-widen-the-delta, proof-gated
  re-anchor, durable-git landed oracle, memory-on-trunk — each binds every future run.
