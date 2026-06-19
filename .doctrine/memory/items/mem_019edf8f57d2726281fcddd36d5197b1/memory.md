# Worktrees share CARGO_TARGET_DIR — builds thrash, just check can be a stale-cache no-op, PATH doctrine is stale

Shared CARGO_TARGET_DIR across worktrees => debug binary may lag source; just check 'Finished' can be a no-op; PATH doctrine is old; verify with strings or cargo build from the worktree
