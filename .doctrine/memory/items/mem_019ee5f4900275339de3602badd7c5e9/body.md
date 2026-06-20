To spawn a pi worker via dispatch on the pi arm (`preferred-subprocess-harness = pi`):

1. **Fork**: `doctrine worktree fork --base $B --branch $BR --dir $D --worker`
2. **Prompt**: Build as NDJSON (one JSON object per line). First: `{"type":"set_auto_retry","enabled":false}`. Second: `{"type":"prompt","message":"<single-line-escaped>"}`. Use python3 or similar to properly escape the prompt as a single-line JSON string.
3. **FIFO keepalive**: Create a fifo, write payload + `sleep 300` to it in background, then start pi reading from it. A regular file redirect (e.g. `< file`) closes stdin when EOF hits, causing pi to exit before the agent processes the prompt.
4. **Spawn**: `timeout 300 env -C "$D" DOCTRINE_WORKER=1 $fork_env pi --mode rpc --thinking off --no-extensions --no-skills --no-themes --offline --approve --tools read,bash,edit,write,grep,find,ls < "$FIFO"`
5. **Funnel cadence** after worker returns:
   - Precond: coordination tree clean, HEAD == B
   - Delta check: single non-merge commit, S^ == B
   - R-5 belt: no `.doctrine/` or `.claude/` in diff
   - Import: `git diff B..S | git -C <coord-tree> apply`
   - Verify: `cargo test --bin doctrine` + `cargo clippy`
   - Branch-point guard: coordination HEAD still B
   - Commit: one commit on dispatch/<N> branch

**Pre-existing test failures**: `dispatch_agent_skill_subagent_type_matches_const` and `dispatch_worker_agent_def_name_matches_const` read `/tmp/sl128-repair/install/` paths — environment-specific, not code-caused. Skip in verify.
