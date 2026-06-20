# pi-arm dispatch worker operational footguns

Three independent failure modes hit while driving SL-104 through the `/dispatch`
subprocess (pi) arm. All cost a full worker respawn.

## 1. RPC needs COMPACT (one-line) JSON per message
pi `--mode rpc` parses **one JSON object per line**. Building the prompt message
with `jq -Rs '{type:"prompt",message:.}'` (default pretty-print) emits multi-line
JSON → every line fails with `Failed to parse command: Expected property name…`,
the prompt never lands, the worker sits idle. **Fix:** `jq -cRs` (compact). Same
applies to any hand-built RPC line — keep them single-line.

## 2. pi parks on success — it does NOT self-exit
After the model finishes, pi emits a typed `agent_end` event but the process
keeps running, blocked on stdin (the SKILL template's `sleep 300/1500` fifo
keepalive holds stdin open; pi RPC only exits on stdin EOF). So a `timeout`-bound
spawn will burn the FULL timeout even though work finished in minutes. **Fix:**
run a watcher that polls the log for `"type":"agent_end"` and kills the pi +
keepalive pids on match — then the spawn returns at completion, not at timeout.
`grep -q '"type":"agent_end"' "$LOG"` is the completion signal. (Relatedly,
`agent_end.messages[-1]` carries the worker's final report, but the fork's git
state is the real ground truth — read the commit, not the self-report.)

## 3. The worker may git-reset its own work away
A pi worker built the phase correctly (tests green, gate clean per its own report)
then **`git reset` wiped the entire delta** — it hallucinated a "pre-existing WIP",
decided its own edits were stray scratch, reverted them, and never committed. The
fork came back clean (HEAD==B, empty diff, reflog showing `reset: moving to HEAD`).
**Fix (prompt-level):** (a) state the fork is a CLEAN checkout with NO WIP and that
the target file does not yet exist; (b) forbid every work-discarding git verb —
`git reset`/`checkout -- `/`stash`/`clean` — and say the ONLY git it runs is the
final `git add <paths>` + `commit`; (c) for TDD red-proof reversions, instruct it
to EDIT the scratch out, never to git-discard it. Worked first try after that.

## Also: scope verify, don't run `just gate` in the funnel
`just gate` runs `just fmt` = `cargo fmt` which **reformats in place** across the
whole crate — it'll pull in unrelated pre-existing fmt drift (e.g. a base commit
whose `status.rs` isn't rustfmt-clean under the pinned toolchain because a fmt fix
sits uncommitted on the main worktree). Funnel/worker verify should be scoped:
`cargo clippy` + `cargo test` + `rustfmt --check --edition 2024 <touched files>`.

Related: [[mem_019ede2f6b487d02b160a07dea4759c6]] (pi RPC stdin/heredoc lifecycle),
[[mem_019ed44caf5570b29f4bbe4125d31561]] (pi worker cwd binding),
[[mem_019eba28977b7573b10c1c5ac2134296]] (DOCTRINE_WORKER unset for worker verify),
[[mem_019ee083a8ce7ab0966576a6693b5a58]] (DOCTRINE_TRUNK_REF for setup/sync off unpushed local trunk).
