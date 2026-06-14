# Dispatch projection parents on the pinned fork-base, not the live trunk tip

`dispatch sync --prepare-review` (stage-1) projects `review/<slice>` and every
`phase/<slice>-NN` onto a trunk base. That base MUST be the pinned fork-point —
`git merge-base(refs/heads/dispatch/<slice>, trunk)` — NOT the live trunk tip
(`git::trunk_commit`).

**Why:** a coordination worktree isolates the *working tree*, not the trunk
*ref*. A foreign commit landing on trunk between `worktree coordinate` and
`dispatch sync` silently reparents the projection onto the moved tip: per-phase
diffs stop being exact (foreign trunk deltas leak in), and the design's
"`integrate --trunk` refuses non-ff" safety net (IMP-043) is bypassed — the
projected tip already descends from the moved trunk, so the ff-check passes
spuriously. A pre-stage-1 move is absorbed; only a post-stage-1 move was ever
caught. This was RV-030 F-1 (the SL-064 close-gating blocker).

**How to apply:** project off the merge-base; keep `trunk_commit()` (live tip)
ONLY at integrate's actual trunk push under CAS, where ff really matters. The
`git::merge_base` seam returns `Ok(None)` for unrelated histories (exit 1),
distinct from a usage error — mirror `is_ancestor`'s explicit exit-code match,
don't route through `git_opt` (which conflates exit 1 with exit 128). Regression
guard: a trunk-moved-during-run e2e that asserts the phase cut is parented on the
fork-point and `integrate --trunk` then refuses the non-ff (RED-proven against
live-tip projection).

Related: [[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]],
[[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]],
[[mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree]].
