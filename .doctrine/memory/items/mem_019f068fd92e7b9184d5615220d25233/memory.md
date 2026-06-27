# Gating a dispatch candidate worktree fails on missing generated embed assets

Fresh git worktrees lack gitignored generated dirs (web/map/dist); RustEmbed builds fail there — stage the dir before gating a candidate.
