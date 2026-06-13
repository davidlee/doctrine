# SPEC-012: Dispatch & worktree

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Dispatch & worktree is the isolation-and-coordination container for concurrent
work. It sits beneath the whole-system root (SPEC-003) and carries **no descent**:
no PRD owns it — ADR-006 (worktree posture: policy-agnostic framework,
orchestrator-sole-writer dispatch, amended SL-056 G2) and ADR-011 (harness-agnostic
orchestrator spawn interface and per-harness capability altitude) are its governing
decisions.

The container's keystone is that **the orchestrator funnel is enforced CLI
mechanism, not prose an LLM may skip** (ADR-011 context — "mechanism in prose is the
design smell"). Worker-sole-writer rides a **disk marker the orchestrator stamps
before the worker runs** — disk is the one identity medium every harness has, an env
channel is not (ADR-011 D1). The create-or-mark + provision + per-worktree env
emission core is **harness-identical and golden-testable**; the verbs that carry the
funnel — `fork`, `import`, `land`, `gc` — are CLI verbs, refused under `worker_mode`,
not a discipline carried in skill text. What stays prose is only the per-harness
*spawn* line (subprocess vs in-session `Agent` tool), selected by the `/dispatch-*`
router.

It owns these mechanisms specific to isolation: **fork provisioning** with a
two-layer tier exclusion the copy physically cannot leak; the **orchestrator verb
family** (`fork`/`import`/`land`/`gc`) that creates, funnels, and reaps forks under
the worker-mode guard; the **worker-mode guard** — disk-marker-primary, fail-closed
on a marker-absent linked worktree; the **branch-point guard**, a HEAD-stationarity
assertion at the batch-commit boundary; and the **born-frame git seam** that confines
all git/disk/process impurity to one shell. Shared substrate — identity, the atomic
claim, id allocation, the scaffold/render pipeline, the storage rule and the
pure/imperative split as system-wide principles — lives in the parent (SPEC-003) and
the entity-engine container (SPEC-004) and is not restated here. Trunk-side id minting
and the reseat verb that resolves offline collisions (ADR-006 D3/D8) belong to the
id-lifecycle container; this container provides the isolation those acts run beneath.

> **Posture.** The funnel-verb family (`fork`/`import`/`land`/`gc`), the marker
> guard, and the per-harness spawn paths are **forward-intent** — authored downstream
> of the locked ADR-006 amendment and ADR-011 (SL-056), landing as code across SL-056
> PHASE-05+. The three read verbs (`provision`/`check-allowlist`/`branch-point-check`)
> and the born-frame git seam are **shipped**. Requirements stay `pending`; coverage
> is reconciled, never inferred.

## Responsibilities

Mirrors the structured `responsibilities` list: provision a fork as the sole copier
with guaranteed tier exclusion; carry the funnel as an orchestrator verb family
(`fork`/`import`/`land`/`gc`) refused under the worker-mode guard; enforce
worker-sole-writer via a disk-marker-primary, fail-closed-on-ambiguity guard; assert
HEAD-stationarity at the batch boundary; capture the impure born frame for anchoring;
and defend tier merge-safety by the tier's absence in the fork.

### Fork provisioning — the sole copy path with guaranteed tier exclusion

`worktree provision <fork>` is the **only** copy path into a fork. The pure core
(`select_copies`, `parse_allowlist`, `is_withheld`) takes paths and strings as
inputs — no disk, git, clock, or rng (ADR-001 leaf) — and the thin impure shell
(`run_provision`) reads `.worktreeinclude`, drives `git ls-files`/`rev-parse`
through the `git.rs` runners, and copies via the `fsutil` safe-copy helper. The
exclusion is **two-layer** (design OQ-3-B): `select_copies` is the *guarantee* — it
drops any file matching the coordination/runtime tier even when a broad `**`
allowlist would otherwise admit it, so the copy physically cannot leak the tier;
`allowlist_violations` behind `check-allowlist` is a static *smell test* whose green
result is explicitly **not** completeness. The withheld tier — the five `Tier`
variants: `.doctrine/state/`, the relative `phases` symlink, `handover.md`,
inquisition scratch, and memory caches — is classified in `is_withheld` by `Tier`;
the worker marker (`.doctrine/state/dispatch/worker`) inherits every withheld-tier
exclusion with no new tier logic. As a fail-fast convenience, provision aborts before
copying if any `.worktreeinclude` pattern names a withheld tier — the same smell test,
not a substitute for the copy-time `select_copies` guarantee that runs regardless —
and `verify_sibling_worktree` refuses to provision the source tree onto itself.

### The orchestrator verb family — `fork` / `import` / `land` / `gc`

The funnel is four `Orchestrator`-classed verbs, each refused under `worker_mode`
(they mutate git refs/dirs — create/remove worktrees, delete branches, merge commits,
reap dirs; classifying them `Read` because they spare the authored TOML corpus would
be a category error, ADR-006 D2/D2a).

- **`fork --base <B> --branch <name> --dir <path> [--worker]`** (codex/pi
  orchestrator-owned creation): one act — `git worktree add -b <branch> <dir> <B>`,
  then `provision` (sole copier, withheld excluded), then (if `--worker`) stamp the
  marker **before any spawn window**, then emit the per-worktree env contract on
  stdout. It is **compensating cleanup, not a transaction** — git mutations are not
  atomic, so any failure after `git worktree add` triggers a best-effort rollback
  (`git worktree remove --force` + `git branch -D` + reap dir); a rollback that
  itself fails **names the leftover and exits non-zero**.
- **`import --base <B> --fork <branch>`** — the dispatch funnel (single distilled
  worker commit, ancestry severed). v1 is the stationary-head case, each step a hard
  refusal, no auto-merge: precond `HEAD == B` (`branch-point-check`) **and** a clean
  tree (tracked+staged only); `S^ == B` (single non-merge delta); the **belt** rejects
  if the `B..S` tracked name-only diff touches `.doctrine/` (`doctrine-touch`) or
  `.claude/` (`claude-touch`); then `git apply --3way --index` (non-committing — the
  orchestrator commits **separately**, ADR-006 D7 cadence). **No runtime receipt is
  stamped** — a flag born before the commit would survive a crash and lie "landed" to
  `gc`.
- **`land --fork <branch>`** — solo `/execute`'s analog (multi-commit branch, ancestry
  preserved): `git merge --no-ff <branch>`, **structurally non-squash** (see D7). It
  refuses a marker-bearing fork (`dispatch-fork` — that delta must funnel through the
  belted `import`) and a worktree-gone fork (`worktree-gone` — the marker is
  unreachable, so provenance cannot be verified); a conflicted merge is **aborted
  before** the refusal (a half-merge wedges the tree against the verb's own re-entry
  guard).
- **`gc --fork <branch> [--superseded-head <SHA>] [--force] [--dry-run]`** — reaps
  worktree + branch + target-dir in one act, **only** when the fork's commit *provably
  landed* against durable git state via `git cherry <coordination-HEAD> <fork-branch>`:
  **ancestry** (the `land` route) **OR** every listed commit is `-` (the `import`
  route's patch-id). It is an idempotent state machine — a `gc` that crashed between
  destructive steps completes on rerun or names the leftover. The `--superseded-head
  <SHA>` reaps a re-dispatched (spent-but-never-landed) fork iff `<SHA>` matches the
  branch head (a TOCTOU movement-guard, not a landing proof).

### The worker-mode guard — disk-marker-primary, fail-closed on ambiguity

Worker-sole-writer is enforced in the CLI by a guard in `run()`, before dispatching a
write-classed **or** `Orchestrator`/`Hook-mint`-classed `Command` (ADR-006 D2a,
ADR-011 D1):

```
worker_mode(root) := (is_linked_worktree(root) && marker_present(root))  // PRIMARY, agnostic
                     OR env DOCTRINE_WORKER set                          // codex/pi worker-on-main catch
if worker_mode(root): refuse(verb)   // names the verb
```

The **disk marker is primary and harness-agnostic** (presence-only, no contents, at
the withheld-tier path `.doctrine/state/dispatch/worker`); `DOCTRINE_WORKER` is a
**codex/pi optimisation, not the identity** — its one job is to catch the
*worker-on-main* hazard (a harness that drops the worker on the coordination root,
where no marker exists). **Critically, a linked worktree whose marker is *absent* is
treated fail-CLOSED** — the Orchestrator/Hook-mint/write classes are *refused* there,
not trusted as the orchestrator. This closes two fail-opens at once: the SubagentStart
stamp-failure case on claude (a worker whose stamp hook errored runs
`marker_present == false`, and SubagentStart is a read-only event that cannot abort
it — SL-056 PHASE-02/03), and the deliberate marker self-clear (clearing now *refuses*
the privileged verbs rather than enabling them). The legitimate orchestrator is
unaffected — it runs at the coordination root (`!is_linked_worktree`); the sole
marker-minting verb (`marker --stamp-subagent`) is exempt **by verb identity**, not by
location. `write_class` is an **exhaustive** match over every `Command` variant with
no wildcard arm — a future verb is a compile error, never a silently-permitted write
(design X4). Reads stay open, and `provision`/`check-allowlist`/`branch-point-check`/
`status` are deliberately `Read` (they write *fork* files, not the doctrine state the
guard protects). `marker --clear --operator` is a bespoke fifth class — refused by
env-set, cwd-not-tree-root, and the linked-worktree `--operator` accident-fence, but
**never by the marker conjunct itself** (locking the marker's only remover behind the
marker is the self-brick). `worktree status [--assert]` derives the resolved mode and
its cause from one `describe_mode` core — the human line and the `--assert` exit never
disagree — and names the `marker --clear --operator` remedy on a `stale-marker`.

Raw-tree confinement (a worker hand-editing a file or running a bare `git commit`) is
**not** CLI-stoppable (ADR-006 D2b) and is honestly deferred to sandbox/harness work
(ADR-008), contained on the dispatch funnel by `import`'s `.doctrine/`/`.claude/`
belt — not papered over here.

### The branch-point guard — HEAD-stationarity, not merge-base

`worktree branch-point-check --base B` asserts that coordination HEAD still equals
the orchestrator's pre-spawn captured base `B`. It is a **ref-equality compare**
(`matches(base, head)`) — exit 0 when HEAD is stationary, exit 1 ⇒ re-dispatch from
the moved HEAD. Because a file-disjoint batch imports onto the single `B` and commits
once, HEAD moves only at the orchestrator's own batch commit; a mismatch therefore
names an *external* mover. This is a stationarity check, never a merge-base
computation (design C-V).

### The born-frame git seam — impurity confined, normalizers byte-for-byte

`git.rs` is the impure born-frame producer for memory anchoring (SL-007). It confines
all git, disk, and process impurity to one module: the `git_bytes`/`git_text`
runners, `capture` of the working-tree frame, remote selection, and submodule
rejection. It reproduces `forgettable`'s normalizers **byte-for-byte** —
`forget.remote.v1` (`normalize_remote_url`, versioned by the `REMOTE_NORMALIZER` tag)
for routing-identity remote-URL normalization, and `forget.checkout.v1`
(`checkout_state_id`, the `CHECKOUT_NORMALIZER` tag) for content-bearing dirty-tree
hashing — so a `repo_id` and checkout-state id derived here match the event-store's
exactly. The version tags make the algorithm replaceable without silently re-anchoring
existing memories.

### Tier merge-safety by construction

The container defends ADR-006 D4 not by trust but by absence: the coordination/runtime
tier is never copied into a fork, so a worker has nothing shared-mutable to corrupt —
a copied phase sheet would be invisibly mutable across worktrees. The orchestrator's
pre-distilled worker prompt (ADR-006 D6) substitutes for the withheld coordination
state; provisioning substitutes for the absent execution environment. No central index
or counter exists to reintroduce a conflict.

### Per-harness altitude — uniform contract, honest non-uniform reach

The create-or-mark + provision + marker + per-worktree-env-*emission* core is
harness-identical (ADR-011 D2); the reachable *altitude* is not (ADR-011 D3). The
contract states honestly what each harness can and cannot enforce — no uniform-altitude
lie, and no harness-specific command (`claude -p`) is ever a required element.

- **codex/pi (`/dispatch-subprocess`).** Subprocess spawn (`codex exec`/pi) binds the
  worker cwd to the fork via `env -C "$D"` / bwrap `--chdir`. Identity = disk marker
  (primary) **+** `DOCTRINE_WORKER` env (optimisation, catching worker-on-main). Marker
  writer = the orchestrator-owned `fork --worker`. Base is **explicitly pinned**
  (`fork --base <B>`). Per-worktree build isolation via `CARGO_TARGET_DIR` (ADR-008
  D-B1, a project-local consumer of the env contract — never a framework primitive);
  nested bwrap is the OS floor (ADR-008 D-B3, spike-contingent). The orchestrator owns
  `fork`, so it **baseline-verifies** the fork before handoff.
- **claude (`/dispatch-agent`).** In-session `Agent` tool, `isolation: worktree` — a
  **first-class** backend, not a degraded rung. Identity = **disk marker only** (no env
  channel). Marker writer = a **matcher-scoped, sync-blocking `SubagentStart` hook**
  running `marker --stamp-subagent`: Claude performs its own default worktree creation,
  and the hook then provisions and stamps the marker into the payload `cwd`. Because the
  hook blocks the worker until it exits, the marker is present before the worker's first
  command **on hook success**. The create-replacing `WorktreeCreate` `create-fork`
  one-act path is **deferred** — the deployed payload carries no `agent_type`/
  `worktree_path`/base (SL-056 PHASE-02). Two altitude concessions, named not papered:
  (1) **not fail-closable** — `SubagentStart` is a read-only event, so a stamp-failure
  leaves an *unstamped* worker, contained by the marker-absent fail-closed privilege
  rule above, not by the hook; (2) **the pre-dispatch baseline-verify guarantee does
  not hold** (accepted weaker class) — Claude creates the worktree at spawn with no
  pre-spawn orchestrator moment, so an unbuildable fork is caught **late, at the
  funnel's `import → verify`**, at the cost of a wasted worker run. The **base is opaque
  and Claude-chosen** (ADR-011 D5/M1) — not orchestrator-controlled, so a
  clean-applying-but-semantically-wrong import is possible (not merely a wasted run); the
  closing import-time content-base assertion is deferred to IMP-043. No per-worktree
  target, no bwrap; worker-on-main is **not** caught (the deferred D2b residual,
  mitigated by always-isolating + the hook-stamped marker).

## Concerns

- **Raw-tree confinement is the deferred residual, not the funnel.** The funnel
  itself is enforced CLI mechanism (the `Orchestrator` verbs + the worker-mode guard).
  What the CLI *cannot* stop is a worker hand-editing a file or running a bare
  `git commit` (ADR-006 D2b); the harness does not confine workers to their worktree.
  This is a known live risk, deferred to sandbox/harness work (ADR-008), and contained
  on the dispatch funnel by `import`'s `.doctrine/`/`.claude/` belt — not papered over.
- **The claude altitude is weaker, and that is stated, not hidden.** SubagentStart-stamp
  is **not fail-closable** (read-only event); the stamp-failure case is contained by
  the marker-absent fail-closed privilege rule, claude has **no pre-dispatch
  baseline-verify** (caught late at `import → verify`), and its **base is opaque**
  (a clean-applying-semantically-wrong import is possible, IMP-043 deferred). codex/pi
  keep the explicit base and pre-dispatch gate.
- **The import belt's scope is honest and narrow.** The `.doctrine/`/`.claude/`
  rejection belt is the **dispatch/import-path** containment, not an unconditional
  all-funnel guard — solo's `land` is a second, **beltless** sanctioned funnel (a
  trusted self-orchestrator legitimately lands doctrine). Because the diff is
  tracked-files-only and *all* of `.claude/` is gitignored, the `.claude/` leg contains
  **only force-add injection** (`git add -f` of a `.claude/` path) — normal installer/
  harness output never enters the diff and cannot ride back. The worker running the
  installer at all is a separate concern, handled by the `claude install` write-class
  refusal (ADR-006 D2, not ride-back).
- **v1 import requires a quiescent coordination branch.** Stationary-head import
  refuses on any HEAD mover — external *or* the orchestrator's own batch (υ). A live
  main mandates delta-branch coordination (ADR-006 D8 team mode); solo-on-main dispatch
  is safe only when main is quiescent. Even with no external committer, v1 lands **one
  worker per base** (not a whole batch): importing+committing worker A moves HEAD
  `B→B+1`, so worker B — also forked at `B` — refuses `head-moved`. Parallel
  *execution* is first-class; parallel *landing* is not. The orchestrator detects the
  moved HEAD and reports the mover rather than silently re-dispatching into livelock;
  the in-verb re-anchor is deferred (IMP-043).
- **Smell test is not a guarantee.** `check-allowlist` green never means the fork is
  clean; only `select_copies` guarantees tier exclusion. The two must not be conflated.
- **Normalizer fidelity is load-bearing.** A drift from `forgettable`'s byte-for-byte
  algorithm would silently mis-anchor memory; the version tags are the seam that makes
  a deliberate change visible rather than silent.

## Hypotheses

- **Mechanism in a verb, not in prose.** Moving each funnel step into a CLI verb makes
  the worktree/dispatch skills shorter **and** more harness-agnostic in one edit, and
  makes trust golden-testable instead of resting on a prose ritual an LLM may skip
  (ADR-011 context).
- **Identity rides disk, not an env seam.** A disk marker is the harness-agnostic floor
  because disk is the one medium every harness has; an env channel is not (claude's
  `Agent` tool has none). `DOCTRINE_WORKER` is an optimisation of the marker, never the
  identity.
- **Fail-closed on ambiguity.** A linked worktree with no marker is refused, not
  trusted — so a stamp-failure or a self-clear *loses* privilege rather than gaining it.
- **Exclude by construction, not by trust.** Withholding the coordination/runtime tier
  from the fork outright is preferred over copying it and trusting workers not to mutate
  it — the tier's *absence* is what makes worker-sole-writer free.
- **Stationarity over merge-base.** Asserting HEAD has not moved from the captured base
  is cheaper and more direct than a merge-base, and sufficient because the batch commits
  exactly once onto `B`.
- **One impure git module.** Concentrating all git/disk/process impurity in `git.rs`
  keeps the rest of the container testable as pure functions, honouring the system-wide
  pure/imperative split.

## Decisions

- **D1 — provision is the sole copier, exclusion is a guarantee not a check.**
  `select_copies` drops the coordination/runtime tier even under a broad allowlist;
  `check-allowlist` is a static smell test whose green result is explicitly not
  completeness.
- **D2 — the branch-point check is ref-equality, not merge-base.** It asserts
  coordination HEAD still equals the pre-spawn base `B`; a mismatch means an external
  mover ⇒ re-dispatch, never auto-merge.
- **D3 — worker-mode is enforced in the CLI by a disk-marker-primary, fail-closed
  guard.** `worker_mode = (is_linked_worktree && marker_present) OR env
  DOCTRINE_WORKER`; the marker is the harness-agnostic primary, env a codex/pi
  worker-on-main optimisation. A **marker-absent linked worktree is fail-CLOSED** — the
  write/`Orchestrator`/`Hook-mint` classes are refused there (closing both the
  SubagentStart stamp-failure case and the marker self-clear). `write_class` is a
  wildcard-free exhaustive match (a new verb is a compile error); the marker-minting
  verb is exempt by verb identity, not location. (Rewritten from the prior
  env-fails-open framing — ADR-006 D2a / ADR-011 D1, SL-056 G2.)
- **D4 — the born frame is impure-isolated and normalizer-versioned.** All git
  impurity lives in `git.rs`; `forget.remote.v1`/`forget.checkout.v1` are reproduced
  byte-for-byte and tagged so the algorithm is replaceable without silent re-anchoring.
- **D5 — the funnel is an enforced verb family, not a prose discipline.** `fork`,
  `import`, `land`, `gc` are `Orchestrator`-classed CLI verbs (refused under
  `worker_mode`) that carry creation, the dispatch funnel, the solo merge, and reaping
  — each a pure classifier (`classify_import`/`classify_land`/`classify_gc`) over an
  impure git shell. The trust-bearing core (create-or-mark + provision + marker + per-wt
  env *emission*) is harness-identical and golden-testable (ADR-011 D2/D5).
- **D6 — per-harness altitude is a uniform contract with honest non-uniform reach.**
  codex/pi reach the full mechanism floor (explicit base-pinning, env-arm, per-wt env
  delivery, pre-dispatch baseline-verify, bwrap); claude reaches an **O3-red
  SubagentStart-stamp** altitude — marker-only, **not fail-closable** (contained by the
  marker-absent rule), **no pre-dispatch baseline-verify** (caught late at import), and
  an **opaque Claude-chosen base** (a confessed residual, not parity — IMP-043). No
  harness-specific command is a required element (ADR-011 D3/D5/D6).
- **D7 — the funnel's honest scope: belt narrow, solo non-squash, import quiescent.**
  (a) The `.doctrine/`/`.claude/` belt is the import/dispatch-path containment only; the
  `.claude/` leg contains exactly **force-add injection** (the rest of `.claude/` is
  gitignored and invisible to the tracked-files diff); solo's `land` is a second,
  beltless sanctioned funnel. (b) Solo **must** land via the **structurally non-squash**
  `land` (`git merge --no-ff`) so `gc`'s ancestry leg and memory-anchor sha-stability
  both hold — a squash-merge is structurally uncertifiable. (c) v1 `import` requires a
  **quiescent** coordination branch and lands **one worker per base**; a live main
  mandates delta-branch coordination (ADR-006 D8).
