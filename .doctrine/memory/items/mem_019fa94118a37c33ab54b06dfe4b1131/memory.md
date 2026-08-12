`worker_guard` resolves the project root by walking up from **CWD**, and that is
correct, not a defect. The marker answers *"is the process running me
confined?"* — not *"is the tree being written to protected?"*.

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
[[ISS-028]] now routes to a topological fix instead.

## Confinement is cooperative — an accident-fence, not a security boundary

`WriteClass::MarkerClear` is deliberately unguarded (*"locking the marker's only
remover behind the marker is a self-brick we reject"*), and `run_marker_clear`
refuses in a linked worktree without `--operator`, saying *"this is the
accident-fence; pass `--operator` to confirm you are the trusted orchestrator."*

A worker can therefore already stand itself down — **explicitly and auditably**.
The corollary that matters when judging any proposed change: the test is not
*"does this grant new capability?"* but *"does this make a sanctioned bypass
silent and undeclared?"* An accident-fence that no longer catches accidents is
worthless.

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

Worker-ness is a property of a **process**. The disk marker records it as a
property of a **tree**. Everything awkward about the marker follows from that one
mismatch — naming it *predicts* the awkwardness instead of cataloguing it:

- a tree outlives the process that marked it, so the **stale-marker class** has
  to exist: `Cause::Marker`, `is_stale_marker`, `worktree status --assert`'s
  stale exit, and the whole `marker --clear --operator` verb with its
  accident-fence and cwd checks (`marker.rs:203-257`) — plus open [[ISS-028]].
- the mark is written by a *different command* at a *different time*
  (`fork.rs:192`, under `--worker`) than the one that confines the worker.

`DOCTRINE_WORKER` has no stale class **by construction** — it dies with the
process — and on the confined subprocess arm it is set by the *same bwrap argv*
that establishes the write floor (`scripts/pi-spawn-confined.sh:127` `--setenv`,
beside the `--ro-bind / /` floor at `:113-131`). Identity and confinement become
one atomic act by one actor.

## Marker *absence* is not a coordination-tree identity

Worth pre-empting, because `ADR-006` §D2a's own prose invites the misreading:
*"the orchestrator's write permission rests on marker-absence"*.

That is the **contrapositive of the refuse predicate**, not a mechanism. Verified
by census: **no reader anywhere concludes "this is the coordination tree" from
absence.** Absence is only the non-firing of a positive worker test — the
actor-based framing above, restated. So removing a leg from the refuse predicate
cannot turn a not-refused tree into a coord claim, because nothing ever claimed
coord. This killed a contest against `DEC-207` that had blocked a design run.

Positive coordination identity already exists and never consulted the marker:
`classify_worktree_role` (`shared.rs:77`) requires linked **and** an all-numeric
`dispatch/<NNN>` branch. Callers: `dispatch whereami`, `review`'s root guard.

`ADR-006` §D2b still names [[IMP-065]] as "the real positive-marker close", but
`IMP-065` was **closed obsolete** (`REV-018`): a positive marker is a cooperative
flag, not a boundary, and the genuine close is enforcement, shipped as
confinement (`SL-182`/`SL-183`/`SL-185`). That argument retires the *negative*
marker identically — which is the general rule this file is really about:

> The test for any marker proposal is whether it is a **boundary** or a **flag**.
> A flag can only ever be an accident-fence. Reach for enforcement when you want
> a boundary, and do not let a flag's *absence* stand in for an identity.
