# ISS-252: claude-arm funnel verify beat runs against a stale coord checkout, yielding a false green

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

SL-230 PHASE-01, claude arm. Funnel order run exactly as `/dispatch` documents it:

```
dispatch_import  → Imported{coord_tip: 290148469}
check regression diff --base <B>  → "✓ no new or changed failures"   # green
dispatch_conclude_phase → Concluded
```

The green was **vacuous**. At the moment the diff ran, the coordination working
tree did not contain the delta at all:

```
$ git status --porcelain          # in the coord tree, after import
D  .doctrine/dispatch/230/boundaries.toml
M  src/entity.rs
$ grep -c "pub(crate) fn write_body" src/entity.rs   # worktree
0
$ git show HEAD:src/entity.rs | grep -c "pub(crate) fn write_body"
1
```

The index/worktree held the **inverse** of the two landed commits — `git diff
--cached HEAD` was `−190 src/entity.rs`, `−5 boundaries.toml`. So the suite
compiled and ran the *pre-delta* tree and reported no regressions, having never
seen the code it was gating.

`git reset --hard HEAD` in the coord tree, then re-running the identical command,
produced a genuine green (and `cargo test --bin doctrine entity::tests::write_body`
→ 6 passed, confirming the delta was really present the second time).

## Cause

Structural, not incidental. On the claude arm the funnel's steps 4+7 are folded:
`dispatch_import` composes `coord-tip ⊕ worker-tip` via `merge-tree` and commits
**object-db only** — explicitly working-tree-free, as its own doc says ("the live
coord index/worktree are never touched"). `dispatch_conclude_phase` is likewise
"working-tree-free". Both therefore advance `refs/heads/dispatch/<slice>` while
leaving the coord checkout at the pre-import commit.

The router's funnel then says:

> 5. Verify — `doctrine check regression diff --base "$B"` (suite @ S, SAME
>    normalised filter state as the capture)

"suite @ S" is the intent, but **nothing puts the tree at S**, and
`check regression diff` builds from the working tree. So on the claude arm the
default path is a silent false green — the failure mode the beat exists to prevent.

This is worse than a missing step because it fails *open*: a real regression in a
worker delta would be reported as clean, and the phase would conclude on it. The
pi/subprocess arm is not exposed the same way — its `import --from-worktree`
applies into the live index, so the tree *is* at S when the beat runs.

## Suggested fix

Options, roughly in order of preference:

1. **Make the beat self-sufficient** — have `check regression diff` resolve and
   materialise `S` itself (or refuse when `HEAD` != the tree's state) rather than
   trusting whatever the working tree happens to be. Fails closed, arm-agnostic,
   and no skill text can drift out of sync with it.
2. **Have `dispatch_import` refresh the coord checkout** after a successful
   compose. Cheap, but gives up the working-tree-free property that makes the
   compose safe against a dirty coord tree.
3. **Document the ritual** in `/dispatch`'s funnel and `/dispatch-agent`'s landing
   mechanics: `git reset --hard HEAD` in the coord tree between import and verify.
   Weakest — it is exactly the kind of manual beat an agent skips, and its absence
   is invisible (green either way).

(1) plus (3) is the belt-and-braces combination.

## Second surface — `conclude_phase` leaves the tree DIRTY, not merely stale

*Added 2026-07-27 from the SL-230 PHASE-03→04 boundary.* The observed section above
is the pre-verify case. In a **serial multi-phase drive** the same root cause has a
second, sharper surface at the *other* end of the funnel.

`dispatch_conclude_phase` commits the boundary row working-tree-free too, so after
it returns the coord checkout holds the **inverse of its own commit** — staged:

```
$ git status --porcelain                       # coord tree, after conclude
M  .doctrine/dispatch/230/boundaries.toml
$ git diff --cached HEAD -- .doctrine/dispatch/230/boundaries.toml
-[[boundary]]
-phase = "PHASE-03"
-code_start_oid = "e7e0d42f…"
-code_end_oid  = "232dbd3f…"
-provenance = "funnel"
```

Two consequences the import case does not have:

1. **It fails the next phase's base-clean precondition.** The funnel's step 1 is
   "worktree/index clean, HEAD == B". After a conclude the tree is never clean, so
   every phase after the first begins on a precondition violation that is entirely
   an artefact of the previous phase's own bookkeeping. Unlike the verify beat this
   at least fails *closed* — but it reads as a defect and invites the operator to
   "fix" it the wrong way.
2. **A pathless commit at that moment silently reverts the boundary row.** The
   staged content is a deletion of the row `conclude_phase` just landed. The
   repo-wide "path-limit the commit itself" rule (AGENTS.md) is what stands between
   this and a lost ledger entry — which `prepare-review`'s completeness gate would
   later report as a missing row for a completed phase, at the worst possible time.

So the ritual under fix (3) is not "reset between import and verify". It is **reset
after every working-tree-free write** — import *and* conclude. That the correct
ritual is broader than the one place the router could name it is itself an argument
for fix (1) or (2): a documented beat whose scope the documentation gets wrong is
weaker than it looks.

Fix (2) reconsidered in this light: refreshing the checkout after a successful
compose gives up the working-tree-free property, but the property is only load-
bearing *during* the compose. A post-success `reset --hard` to the new tip is not
the same concession and would close both surfaces.

## Third surface — `sync --prepare-review`, at the conclude cadence

Confirmed SL-230, 2026-07-27, at 6 of 6. `dispatch sync --slice 230
--prepare-review` commits the boundaries ledger and the journal, and it is
working-tree-free by the same construction. Immediately after a successful run
(7 refs created, exit 0):

```
$ git status --porcelain
D  .doctrine/dispatch/230/journal.toml
$ git diff --cached HEAD --stat
 .doctrine/dispatch/230/journal.toml | 57 -------------------------
```

Same shape as the second surface: the tree holds the **staged inverse** of the
write's own commit. The exposure is identical — a pathless commit here reverts
the journal `prepare-review` just wrote, and the conclude cadence's very next
beats (worktree removal, `slice status audit`) run against a tree that looks
dirty for no reason the operator caused.

**So the count is now three, and the generalisation should be stated as the rule
rather than enumerated as instances:** *every* working-tree-free write on this
arm — `dispatch_import`, `dispatch_conclude_phase`, `sync --prepare-review` —
leaves the coord checkout at the pre-write commit and the index holding the
inverse. Treat `git reset --hard HEAD` (after confirming with `git diff --cached
HEAD` that nothing local is at risk) as the standing post-condition of the class,
not as a per-verb workaround. A fix that only patches the two known verbs will be
overtaken by the fourth.

## Related

- The same staleness bites `slice verify-vt` indirectly but harmlessly: it reads
  the runtime registry at `<primary>/.doctrine/state/slice/<nnn>/boundaries.toml`
  (`src/state.rs:716-722`), which `sync --prepare-review` derives from the coord
  branch's committed ledger — so pre-`prepare-review` every VT row is
  `UNATTRIBUTABLE` by construction. Expected, not a defect, but easy to misread as
  a coverage gap at handover. **Now confirmed empirically** (SL-230, 2026-07-27):
  the same 8 rows read `≈ UNATTRIBUTABLE` before `prepare-review` and `✓ PASS`
  after it, with no intervening code or plan change. So the pre-`prepare-review`
  reading carries no signal at all — it should arguably be rendered as "not yet
  attributable" rather than as a per-row verification outcome.
- Adjacent wording nit in the same area: `UNATTRIBUTABLE`'s message reads "keyword
  present but `<path>` not modified by this slice", yet `vtgate.rs` step (4)
  short-circuits **before** step (5) matches any keyword — so no keyword was ever
  checked. Already noted as an RFC-011 case note; worth folding into the same fix
  pass.

## Provenance

SL-230 PHASE-01/02 dispatch drive, 2026-07-27. Caught at the head of PHASE-02
while investigating unexpected index/HEAD divergence in the coord tree; PHASE-01's
verify beat was re-run honestly before the drive continued, and the phase's result
stands. Recorded in `.doctrine/slice/230/notes.md` § Funnel mechanics.
