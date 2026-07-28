# Worker-guard honours explicit project root

## Context

`worker_guard` (`src/commands/guard.rs`) resolves the project root from **CWD**
— `crate::root::find(None, &default_markers())` — and therefore tests the
worker-confinement marker against a tree the command is not operating on. When a
command is invoked with an explicit `-p <root>`, the guard still walks up from
CWD, finds an enclosing worktree's `.doctrine/state/dispatch/worker` marker, and
refuses a write that was never aimed at a marked tree.

This is the sole *unintentional* CWD-rooted resolution in the codebase. The two
other `root::find(None, …)` sites (`src/commands/cli.rs`, `src/state.rs`) carry
comments marking them as deliberate; `state.rs`'s is the inverse concern (a
worker recording a row a later integrator reads from the main tree).

**Consequence.** Inside a stamped worker fork a dispatch worker has no reliable
local green signal: ~51 of 85 `tests/e2e_*.rs` drive the CLI with `-p <temp
root>` without setting `current_dir`, so all are latently exposed. Combined with
the `just validate` fork skip (DEC-003) the worker burns a `worker_commit` round
trip to discover an ordinary gate failure, and each refusal returns the whole
transcript (ISS-219). The three compound.

Originating records: ISS-028 (canonical), ISS-267 (test-side symptom), ISS-240
(closed duplicate), ISS-260 (resolved predecessor, established the
`test_support::under_worker_marker()` seam).

### Why the recorded fix direction does not work

ISS-028 proposes "threading the explicit `-p` root into the worker-marker
lookup". Research (pre-design, 2026-07-28) established that the obvious form of
this is structurally incapable:

- `write_class` matches on the **outer** `Command` enum. Only **9 of 33**
  Write-classed variants carry a top-level `path` field.
- The other 24 — including `Slice`, `Memory`, `Adr`, `Backlog`, `Review`,
  `Worktree`, `Dispatch` — are newtype-args variants (`Command::Adr { command:
  AdrCommand }`). Their `-p` lives one level below, on the inner args struct.
- So a `write_class`-level extraction cannot see the `-p` that
  `e2e_adr_cli_golden` passes — one of the two tests ISS-028 was filed about.

`-p` is declared **204 times across 27 files** — 202 as `#[arg(short = 'p',
long)]` (20 in `commands/cli.rs`, 182 nested) plus 2 declared long-only
(`commands/serve.rs:17`, `commands/map.rs:13`), which a short-flag grep misses
but which collide with a global `--path` all the same. All `Option<PathBuf>`,
none with `default_value`, all meaning "project root". `Cli` itself carries no
`-p`; its only `global = true` arg is `--color`.

## Scope & Objectives

Make the worker-confinement guard evaluate the marker against **the tree the
command is actually operating on**, for every Write/Orchestrator/Hookmint-classed
verb — without weakening confinement.

Invariants any solution must preserve:

- **ADR-006 D2a** worker-mode formula: `(is_linked_worktree && marker_present)
  OR env DOCTRINE_WORKER`. Unchanged — this slice corrects *which tree* is
  tested, not the test.
- **SPEC-012 REQ-192**: `write_class` stays exhaustive (a new verb is a compile
  error).
- **Guard laziness**: only a Write-classed verb resolves the root, so a Read verb
  in a non-doctrine CWD gains no new failure path.
- The env leg (`DOCTRINE_WORKER`) stays root-independent.

Then sweep the residual test-side class (ISS-267): tests that legitimately run
against a marked tree and neutralise only the env leg.

## Non-Goals

- **Changing worker-confinement semantics.** Marker meaning, the D2a formula, and
  the D2b raw-tree gap are untouched.
- **ISS-219** (refusal transcript bloat) and **DEC-003** (`just validate` fork
  skip). They compound the pain and are named in Context for motive only.
- **Sibling instances of the same defect class** — IMP-233 (`arm-spawn` targets
  the CWD-detected root) and ISS-011 Defect C (`SubagentStart` hook cwd). Same
  class, different call sites; candidates for a follow-up sweep, not this slice.
- Broad CLI-surface redesign beyond what the fix requires.

## Design fork — RESOLVED ([[DEC-093]], `design.md` §7)

`-p/--path` becomes one `global = true` argument on `Cli`, mirroring `--color`;
the 204 per-subcommand declarations are deleted; `worker_guard` receives the
explicit path and evaluates the marker against the tree actually being written.

Rejected alternatives, with reasons, in `design.md` §7:

- **Extend `write_class` to yield class + path** — rejected on *capability*: 35
  of 54 `Command` variants carry no top-level path, so it cannot fix
  `e2e_adr_cli_golden`, one of the two originating tests.
- **Push the check down into the write functions** — rejected because there is no
  single write chokepoint, so the check scatters and REQ-192's compile-time
  exhaustiveness degrades to discipline.

The scale objection to the chosen option does not hold: removing a field from 204
declarations makes every missed site a **compile error**, unlike the `--color`
precedent, where *adding* a global flag failed silently.

## Affected surface

- `src/main.rs` — `Cli` gains the global `path`; guard + dispatch call sites
- `src/commands/guard.rs` — `worker_guard` signature (`write_class` untouched, D2)
- `src/commands/cli.rs` — `dispatch` signature; declarations deleted
- 26 further modules — declarations deleted; sub-dispatcher signatures
- `src/commands/serve.rs`, `src/commands/map.rs` — long-only `--path` deleted
- `tests/e2e_worker_guard.rs` — the new VTs, riding its linked-worktree fixtures
- `tests/e2e_*.rs` — help goldens (volume per OQ-1)

Explicitly **unchanged**: `src/root.rs` (D1 — its signature stays, ~150 call
sites), `src/worktree/marker.rs` (`resolve_mode` merely receives a correct root),
`src/test_support.rs` (Q1 closed without code change).

The authoritative touch-set is recorded as `design-target` selectors
(`doctrine slice selector list 236`).

## Risks, assumptions, open questions

- **R1 — atomicity.** Under option 1 the `-p` removal cannot be staged; a partial
  application does not compile. Phase boundaries must respect that.
- **R2 — golden churn.** `--help` output is byte-exact pinned; option 1 relocates
  `-p` in every subcommand's help. Volume unmeasured at scoping time.
- **R3 — fencepost.** The recorded `--color` precedent missed a handler at far
  smaller scale.
- **R4 — state.** The rejected push-down option implied carrying the write-class
  decision into the leaf layer; shared mutable state would breach the
  pure/imperative split. Retained as the reason that option stays rejected.
- **R5 — clap duplicate-arg behaviour is asserted, not run.** R1's atomicity
  rests on a global `-p` colliding with any surviving local. If clap silently
  shadows instead, the sweep could be staged and the phase plan changes. Confirm
  empirically before sweeping (`design.md` §10 F-1).
- **R6 — nested global propagation unconfirmed.** clap globals are expected to
  reach two-level-nested subcommands (`Command::Adr` → `AdrCommand::New`).
  Expected but unrun; failure invalidates the chosen approach outright
  (`design.md` §10 F-7).
- **R7 — fixture linkage.** ✓ `resolve_mode` requires `is_linked &&
  marker_present`, so a marker file in a bare tempdir is never refused. VTs built
  on hand-rolled tempdirs would pass trivially before *and* after, proving
  nothing (`design.md` §10 F-2/F-3).
- **A1** — marker semantics are correct; only tree selection is wrong.
  (ADR-006 D2a's SL-064 amendment retired location-pinning, supporting this.)
- **A2** — `19` of ISS-267's 29 `env_remove("DOCTRINE_WORKER")` files pass `-p`
  and are dissolved by the guard fix; `10` (chiefly `e2e_worktree_*`) genuinely
  operate on a marked tree and need the both-legs helper. Measured, not assumed —
  but re-verify after the guard fix lands rather than trusting the count.
- **Q1 — CLOSED, no code change.** The sequencing dissolves it: the 19
  tempdir-rooted files stop needing `under_worker_marker()` once the guard is
  fixed, and the 10 residual files run against the real tree, for which its
  `repo_root()` basis is exactly right.
- **Q2 — CLOSED.** ✓ `Boot` declares `path` documented as *"Used by the bare
  regenerate; `boot install` carries its own `-p`"*, so `doctrine boot -p X
  install` and `doctrine boot install -p X` are two different flags today. Both
  are deleted; the global serves each, collapsing the split.
- **OQ-1 — golden churn measured AFTER the sweep**, not before. A pre-sweep spike
  would need the global and the locals to coexist — the one state R1 forbids
  (`design.md` §10 F-1).

## Verification / closure intent

- The two originating tests (`e2e_adr_cli_golden`, `e2e_relation_migration_storage`)
  pass **inside a stamped worker fork**, not only on the coordination tree.
- A worker fork's `cargo test` is a trustworthy green signal for the exposed suites.
- Confinement is not weakened: a genuine worker write to a genuinely marked tree
  is still refused — including when `-p` names the marked tree itself.
- `write_class` exhaustiveness and guard laziness demonstrably intact.
- ISS-028 and ISS-267 closable on the evidence; ISS-240 already closed.

## Summary

## Follow-Ups
