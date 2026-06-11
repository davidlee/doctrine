# SPEC-012: Dispatch & worktree

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Dispatch & worktree is the isolation-and-coordination container for concurrent
work. It sits beneath the whole-system root (SPEC-003) and carries **no descent**:
no PRD owns it — ADR-006 (worktree posture: policy-agnostic framework,
orchestrator-sole-writer dispatch) is its governing decision. The container is
deliberately small in code: the orchestrator *funnel* is a doctrine-mediated
discipline carried by the `dispatch` skill, not a Rust subsystem, so what ships in
the binary is exactly the machinery the funnel cannot enforce in prose — three
fork-side worktree verbs, the CLI worker-mode guard, and the impure git seam that
anchors memory.

It owns four mechanisms specific to isolation: **fork provisioning** with a
two-layer tier exclusion that the copy physically cannot leak; the **branch-point
guard**, a HEAD-stationarity assertion at the batch-commit boundary; the **CLI
worker-mode guard** that refuses doctrine-mediated writes under `DOCTRINE_WORKER=1`;
and the **born-frame git seam** that confines all git/disk/process impurity to one
shell. Shared substrate — identity, the atomic claim, id allocation, the
scaffold/render pipeline, the storage rule and the pure/imperative split as
system-wide principles — lives in the parent (SPEC-003) and the entity-engine
container (SPEC-004) and is not restated here. Trunk-side id minting and the reseat
verb that resolves offline collisions (ADR-006 D3/D8) belong to the id-lifecycle
container; this container provides the isolation those acts run beneath.

## Responsibilities

Mirrors the structured `responsibilities` list: provision a fork as the sole
copier with guaranteed tier exclusion; classify and smell-test the withheld
coordination/runtime tier; assert HEAD-stationarity at the batch boundary; enforce
the worker-sole-writer invariant in the CLI; capture the impure born frame for
anchoring; and defend tier merge-safety by the tier's absence in the fork.

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
result is explicitly **not** completeness. The withheld tier — `.doctrine/state/`,
the relative `phases` symlink, `handover.md`, memory caches — is classified in
`is_withheld` by `Tier`. Provision refuses outright if any `.worktreeinclude`
pattern names a withheld tier, and `verify_sibling_worktree` refuses to provision
the source tree onto itself.

### The branch-point guard — HEAD-stationarity, not merge-base

`worktree branch-point-check --base B` asserts that coordination HEAD still equals
the orchestrator's pre-spawn captured base `B`. It is a **ref-equality compare**
(`matches(base, head)`) — exit 0 when HEAD is stationary, exit 1 ⇒ re-dispatch from
the moved HEAD. Because a file-disjoint batch imports onto the single `B` and
commits once, HEAD moves only at the orchestrator's own batch commit; a mismatch
therefore names an *external* mover. This is a stationarity check, never a
merge-base computation (design C-V).

### The CLI worker-mode guard — worker-sole-writer enforced where doctrine owns it

A dispatched worker self-arms `DOCTRINE_WORKER=1`. The guard in `main` bails before
dispatch on any write-classed verb, with a refusal naming the verb. The
classification (`write_class`) is an **exhaustive** match over every `Command`
variant with no wildcard arm — a future variant is a compile error, never a
silently-permitted write (design X4). Reads stay open (a worker may read doctrine
state freely), and the three worktree verbs are deliberately `Read`: they write
*fork* files, not the doctrine state the guard protects. This is the enforceable
half of ADR-006 D2a; raw-tree confinement (a bare `git commit`) is *not*
CLI-stoppable (D2b) and is left to the orchestrator's import-time belt and to
harness/sandbox work, not papered over here.

### The born-frame git seam — impurity confined, normalizers byte-for-byte

`git.rs` is the impure born-frame producer for memory anchoring (SL-007). It
confines all git, disk, and process impurity to one module: the `git_bytes`/
`git_text` runners, `capture` of the working-tree frame, remote selection, and
submodule rejection. It reproduces `the external decision register`'s normalizers **byte-for-byte** —
`forget.remote.v1` (`normalize_remote_url`, versioned by the `REMOTE_NORMALIZER`
tag) for routing-identity remote-URL normalization, and `forget.checkout.v1`
(`checkout_state_id`, the `CHECKOUT_NORMALIZER` tag) for content-bearing dirty-tree
hashing — so a `repo_id` and checkout-state id derived here match the event-store's
exactly. The version tags make the algorithm replaceable without silently
re-anchoring existing memories.

### Tier merge-safety by construction

The container defends ADR-006 D4 not by trust but by absence: the
coordination/runtime tier is never copied into a fork, so a worker has nothing
shared-mutable to corrupt — a copied phase sheet would be invisibly mutable across
worktrees. The orchestrator's pre-distilled worker prompt (ADR-006 D6) substitutes
for the withheld coordination state; provisioning substitutes for the absent
execution environment. No central index or counter exists to reintroduce a
conflict.

## Concerns

- **The funnel is a discipline, not enforced code.** Import-all → verify →
  branch-point guard → one commit → record is carried by the `dispatch` skill and
  run by the orchestrator on the trusted side. The binary supplies only the
  branch-point verb and the worker-mode guard; report-and-halt-never-auto-merge is
  a policy the code cannot compel.
- **`DOCTRINE_WORKER=1` fails open.** The env contract is self-armed in the worker
  prompt; nothing in the harness sets it, so an unarmed worker runs with the CLI
  fully open. The real protection against authored-tree touches is the
  orchestrator's import-time net-diff belt, which runs worker-mode OFF on the
  trusted side — not the env var.
- **Smell test is not a guarantee.** `check-allowlist` green never means the fork
  is clean; only `select_copies` guarantees tier exclusion. The two must not be
  conflated.
- **Normalizer fidelity is load-bearing.** A drift from `the external decision register`'s
  byte-for-byte algorithm would silently mis-anchor memory; the version tags are
  the seam that makes a deliberate change visible rather than silent.

## Hypotheses

- **Exclude by construction, not by trust.** Withholding the coordination/runtime
  tier from the fork outright is preferred over copying it and trusting workers not
  to mutate it — the tier's *absence* is what makes worker-sole-writer free, with
  no shared-mutable file to defend.
- **Enforce only where doctrine owns the surface.** The CLI guard covers exactly
  the writes that mint ids and anchor memory, because that is the surface doctrine
  controls; raw-tree confinement is honestly deferred rather than faked, keeping the
  guarantee truthful.
- **Stationarity over merge-base.** Asserting HEAD has not moved from the captured
  base is a cheaper, more direct safety property than computing a merge-base, and is
  sufficient because the batch commits exactly once onto `B`.
- **One impure git module.** Concentrating all git/disk/process impurity in
  `git.rs` keeps the rest of the container (and the corpus) testable as pure
  functions, honouring the system-wide pure/imperative split named at the root.

## Decisions

- **D1 — provision is the sole copier, exclusion is a guarantee not a check.**
  `select_copies` drops the coordination/runtime tier even under a broad allowlist;
  `check-allowlist` is a static smell test whose green result is explicitly not
  completeness.
- **D2 — the branch-point check is ref-equality, not merge-base.** It asserts
  coordination HEAD still equals the pre-spawn base `B`; a mismatch means an
  external mover ⇒ re-dispatch, never auto-merge.
- **D3 — worker-mode is enforced in the CLI, exhaustively, and fails open at the
  env boundary.** `write_class` is a wildcard-free match (a new verb is a compile
  error); the real authored-tree protection is the orchestrator's trusted
  import-time belt, not `DOCTRINE_WORKER=1`.
- **D4 — the born frame is impure-isolated and normalizer-versioned.** All git
  impurity lives in `git.rs`; `forget.remote.v1`/`forget.checkout.v1` are
  reproduced byte-for-byte and tagged so the algorithm is replaceable without
  silent re-anchoring.
