## Decision

The conformance fixture binds **no directories** as readable inputs. Its readable set is the closure of the declared payload toolset and nothing else, so the capsule's inner `PATH` is empty and every payload invokes its tools by absolute path. The capsule discovers its own reach by parsing `/proc/self/mountinfo`, not by reading a `PATH` the fixture arranged for it.

This is the **no-directories** option of `inq-2`'s three, taken with the *discovered* rewrite of row 2
rather than the declared one.

**Letter warning.** `DEC-185` uses "shape (C)" for a *different* thing — `inq-1`'s third candidate,
*the toolset's directories plus the toolset's closures*. This decision rejects that. The letters were
reused across two nodes and mean opposite things; cite the descriptions, never the letters.

## Why

Two constraints in the production backend, both verified at `crates/doctrine-control/src/backend/bubblewrap.rs` on 2026-08-11:

1. **Only a bound directory reaches the inner `PATH`.** `derived_inner_path` (`:1014`) keeps a host `$PATH` entry when it lies within a bound path, and `is_within` (`:1000`) is one-directional by design — its doc comment refuses to admit `/usr/bin` on the strength of a bound file `/usr/bin/git`. Binding tool files, however complete their closures, yields an empty inner `PATH`.
2. **The readable set must be pairwise non-overlapping and nothing dedupes it.** `readable_paths` (`:941`) concatenates declared roots then closure members with no containment filter; `CapsulePlacement::try_new` (`backend.rs:410`) refuses any pair where either lies on the other's root-ward chain. Combined with the resolver contract's requirement to echo the queried path, any shape that binds a `PATH` entry directory *and* expands the closure of a tool inside it collides with itself on every host.

So the two directory-binding shapes were (A) closure roots are the directories, the resolver echoes the directory; and (B) entry dirs as readable roots, tool files as closure roots, resolver suppressing its own echo. Both bind a whole `bin` directory. On a store-managed host that exposes names rather than bytes — the farm's symlinks dangle unless their targets are bound — but on a conventional distribution, where one directory supplies every tool, `/usr/bin` becomes readable whole. (C) has no such host dependence: the readable set is the closure, identically on every glibc Linux.

The fixture cannot evade the empty `PATH` by other means: `capsule_environment` (`:1063`) always overwrites `PATH` with the derived inner path, so `Execution::env` cannot inject one.

## What it costs

Row 2 must be rewritten. `execs_only_what_is_bound` (`conformance.rs:3360`) derives its root set by cutting the top-level component from each `$PATH` entry; with an empty `PATH` the loop never runs, `bound=0`, the guard short-circuits, neither token prints, and the row reads `NoObservation`. That rewrite is an explicit line item in this slice's cost and is the figure `DEC-185`'s relent-to-S3 permission should be measured against.

The rewrite has two candidate shapes and this decision takes the second: a fixture-declared bound set the payload confirms (near-tautological — we assert what we bound and the capsule agrees), or the payload parsing `/proc/self/mountinfo` for its own read-only mounts and executing from each. The latter is discovered rather than declared and is stronger than today's `$PATH` proxy under *any* readable-set shape. Its design is carried by a separate blocking inquiry.

Row 4's permitted-`/` list must also stop being coupled to `top_level_ancestor(SHELL)`, but that is common to all three shapes and is not differential cost.

## What it gives up

`derived_inner_path` is production code with no live conformance exercise today. Shapes (A) and (B) would have given it one; (C) never will. This is the mirror of the argument `DEC-186` used to justify routing the fixture through `closure-roots` deliberately, and it is accepted knowingly.

## Reversibility

Cheap, and roughly at parity with doing it the other way now. Three fixture-side sites in one file; no production change; no spec change, because the fixture's own construction is ungoverned at this granularity. Payload text is forward-compatible in the expensive direction — absolute paths keep working if a `PATH` ever reappears — so the nineteen rows are not re-edited on reversal.

The cost that is *not* cheap to undo is epistemic rather than structural: if row 2 ships in its declared form, a green row nobody re-examines is exactly how this defect family arose. That is why the discovered form is part of this decision rather than left to implementation.

## Explicitly not decided here

Whether bubblewrap confinement is adequate on non-NixOS Linux, or whether such hosts want a VM- or microVM-backed capsule, is a backend-authority question in `ADR-020`'s territory. Nothing in any of the three shapes touches it, and deferring it costs this slice nothing.


---

## Withdrawn in part, 2026-08-11

**The readable-set half of this decision is withdrawn.** Measurement one turn
after it was banked showed that binding files alone does not run: `bwrap`
dereferences a bind's source and `provision.rs` binds at the **resolved** path,
so a multicall binary arrives under its target's name and `cat`, `head`, `cut`,
`ls`, `tr`, `pwd`, `env`, `true` and `sleep` become uninvokable
(`mem.fact.capsule.resolved-path-bind-breaks-multicall-dispatch`). Worse, an
empty inner `PATH` breaks *provisioning* — `clone_inside` runs all four git
operations as bare `git` — so the fixture would refuse before row 1.

A proposed rescue (bind the file when its resolved basename matches the declared
one, the containing directory when it does not) was withdrawn too: `/bin/sh →
/usr/bin/dash` is a renamed symlink and not a multicall alias, so the rule binds
`/usr/bin` for the shell alone.

The root cause is `ISS-344` — canonicalization destroys the declared name, and
`SL-248` `PHASE-05` `EX-3` and `EX-9` contradict each other about whether it
should. That is production work and `SL-252` does not take it on. `DEC-188`
records what `SL-252` does instead.

**The mountinfo half stands, demoted.** Discovering reach from
`/proc/self/mountinfo` was confirmed by live probe and remains the better shape
for row 2. But under `DEC-188` the inner `PATH` is populated on and off jail, so
row 2 needs no rewrite: the mountinfo work is an improvement to be carried
separately, not a dependency of this slice.
