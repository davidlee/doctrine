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
