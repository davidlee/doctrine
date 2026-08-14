# REQ-192: Enforce worker-sole-writer via a disk-marker-primary, fail-closed guard: refuse authored/Orchestrator/Hook-mint writes under `worker_mode = (is_linked_worktree && marker_present) OR env DOCTRINE_WORKER`; a marker-absent linked worktree is fail-closed; reads and the fork-side read verbs stay open; `write_class` is exhaustive (a new verb is a compile error).

## Statement

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The title line above is the
> SL-056 statement and is no longer true: there is no disk marker, no
> `is_linked_worktree` leg, and no marker-absent fail-closed class. Worker
> identity is now the `DOCTRINE_WORKER` environment variable and nothing else
> (`DEC-207`). The title is retained unchanged (titles are not rewritten on
> amendment); the statement below is what must hold.

The worker-sole-writer guard (`src/commands/guard.rs::worker_guard`) refuses
every Write- and Orchestrator-classed verb when this PROCESS is in worker mode,
and passes Read-classed verbs through.

1. **Worker mode is `env DOCTRINE_WORKER == 1`, and nothing else.** Identity is a
   property of the process, not of a tree: it is set by the same confinement
   argv that establishes the write floor
   (`scripts/spawn-confined.sh`, `jail.rs`'s `bwrap_argv` / `sandbox_exec_argv`)
   and it dies with the process. `marker::describe_mode(env_set)` is a two-row
   truth table; SL-056's was eight rows over three inputs.
2. **Topology does not participate.** A linked worktree without the variable is
   NOT in worker mode and is NOT refused; a primary tree WITH the variable IS.
   The old "a marker-absent linked worktree is fail-closed" rule is retired with
   the marker — there is no stale class left to detect, hence no `marker
   --clear`, no `status --assert` stale exit, and no `Cause` truth table.
3. **`write_class` remains exhaustive** — a new verb that is not classified is a
   compile error, so the guard cannot silently fall open on an unclassified verb.
4. **Reads and the fork-side read verbs stay open**, unchanged.

## Rationale

The disk marker was orchestrator-stamped state about a TREE, which could go
stale, be missed, or be asked about across trees; the environment variable is
carried by the process the jail creates, so the thing that ESTABLISHES the write
floor and the predicate that OBSERVES it are the same two constants and cannot
drift (STD-001). Fail-closed-on-ambiguity was the SL-056 answer to a signal that
could be absent; with a single unambiguous signal there is no ambiguity to fail
closed on. Kernel-level confinement (`--ro-bind / /`) is now the primary
enforcement; this guard is the cooperative second belt that produces a NAMED
refusal instead of an opaque EROFS.
