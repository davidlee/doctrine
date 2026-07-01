# Locate coord-owned dispatch files by layout-strip, not git-common-dir

A dispatch worktree's git-common-dir points at the PRIMARY .git, not the coord root where .worktrees/ + jail dir live; recover the coord root by stripping the .worktrees/&lt;name&gt; layout.
