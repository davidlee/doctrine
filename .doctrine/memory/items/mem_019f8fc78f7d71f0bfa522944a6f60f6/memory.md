# git-common-dir identifies repository membership, not a coordination worktree root

All linked worktrees share the primary repository's git-common-dir; parent(--git-common-dir) resolves the primary worktree only in ordinary layouts and cannot identify which coordination worktree spawned a worker.
