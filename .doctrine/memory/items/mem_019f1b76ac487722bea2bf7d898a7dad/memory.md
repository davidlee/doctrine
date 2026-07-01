# CLAUDE_PROJECT_DIR present in worktree-subagent hook env

`CLAUDE_PROJECT_DIR` **is** exported into the `PreToolUse` hook *process* env for
`isolation:worktree` subagents, pointing at the **project root** (the anchor tree,
not the subagent's worktree). It is **not** present in the subagent's own
tool-exec env — a probe that runs `echo $CLAUDE_PROJECT_DIR` from the subagent's
Bash sees it empty and will mismeasure. Confirmed by `docs/claude/hooks.md:366`
(hooks export the path placeholders as env vars on the spawned process) and
verified live (SL-182 PHASE-03 VA-1): a live worktree subagent's in-worktree
write was ALLOWED (only possible if the hook resolved the anchor), escape write
DENIED.

This is the empirical basis for SL-182's D-anchor topology check
(`cwd_is_project_worktree`): the hook confirms `cwd` shares the project's
git-common-dir with the `CLAUDE_PROJECT_DIR` anchor. Absent anchor ⇒ fail-closed
Reject.

Distinct from [[mem_019f01e2f7d27fe1886c12ff80811c0c]] ("cannot get *per-worktree*
env via hooks"): the project-root anchor is available; per-worktree/per-worker
state is not. Pairs with the confinement mechanism
[[mem_019f18d2a9307cc38d5e4ba9749e6208]] and the fail-open caveat
`mem.fact.claude.pretooluse-hook-fail-open`.
