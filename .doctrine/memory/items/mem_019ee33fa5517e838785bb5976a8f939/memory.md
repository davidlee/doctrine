# Candidate worktree is detached; advance the branch + admit by ref

A candidate worktree is checked out DETACHED: an audit fix-now commit must 'git checkout -B <candidate-branch>' to advance the ref before admit; 'candidate admit --candidate' takes the branch REF (refs/heads/candidate/<n>/<label>), not the cand-<id>.
