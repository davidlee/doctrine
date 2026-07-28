# ISS-028: worker-marker confinement refuses CLI writes in stamped fork, breaking tests that shell the doctrine CLI

Discovered during SL-111 PHASE-02 (commit `83a12e04`).

**Status of the diagnosis, 2026-07-29.** This card has carried three successive
root causes. The third — root-resolution skew, fixable by threading `-p` into the
guard — was implemented and **retracted on evidence** (RV-319 F-2). The history
is preserved at the bottom because the retraction is the useful part; read
*Current diagnosis* and *Fix direction* first and do not act on the archived
sections.

## Symptoms

`e2e_adr_cli_golden` and `e2e_relation_migration_storage` failed *inside a
stamped worker fork* with `worker fork (signal: marker): refusing authored
write`. Both scaffold entities via the `doctrine` CLI, which the worker-mode
marker blocks. Observed again at SL-228 PHASE-03: ~8 targets red in the worker
shell (`e2e_link_unlink`, `e2e_adr_cli_golden`, `e2e_dep_seq_verbs`, …) while the
same targets passed inside the `worker_commit` gate's own run.

## Current diagnosis

The original framing was right, and the two "sharper" ones that followed it were
not: **the marker cannot distinguish a worker agent writing authored content from
a test fixture scaffolding entities.** That is a question about *actor intent*,
and no path argument can express it.

The marker identifies the **actor**, not the target — the refusal says so
outright (*"workers return a source delta; doctrine-mediated writes funnel
through the orchestrator"*). Decisively: **every tree a worker must not write to
is markerless** — the coordination tree and the primary repo — and the only
marked tree in a dispatch topology is the worker's own fork. So any fix that
re-keys the guard to the tree named by `-p` *inverts* the protection: it guards
only trees that carry a marker, which are exactly the ones that do not need
guarding, while every protected tree becomes reachable by naming it.

Reproduced (RV-319 F-2): from a marked fork, `doctrine adr new smuggled -p <a
markerless coordination tree>` exits 0 and creates the ADR; on unpatched HEAD the
same argv is refused. Three existing tests already encode the actor contract and
go red under the change —
`tests/e2e_dispatch_sync.rs::{prepare_review,integrate,record_boundary}_refused_under_worker_mode`.

## How much of this is already solved

More than the card implied. Authored-write goldens early-return on
`under_worker_marker()`, **and the server-side commit gate clears the marker
before its run (SL-199 F2), so those goldens still execute there — coverage is
preserved.** What remains is narrower than "tests are broken in a fork":

- a worker's **manual** `cargo test` skips the authored-write goldens;
- ~29 files carry `env_remove("DOCTRINE_WORKER")` scaffolding to cope
  (**ISS-267** owns that residual — re-measure it rather than assuming the split
  holds).

So this is an ergonomics-and-tidiness problem with an existing mitigation, not a
correctness hole.

## Fix direction: topological

**Do for the worker's manual test run what the commit gate already does** — clear
the marker around the run, restore it after.

Why this and not a guard change: it invents no new signal, alters no confinement
semantics, and rides a mechanism the design already sanctions for exactly this
purpose. `MarkerClear` is deliberately left unguarded (*"locking the marker's
only remover behind the marker is a self-brick we reject"*), and
`worktree marker --clear` already carries its own accident-fence — it refuses in
a linked worktree without `--operator`, whose message is *"pass `--operator` to
confirm you are the trusted orchestrator."*

That fence is the point, and it settles the framing question this card kept
tripping over: **worker confinement is cooperative, not adversarial.** A worker
can already stand itself down — by saying so, explicitly and auditably. What the
retracted fix did was make that bypass *silent and undeclared*: `-p <coord tree>`
looks like ordinary use and leaves no trace. An accident-fence that no longer
catches accidents is worthless, which is why the path fix failed even though the
capability it granted was not strictly new.

Alternatives considered and ranked below it:

- **Explicit declared carve-out** (a test-harness opt-in the guard honours) —
  coherent with the cooperative framing, but adds a second env leg, and ADR-006
  already treats env as the unreliable optimisation with the marker as primary.
- **Target discrimination by something other than the marker** (refuse writes to
  the coord tree, the fork, and their ancestors; permit scratch roots) — what the
  path fix was reaching for, without the inversion. A heuristic in a place that
  should not have one. Avoid.
- **Close as already-mitigated**, leaving ISS-267's sweep as the tidy-up.
  Legitimate, given the gate preserves coverage.

## Why the workaround is more expensive than it looks

Combined with `just validate` short-circuiting to a no-op inside a worker fork
(`justfile:36-40`, SL-225 #1 / DEC-003), a dispatch worker has **no reliable
local green signal**: its own `check gate` run is polluted by this skew, and the
recipe the commit gate runs skips the governance legs. The worker burns a
`worker_commit` round trip to discover an ordinary gate failure — and each such
refusal currently returns the whole transcript (**ISS-219**). The three compound.

## Guard rails for whoever picks this up

Pinned green, and they hold under any resolution — do not "fix" them away:

- `tests/e2e_worker_guard_explicit_root.rs` — a marked CWD refuses; the env leg
  stays root-independent with its own distinct message; **pathless guarded verbs
  cannot be handed a project root** (RV-319 F-1); Read verbs never resolve one.
  The two disputed skew cases are deliberately absent rather than asserted either
  way.
- `tests/common/mod.rs` — `marked_linked_fork` builds a **genuine linked
  worktree** and self-validates both marker legs. A marker file in a bare tempdir
  is never refused (`resolve_mode` requires `is_linked && marker_present`), so a
  tempdir fixture passes whatever the guard does and proves nothing.

Related: **ISS-267** (test-side residual), **IMP-348** (the CLI-surface cleanup
that SL-236 had bundled into this — independent; keep it that way).

---

## Archived: superseded diagnoses

Retained because the retraction is instructive. **Do not act on either.**

### Superseded — "sharper diagnosis" (SL-228 PHASE-03, 2026-07-25, was ISS-240)

Held that the collateral damage was not inherent but a root-resolution skew:
each affected e2e drives the CLI with `-p <temp root>` while
`src/commands/guard.rs` resolves from CWD, so the guard finds the worktree's
marker and refuses a write never aimed at a marked tree — and that threading the
explicit root in would fix it "without weakening confinement at all".

*Retracted 2026-07-29 (RV-319 F-2).* The skew description is accurate; the
inference is not. Confinement is weakened, severely, because the protected trees
are the markerless ones.

### Superseded — correction (2026-07-28, SL-236 pre-design research)

Held that the above was unreachable where the guard runs, since `worker_guard`
sees the outer `Command` and most variants are newtypes whose `-p` sits a level
down — and that the real fix was a design fork (global `-p` on `Cli`, or pushing
the check into `root::find`), governed by SL-236.

*Retracted 2026-07-29.* The reachability claim was itself falsified (RV-319 F-1
measured a leaf-level `ArgMatches` lookup reaching every shape), and SL-236 was
retired when F-2 invalidated the premise both its candidates shared. Its measured
CLI-surface analysis survives as **IMP-348**.
