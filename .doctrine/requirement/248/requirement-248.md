# REQ-248: `worktree fork --base B --branch N --dir P [--worker]` creates the worktree, provisions it (sole copier, withheld excluded), stamps the worker marker before any spawn window, and emits the per-worktree env contract — Orchestrator-classed, with compensating rollback that names any leftover on failure.

## Statement

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The title line above retains the
> SL-056 statement. Its "stamps the worker marker before any spawn window" clause
> is falsified: the disk marker is gone and worker identity is now the
> `DOCTRINE_WORKER` environment variable set by the confinement argv at spawn
> (`DEC-207`). Its "emits the per-worktree env contract" clause was already dead
> before SL-254 — see the note under point 4; that drift is NOT SL-254's and is
> recorded, not repaired, here. The title is retained unchanged; the statement
> below is what must hold.

`doctrine worktree fork --base B --branch N --dir P [--worker] [--slice N
--phase PHASE-NN]` creates an orchestrator-owned worktree fork off `B`:

1. **Create then provision.** It creates the worktree and provisions it through
   `worktree provision` — the sole copier — with every withheld
   coordination/runtime-tier path excluded (REQ-189).
2. **No stamping.** It stamps nothing. There is no disk marker to stamp and no
   spawn window to stamp before: worker identity is established *at spawn* by
   `scripts/spawn-confined.sh`, which sets `DOCTRINE_WORKER=1` in the same argv
   that establishes the jail's write floor. `--worker` no longer selects a
   stamping behaviour; it selects only whether the fork is a candidate for the
   durable `(slice, phase)` binding.
3. **Binding is best-effort and may be a no-op.** It binds the fork to its
   `(slice, phase)` only when ALL of `--worker`, both `--slice` and `--phase`,
   and a `dir` matching the `<coord>/.worktrees/<name>` layout the resolver
   strips. Otherwise the fork is simply UNBOUND, which downstream verbs report
   honestly as `unprovable-fork` rather than guessing.
4. **No env contract is emitted.** Since SL-156 the fork compiles into its own
   in-tree `<dir>/target` and `fork.rs` emits no `KEY=value` lines at all — the
   platform exited the build-env business. The title's emission clause therefore
   describes pre-SL-254 drift inherited from SL-156, not anything SL-254 changed.
   Flagged for the orchestrator; not repaired under this slice (see REQ-252).
5. **Orchestrator-classed**, so `worker_guard` refuses it under worker mode, with
   compensating rollback that NAMES any leftover on failure
   (`fork-rollback-debris`).

## Rationale

Provisioning through one copier is what keeps tier exclusion true by
construction rather than by a trusted check (REQ-304). Moving identity out of the
tree and into the spawning process removes the whole stamp/stale/cure surface: a
marker had to be written before a spawn window and could be missed or go stale,
whereas an environment variable set by the confinement argv cannot exist without
the confinement that carries it, and dies with the process.
