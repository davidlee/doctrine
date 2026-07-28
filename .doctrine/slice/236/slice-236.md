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

`-p` is declared **202 times across 27 files** (20 in `commands/cli.rs`, 182
nested), all `Option<PathBuf>`, none with `default_value`. `Cli` itself carries
no `-p`; its only `global = true` arg is `--color`.

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

## Design fork (unresolved — for `/design`)

Not settled at scoping time. Named here so `/design` load-bears on it:

1. **Global `-p` on `Cli`** (`global = true`, mirroring `--color`). Uniform and
   complete; removes 202 declarations in one necessarily atomic commit (a global
   short flag collides with any surviving local `-p`). Changes every subcommand's
   `--help`, so byte-exact goldens churn. The `--color` migration (SL-079) is the
   precedent — and recorded a fencepost miss at ~1/10th this scale.
2. **Push the check down to `root::find`**, where the correct explicit path is
   already threaded at ~150 call sites. Fixes every verb with no path-threading,
   but must carry the write-class decision to that point without breaching
   REQ-192 exhaustiveness or laziness.

Option 3 — extending `write_class` to yield class + path — is **rejected**: it
fixes 9 of 33 Write-classed variants and cannot fix the originating test.

## Affected surface

- `src/commands/guard.rs` — `worker_guard`, `write_class`
- `src/main.rs` — `Cli`, the guard call site
- `src/root.rs` — `find`
- `src/commands/cli.rs` — `dispatch`, per-variant `-p`
- `src/worktree/` — `resolve_mode`, marker legs
- `src/test_support.rs` — `under_worker_marker`
- `tests/e2e_*.rs` — exposed suites

## Risks, assumptions, open questions

- **R1 — atomicity.** Under option 1 the `-p` removal cannot be staged; a partial
  application does not compile. Phase boundaries must respect that.
- **R2 — golden churn.** `--help` output is byte-exact pinned; option 1 relocates
  `-p` in every subcommand's help. Volume unmeasured at scoping time.
- **R3 — fencepost.** The recorded `--color` precedent missed a handler at far
  smaller scale.
- **R4 — state.** Option 2 implies carrying the write-class decision down to
  `root::find`; shared mutable state would breach the pure/imperative split.
- **A1** — marker semantics are correct; only tree selection is wrong.
  (ADR-006 D2a's SL-064 amendment retired location-pinning, supporting this.)
- **A2** — `19` of ISS-267's 29 `env_remove("DOCTRINE_WORKER")` files pass `-p`
  and are dissolved by the guard fix; `10` (chiefly `e2e_worktree_*`) genuinely
  operate on a marked tree and need the both-legs helper. Measured, not assumed —
  but re-verify after the guard fix lands rather than trusting the count.
- **Q1** — is `under_worker_marker()` correct for temp-root tests? It checks
  `repo_root()`, the test binary's root, not the root under test.
- **Q2** — does the `Boot` variant (inline `path` *and* a subcommand) need
  special handling under option 1?

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
