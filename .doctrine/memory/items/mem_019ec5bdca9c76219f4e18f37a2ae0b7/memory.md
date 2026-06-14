# Corpus-walk invariant test compiled in a worktree validates the wrong branch corpus (false green)

Corpus-walk tests use env!(CARGO_MANIFEST_DIR); a worktree-compiled binary on a divergent branch reads that branch's corpus, false-greening a violation on the target branch
