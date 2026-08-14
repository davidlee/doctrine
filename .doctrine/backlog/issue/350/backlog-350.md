# ISS-350: Source-delta registry resolves to the primary worktree while phase state resolves locally

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

Two functions in `src/state.rs` resolve the *same directory* by different roots:

| function | root | line |
|---|---|---|
| `boundaries_path()` — the source-delta registry, read **and** write | `git::primary_worktree(cwd)` | `state.rs:943` |
| `phases_dir()` — phase sheets / completion status | the local `project_root` | `state.rs:135` |

`registry_completeness(cwd, project_root, slice_id)` (`state.rs:1208`) consumes
both in one call: it reads recorded deltas by `cwd` (→ primary) and completed
phases by `project_root` (→ local). Both artefacts live in
`.doctrine/state/slice/<NNN>/`.

So a linked worktree that carries its own runtime state reports **`10/10` phases
and zero source deltas at the same time**, and `slice conformance` degrades to
`unavailable — no recorded source deltas` while a complete registry sits in the
tree the command was run from.

## How it surfaced

`SL-254` was built inside a microVM capsule and delivered over the capsule
sideband (`microvm-spike` ledger item 32): `refs/capsule/a/heads/work` for the
code, `refs/capsule/a/state/implementation` for the runtime tier, fetched
atomically so *"nobody observes a result commit without the capsule state that
goes with it"*. The state half landed at
`.worktrees/SL-254-audit/.doctrine/state/slice/254/boundaries.toml` with **ten
rows, `provenance = "solo"`** — every phase boundary recorded automatically as
the phases landed.

`SL-254`'s audit (`RV-356` `F-15`) ran `slice conformance` from that worktree and
got `unavailable`. Both plausible diagnoses were wrong, and the audit recorded
two superseded findings (`F-7`, `F-14`) before reading the resolver:

- **not** a collection failure — the sideband collected it, atomically, with a
  `code-oid` provenance pin;
- **not** raw commits bypassing the CLI — the automatic solo phase-binding fired
  on every `slice phase --status completed`.

The registry was simply read from a tree that had never heard of the slice.

## The second-order hazard

The write half is primary-pinned too, and warns about nothing. `SL-254`'s audit
ran ten `slice record-delta` calls from the primary tree to "bootstrap" what it
believed was an empty registry; they landed as `provenance = "manual"` rows in a
tree that had never built the slice, **silently shadowing the authoritative
`provenance = "solo"` registry** that already existed elsewhere.

Nothing refused, nothing warned, and the shadowing was only noticed because the
audit later went looking for where `record-delta` writes. A tree can accumulate
registry rows for slices whose work it does not contain.

*(No harm in that instance: the two registries produce byte-identical conformance
output — undeclared 96 / undelivered 0 / conformant 40 — verified by swapping the
files. The authentic registry was restored.)*

## Why the pinning exists, and where its boundary really is

`primary_worktree()` is **correct** for a *worker fork of this repo*: a fork must
not keep its own registry, or the orchestrator's conformance view fragments
across N forks. That is the case the pinning was written for and it should keep
working.

It is **wrong** for an *adopted foreign checkout* — a worktree whose history was
developed elsewhere and whose runtime state arrived with it. Doctrine has no
predicate distinguishing the two today, and the capsule workflow makes the second
case routine rather than exotic.

## Fix directions

1. **Recommended — one root per slice's runtime-state directory.** Adopt the
   invariant that `.doctrine/state/slice/<NNN>/` resolves by a single root, and
   deliver it with a rule that needs no new concept: read locally when the linked
   worktree holds its own `.doctrine/state/slice/<NNN>/` for a slice the primary
   does not know; fall back to primary otherwise. Preserves the worker-fork
   semantics (a fork has no state dir of its own for the slice) and fixes the
   write hazard for free.
2. **Promote the registry to the authored tier.** Source-delta rows would then
   land with the branch, which also kills the residual that runtime state must be
   hand-copied into whichever tree reconciles. Bigger change, and it puts derived
   evidence in the committed tier — a storage-rule argument worth having on its
   own rather than settling here.
3. **Minimum viable, if neither lands soon** — have `record-delta` and
   `slice conformance` say which tree they resolved to. The whole misdiagnosis
   came from a silent root choice; naming it in the output would have collapsed
   three findings into zero.

## Residual regardless of fix

Runtime state does not travel with a branch. Whichever tree performs
reconciliation still needs the slice's `.doctrine/state/slice/<NNN>/` copied
across from the adopted worktree unless fix direction 2 is taken.

## Verification

- `src/state.rs:135`, `:943`, `:1208` — the three resolution sites.
- `.worktrees/SL-254-audit/.doctrine/state/slice/254/boundaries.toml` — ten rows,
  `provenance = "solo"`, delivered by the sideband.
- Swap test: authentic vs hand-reconstructed registry → identical conformance
  output, so the algebra is not implicated.
- A regression test wants an adopted-worktree fixture (own state dir, slice
  unknown to the primary) asserting `slice conformance` resolves rather than
  reporting `unavailable`.
