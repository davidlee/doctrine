`worker_guard` **used to** resolve the project root by walking up from **CWD**,
and at the time that was correct, not a defect — the marker answered *"is the
process running me confined?"*, not *"is the tree being written to protected?"*.
*(SL-254 later removed the root resolution entirely: the verdict is a property of
the process, so there is no tree left to resolve. The rest of this file is the
argument that got us there.)*

## Why target-based resolution inverts the guard

In a dispatch topology the **only marked tree is the worker's own fork**. Every
tree a worker must not write to — the coordination tree, the primary repo — is
**markerless by construction**. So keying the guard to the tree named by `-p`
guards only trees that carry a marker, which are exactly the ones that need no
guarding, while every protected tree becomes reachable by naming it.

Measured (RV-319 F-2): from a marked fork,
`doctrine adr new smuggled -p <markerless coord tree>` exits 0 and creates the
ADR. Unpatched, the same argv is refused. Three tests already encode the actor
contract and go red under such a change —
`tests/e2e_dispatch_sync.rs::{prepare_review,integrate,record_boundary}_refused_under_worker_mode`.

This killed SL-236 (both its candidate fixes shared the premise) and is why
[[ISS-028]] was routed to a topological fix rather than a `-p` retarget. SL-254
then deleted the very marker that issue is about; `ISS-028` was still `open` on
2026-08-05, so it is **likely obsolete and worth a triage check**.

## Confinement is cooperative — an accident-fence, not a security boundary

> **CORRECTED 2026-09-26 — SL-254 shipped, and the escape hatch retired with the
> marker.** `WriteClass::MarkerClear`, `run_marker_clear`, `worktree marker
> --clear`, `status --assert`'s stale exit and the `Cause` truth table are all
> gone — `src/worktree/marker.rs`'s header now reads *"the disk marker this module
> was named for is gone. Identity is now a property of the PROCESS, not of a
> tree"*, and `main.rs` asserts *"`worktree marker --clear` must no longer
> parse."* The retired text below claimed a worker could stand itself down
> *"explicitly and auditably"* via `--operator`; with the env leg there is nothing
> to stand down from — a worker cannot unset the confinement argv's variable in
> the orchestrator's process. The fence is now structural rather than
> cooperative, which is strictly stronger, though still a fence and not a
> boundary.

What survives, and is still the right lens on any proposed guard change: the test
is not *"does this grant new capability?"* but *"does this make a sanctioned
bypass silent and undeclared?"* An accident-fence that no longer catches
accidents is worthless.

## Corollary for CLI design

A per-verb `-p` declaration is the machine-checkable record that **that verb
consumes a project root**. Promoting `-p` to a `global = true` arg makes
acceptance universal while consumption stays per-verb, destroying that
information — four guarded verbs (`Command::Onboard`,
`WorktreeCommand::{CreateFork, Nominate, Denominate}`) are pathless unit
variants that would then accept a root nothing reads (RV-319 F-1). See
[[IMP-348]].

## The deeper reason: the marker models a process fact as a tree fact

Added `SL-254` design, 2026-08-13 (`DEC-207`), from a full reader/writer census
of the marker surface.

> **SL-254 resolved the mismatch by deleting the tree fact.** This section is the
> pre-ship analysis that named *why* it had to go; it is retained because the
> naming is what made the deletion obvious. Read the verbs below as retired.

Worker-ness is a property of a **process**. The disk marker recorded it as a
property of a **tree**. Everything awkward about the marker followed from that one
mismatch — naming it *predicted* the awkwardness instead of cataloguing it:

- a tree outlives the process that marked it, so the **stale-marker class** had to
  exist: the `Cause` truth table, `is_stale_marker`, `worktree status --assert`'s
  stale exit, and the whole `marker --clear --operator` verb with its
  accident-fence and cwd checks — plus [[ISS-028]]. **All retired at SL-254.**
- the mark was written by a *different command* at a *different time* than the one
  that confines the worker.

`DOCTRINE_WORKER` has no stale class **by construction** — it dies with the
process — and on the confined subprocess arm it is set by the *same bwrap argv*
that establishes the write floor (`scripts/spawn-confined.sh` — `--setenv
DOCTRINE_WORKER 1` sits beside the `--ro-bind / /` floor in one argv: `:183`
beside `:137`-`:200` as of 2026-09-26). Identity and confinement become one atomic
act by one actor.

## Marker *absence* is not a coordination-tree identity

Worth pre-empting, because `ADR-006` §D2a's historical prose invites the
misreading: *"the orchestrator's write permission rests on marker-absence"*. (The
ADR now carries an SL-254 amendment over it — *"Read every 'marker-absence' below
as 'env-absence'; the sole signal is `DOCTRINE_WORKER` in its own process
environment"*.)

That is the **contrapositive of the refuse predicate**, not a mechanism. Verified
by census: **no reader anywhere concludes "this is the coordination tree" from
absence.** Absence is only the non-firing of a positive worker test — the
actor-based framing above, restated. So removing a leg from the refuse predicate
cannot turn a not-refused tree into a coord claim, because nothing ever claimed
coord. This killed a contest against `DEC-207` that had blocked a design run.

Positive coordination identity already exists and never consulted the marker:
`classify_worktree_role` (`shared.rs:77`) requires linked **and** an all-numeric
`dispatch/<NNN>` branch. Callers: `dispatch whereami`, `review`'s root guard.

`ADR-006` §D2b **used to** name [[IMP-065]] as "the real positive-marker close";
the ADR has since been amended to record the resolution (SL-254, 2026-08-14) —
`IMP-065` was **closed obsolete** (`REV-018`), a positive marker is a cooperative
flag and not a boundary, and the genuine close is enforcement, shipped as
confinement (`SL-182`/`SL-183`/`SL-185`, made uniform by SL-254). That argument
retires the *negative* marker identically — and the marker has now been retracted
outright — which is the general rule this file is really about:

> The test for any marker proposal is whether it is a **boundary** or a **flag**.
> A flag can only ever be an accident-fence. Reach for enforcement when you want
> a boundary, and do not let a flag's *absence* stand in for an identity.
