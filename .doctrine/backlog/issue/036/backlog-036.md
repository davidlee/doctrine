# ISS-036: dispatch setup forks coord tree off stale origin/HEAD not local main

`dispatch setup` → `worktree::coordinate` (`worktree.rs:1700`) forks the
coordination worktree off `git::trunk_commit` → `trunk_ladder`, which prefers
`origin/HEAD` over local `main` (`git.rs:1042`: `["origin/HEAD", "main",
"master"]`). In a local-first workflow `origin/main` is routinely many commits
behind local `main` (boot itself notes "ahead of origin" is normal). If the
slice being dispatched was authored in local-only commits, the coordination tree
checks out a stale trunk that lacks `slice/NNN/plan.toml`, and phase-sheet regen
fails with `Plan for slice NNN not found at .../plan.toml`.

Hit live during SL-122 dispatch (`origin/main` 6 commits behind). Workaround:
`DOCTRINE_TRUNK_REF=main doctrine dispatch setup …` (also needed for `dispatch
sync` at conclude — same trunk ladder). See `.doctrine/slice/122/notes.md`.

Fix candidates: (a) prefer local `main` ahead of `origin/HEAD` in the ladder;
(b) detect ahead-of-trunk and warn/fail with the `DOCTRINE_TRUNK_REF` hint;
(c) gate-check the plan exists at the chosen trunk before forking and message
clearly. Low effort, removes a silent wrong-base footgun.
