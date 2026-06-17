# doctrine worktree import corrupt patch workaround

ISS-016: doctrine worktree import generates corrupt patch for git apply --3way. Workaround: git diff B..S | git apply --index manually.
