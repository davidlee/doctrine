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

## Related

- The same staleness bites `slice verify-vt` indirectly but harmlessly: it reads
  the runtime registry at `<primary>/.doctrine/state/slice/<nnn>/boundaries.toml`
  (`src/state.rs:716-722`), which `sync --prepare-review` derives from the coord
  branch's committed ledger — so pre-`prepare-review` every VT row is
  `UNATTRIBUTABLE` by construction. Expected, not a defect, but easy to misread as
  a coverage gap at handover.
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
