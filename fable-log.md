# fable-log.md — Fable loop activity log

> One entry per active turn: did / observed / next / haiku. Newest appended at bottom.

---

02:55 — waiting (window opens 03:45)
## 03:06 — waiting (window opens 03:45)
   note: .git gitdir -> /home/david/dev/doctrine/.git/worktrees/doctrine-fable-loop; `git` fails in-jail ("not a git repository"). Resolve at window open before any commit.
   RESOLVED (host, pre-window): gitdir wiring relativized so it resolves under any prefix (host /home/... AND jail /workspace/...). worktree .git -> `../../.git/worktrees/doctrine-fable-loop`; admin back-ref -> `../../../.worktrees/fable-loop/.git`. `git` now works in-jail without edits. Caveat: if a real commit fails with a read-only/permission error (NOT "not a git repository"), the jail ro-binds shared .git — stop and tell the User (plan B: standalone clone).
