# ISS-504: Capsule adopt worktree unreachable from the jail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`scripts/oubliette.sh` `cmd_back` (capsule adopt / "generation N is home")
runs `git worktree add "$wt" "$branch"` on the host. Git records absolute
paths both ways: the worktree's `.git` file points at
`/home/david/dev/doctrine/.git/worktrees/<name>`, and the admin dir points
back at the host worktree path.

Inside the bubblewrap jail the repo is mounted at `/workspace/doctrine`, so
neither path resolves:

- `git worktree list` marks the worktree `prunable`;
- every git command in it fails: `fatal: not a git repository: (null)`.

So the printed next steps — `check gate`, `slice verify-vt`,
`slice conformance --against <base>..capsule/<ID>/<slot><gen>`, close,
land — can't be run from a jailed agent session. Seen adopting SL-273 c32
(`.worktrees/SL-273-c32`, 2026-09-29).

`git worktree repair` from the jail is no fix: it rewrites the links to jail
paths and breaks the worktree for the host.

## Options

- Run `git worktree add --relative-paths` (git ≥ 2.48; check the version in
  the flake), so the links survive a remount as long as `.worktrees/` stays
  under the repo root. Most likely fix.
- Or `git config worktree.useRelativePaths true` repo-wide, which also covers
  forks made by `doctrine worktree fork`. That has a wider blast radius: ISS-205
  and ISS-209 are the same host/jail path split in other places.
- Or document that audit and close must run on the host (a stopgap).

Related: ISS-205, ISS-209 (same host-vs-jail absolute path problem).
