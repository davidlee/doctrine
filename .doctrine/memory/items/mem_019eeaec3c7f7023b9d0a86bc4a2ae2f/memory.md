# worktree fork emits a human status line on stdout before the env contract

doctrine worktree fork prints a 'provisioned …' status line on stdout before the KEY=VALUE env contract; capturing $fork_env unfiltered breaks env $fork_env cmd (rc 127).
