# ISS-275: Review verbs refuse in the coordination worktree

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`src/review.rs::resolve_review_root` guarded on `is_linked_worktree` — a blanket
linkage test with no role distinction — so every baton-driving review verb bailed
in a dispatch **coordination** worktree:

```
$ doctrine review prime RV-320      # in .dispatch/SL-233, branch dispatch/233
Error: review verbs are not supported on a worktree fork (IMP-024): the turn
baton lives in the parent tree's gitignored state, which a fork cannot
co-write. Run `review` from the parent tree.
```

The guard's own rationale does not describe that tree. It is written for a
*worker fork*, whose `WITHHELD` tier hides the parent's gitignored state. A
coordination tree is the sole writer of `dispatch/<NNN>` and carries its own
state tier, and `dispatch whereami` already reports it as `role: "coord"`, not
`fork`.

## Why it mattered

SL-233's plan makes an RV ledger the second half of all three design-gate
entrance criteria (PHASE-02 projection bounds, PHASE-06 marker grammar,
PHASE-08 thin adapter). Every one of those sketches lives in the coordination
tree, because that is where ADR-012's topology puts a slice's authored work
during dispatch. So the guard stranded the reviews in the one tree they have to
run in, with no in-tree workaround — only `review list` (which touches no baton)
worked.

## Fix

The test is the **role**, not mere linkage:

- `classify_worktree_role` + `COORD_BRANCH_SHORT_PREFIX` relocated from
  `src/dispatch.rs` to `src/worktree/shared.rs`, beside `is_linked_worktree`
  where they belong — two command-tier callers now need them and STD-001 wants
  one home. `review` already declares a `worktree` edge in
  `.doctrine/adr/001/layering.toml`, so no tier moved. Its unit tests moved with
  it.
- `resolve_review_root` classifies and bails only on `"fork"`. `primary` and
  `coord` pass. The numeric-suffix test in the classifier is what keeps a worker
  fork's `dispatch/<agent>` branch on the refusing side.

**No state relocation was needed**, which was the open question going in.
`state_dir(root, id)` is root-derived (`root/.doctrine/state/review/<NNN>`), so
admitting the tree is the entire fix: the baton lands in the coordination tree's
own gitignored state, with nothing to contend over. The doc comment calling that
a "parent-tree locus" described the guard, not the mechanism, and was corrected
to "invoking-tree". Loss degrades gracefully by design — an absent baton reads
as cold and recomputes (D-C4a).

## Tests

- `vt10b_coord_worktree_admitted_and_baton_in_its_own_state` (new) — builds a
  real linked worktree on `dispatch/001`, asserts the verb is admitted and the
  baton lands in the coord tree, not the parent. Observed red before the change,
  failing on the guard.
- `vt10_fork_root_refused_and_baton_in_parent_state` (existing) — unchanged and
  still green; its `git worktree add` yields a non-`dispatch/<NNN>` branch, so it
  still classifies as `fork`.
- `classify_worktree_role_maps_branch_and_isolation` — moved, unchanged.

Scope note: this is not SL-233 content and deliberately did not land on
`dispatch/233` — `src/worktree/**` carries no selector on that slice, so it would
have failed conformance and commingled an unrelated engine fix into the slice's
diff. Landed on `edge` and flowed into the coordination tree through
`refresh-base`.

Related: [[IMP-024]] (the larger parallel-raiser funnel this guard was holding a
place for — untouched; forks still refuse).
