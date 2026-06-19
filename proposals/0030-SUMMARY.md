---
seq: 0030
scope: capture
target: SUMMARY — index of proposals 0001–0029
confidence: high
reversible: yes (the whole branch is `git branch -D loop/proposals-2026-06-20` to discard)
---
## What
Overnight architect loop output: **29 proposals** on branch
`loop/proposals-2026-06-20`, one commit each, in `proposals/`. Read-only analysis;
**no authored state, code, or backlog transitions were touched** (hard fence held).
Worked from an isolated worktree (`.worktrees/loop-proposals`) after the shared main
tree got branch-switched mid-run by other agents. Every proposal defers its decision
to you.

**Start with [0021](0021-review-queue-triage-guide.md)** — the triage guide (Tiers
A–D + dependencies). This index is the flat list; 0021 is the reading order.

## Index by scope
**Backlog triage** — 0001 (dedup ISS-025/027), 0004 (widen IMP-067 → corpus id_path),
0008 (IMP-069/070/097 altitude cluster), 0015 (IMP-056 formatter scope), 0018
(SL-121↔IMP-075 integrate rework — *time-sensitive*), 0025 (IMP-081 done-but-open).
**Spec** — 0002 (SPEC-003 stale container refs), 0005 (SPEC-018 no requirement spine),
0012 (SPEC-001/002 detached from C4 tree), 0013 (SPEC-005 stale supersession storage),
0016 (PRD-005 leasing over-promises), 0019 (requirement AC ~52% empty), 0027
(verification basis fragmented — refines 0019).
**Codebase** — 0003 (transitive impact query), 0006 (concept-map↔graph bridge), 0010
(ADR-001 layering burn-down), 0017 (worktree gc --all), 0020 (4 skills off boot
routing), 0022 (MCP surface review-only), 0023 (STD kind undogfooded), 0028 (inspect
bare ids no titles), 0029 (prose-citation integrity reactive-only).
**Capture / synthesis** — 0007 (graph interchange export), 0009 (authored-priority
slot unbuilt), 0011 (unified `doctrine doctor`), 0014 (thesis: value gated on
consumption surfaces), 0021 (triage guide), 0024 (empty boot sections), 0026
(done-but-open detector).

## Highest-signal
- **Concrete defects, high confidence (cheap):** 0012 (missing `parent` edges —
  priority/graph subtree detached from C4 root), 0013 (SPEC-005 describes
  `supersedes` as typed; it migrated to `[[relation]]` SL-095 + verb landed SL-062),
  0001 (duplicate issue), 0025 (IMP-081 shipped but still open), 0002 (3-of-11 spec
  refs).
- **Time-sensitive:** 0018 — SL-121 is in `design` *now* and IMP-075 refactors the
  same `integrate` body; fold or sequence before its plan locks.
- **Strategic (the indispensable-to-teams bets, per 0014):** 0003 impact query,
  0011 doctor gate, 0022 MCP read tools, 0006 concept-map bridge, 0007 export. All
  reuse shipped primitives (cordage `reachable`, `/api/graph`, existing checks) —
  the model is mature; the *consumption surfaces* are the gap.

## Standing observation
The corpus is **markedly mature** — verified clean across: ADR-001 layering (fitness
test), spec FK/orphans (`spec validate`: corpus clean), all 47 spec lineage edges,
all 15 PRD→tech-spec descents, out-of-scope+success-measures on every PRD, backlog
dep/seq graph, production panics (test-only), withdrawn-entity refs, web client (thin
renderer, no server-policy dup). Recent loop iterations increasingly *disproved*
their own hypotheses — the signal that the reachable finding-surface was thoroughly
swept. Net: the durable themes are (1) **consumption surfaces** lag a strong model
(0014), and (2) minor **hygiene drift** (done-but-open, stale prose, uneven AC) that
a `doctrine doctor` (0011) + done-but-open detector (0026) would keep swept
automatically.

## How to act
- Accept: cherry-pick / re-author the proposal's "Next doctrine move" verbs (none run).
- Reject: `rm proposals/NNNN-*.md`.
- Discard everything: `git branch -D loop/proposals-2026-06-20` (zero residue;
  nothing else was touched). The `.worktrees/loop-proposals` worktree can be removed
  with `git worktree remove`.

## Loop end
Ending the loop: 29 quality proposals span all four scopes; the finding-surface is
swept and recent ticks yielded only refinements. Not rescheduling. Re-run `/loop …`
to resume (it appends from 0031). No STOP file was needed.
