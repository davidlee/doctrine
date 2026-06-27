# Candidate worktree is detached HEAD — move the branch ref + re-admit after repair

dispatch candidate create --worktree checks out DETACHED HEAD; repairs need git branch -f <candidate> + re-admit so the admitted OID includes them
