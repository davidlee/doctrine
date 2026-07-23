# ISS-234: Funnel ref-advance leaves coord worktree reverse-diff

## Problem

The dispatch funnel is deliberately **working-tree-free** (`commit_on_behalf` =
`commit-tree` + `update_ref_cas`, object-db only; ADR-012 / dispatch design §B).
`dispatch_import` / `dispatch_conclude_phase` advance the `dispatch/<NNN>` branch
ref but never touch the coord tree's index/worktree, so after a land `git status`
in the coord tree shows every landed file as a **staged deletion** (reverse-diff).
A pathless `git commit` here would commit mass reversions; the orchestrator must
`git restore --source=HEAD --staged --worktree -- <paths>` after every funnel
write. RFC-011 case-notes: 6 repros (SL-206, SL-210, SL-213, SL-219, SL-220,
SL-221), HIGH impact.

## Why not fixed in SL-225 (Cluster 1)

Considered as SL-225 fix #3 and pulled: it is **not a false-red** (SL-225's
theme) but a tree-state footgun on a different seam, and the durable fix is
RFC-016 **move A/B** — read-verbs covering every funnel read, then prohibit
shell-git-in-funnel. Once the orchestrator never shells raw `git status` /
`git commit` against the coord tree, the reverse-diff is irrelevant. An interim
auto-sync (`reset --hard` post-CAS) would be built against the fault-safety
invariant then ripped out when the read-verbs land — churn. So this routes to
**RFC-016 Cluster 2** (the `dispatch next` / read-verb slice), fixed once.

## Fix direction (Cluster 2)

Either: (a) funnel results / a `dispatch delta` read-verb report the forward-diff
state so no shell `git status` is needed, plus a no-pathless-commit guard; or
(b) if a real checkout sync is still wanted, a *conditional* forward-sync
(clean-tree → fast-forward, dirty → advisory) that preserves the working-tree-free
fault-safety invariant. Decide under Cluster 2's design.

## Links

- RFC-016 — Cluster 2 / move A/B (durable home).
- SL-225 — Cluster 1 sibling that explicitly excludes this (non-goal).
- Case-note #2, `.doctrine/rfc/011/case-notes-analysis.md`.
