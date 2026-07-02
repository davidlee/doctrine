# fable-log.md — Fable loop activity log

> One entry per active turn: did / observed / next / haiku. Newest appended at bottom.

---

02:55 — waiting (window opens 03:45)
## 03:06 — waiting (window opens 03:45)
   note: .git gitdir -> /home/david/dev/doctrine/.git/worktrees/doctrine-fable-loop; `git` fails in-jail ("not a git repository"). Resolve at window open before any commit.
   RESOLVED (host, pre-window): gitdir wiring relativized so it resolves under any prefix (host /home/... AND jail /workspace/...). worktree .git -> `../../.git/worktrees/doctrine-fable-loop`; admin back-ref -> `../../../.worktrees/fable-loop/.git`. `git` now works in-jail without edits. Caveat: if a real commit fails with a read-only/permission error (NOT "not a git repository"), the jail ro-binds shared .git — if you can't fix it from /workspace/doctrine, you'll have to do without.

## 03:55 — turn 1 (first active): oriented, unblocked build, shipped ADR-001 gate-edge-model note
observed: fable-loop worktree was hand-created (raw `git worktree add`), so it
  bypassed `.worktreeinclude` and lacked `web/map/dist/` → cargo build failed
  (RustEmbed). Copied dist from main worktree (gitignored). Tree green.
  Chose thread = RFC-011 friction burn-down. Increment 1 shipped: ADR-001 body
  now documents the layering gate's edge model (top-level→first-segment edges,
  BTreeSet-deduped; sub-class refines direction check only, NOT tangle ratchet)
  — both claims verified vs tests/architecture_layering.rs. Commit ceff6e90.
  Increment 2 (memory retrieve phrasing) = OBE; boot-footer already fixed on edge.
  vtgate comment-match = by-design (POL-002). Two case-notes already OBE →
  burn-down has high verify-then-skip cost, so launched a background triage
  sub-agent (a608c2f9a5f7f8337) to classify ALL remaining case-notes and hand
  back a clean still-open doc-gap worklist.
next: process triage worklist; ship the top still-open doc-gap fix.

  Edges collapse to
  top-level; the map
  hides the deep paths.
