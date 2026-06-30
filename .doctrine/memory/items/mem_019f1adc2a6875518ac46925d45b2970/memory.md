# Seatbelt write-floor confines a real isolation:worktree subagent in-situ (RSK-014 H2 pass 2)

Proven on claude-code 2.1.197 (macOS, Darwin 25.4.0 arm64), RSK-014 H2 pass 2,
for SL-183. Full apparatus + matrix + verdict:
`.doctrine/backlog/risk/014/probe-h2-seatbelt/{results.md (Pass 2), insitu/}`.

## What's proven
A `PreToolUse(Bash)` hook that rewrites a worktree-subagent's command via
`updatedInput` into a nested `sandbox-exec` write-floor (`allow default` / `deny
file-write*` / re-allow realpath'd worktree+TMP) **hard-confines the subagent
in-situ**. The floor composes *inside* Claude's own native subagent Seatbelt — the
nesting is NOT refused. macOS analog of the Linux/bwrap arm
([[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]]); the profile
builder itself is [[mem.pattern.dispatch.seatbelt-write-floor-rule-ordering]].

Clean evidence = the **yolo (bypassPermissions)** armed-vs-control pair: armed ⇒
every external vector `Operation not permitted`, all canaries intact; control
(floor bypassed, same mode) ⇒ all four canaries breached. Same mode + vectors,
sole difference = the floor ⇒ every denial is Seatbelt, nothing else. Child/
detached writes (python, `nohup`) inherit the floor; `at`/`launchctl submit`
denied.

## Three load-bearing facts for the Rust port

- **`updatedInput` IS honoured on macOS** (the H1b analog — previously Linux-only):
  all armed-yolo calls logged `WRAP`; the rewritten command ran, the original
  never did. The opaque base64 wrap survives the round-trip.
- **F-G — derive the worktree from PreToolUse `cwd` via git, NOT a path template.**
  macOS Agent-tool worktrees land at `<repo>/.claude/worktrees/agent-<id>` — ≠
  Linux's `<repo>/.worktrees/agent-<id>`, ≠ the `/private/tmp` location assumed
  earlier. The path is harness-version surface; the git relationship (toplevel ≠
  main checkout, via `--git-common-dir`/`--git-dir`, realpath'd) is invariant. The
  `Jailer` MUST bind via git. (The `cwd == agent worktree` invariant held under
  concurrency across 4 agents — cf. the bwrap memory's binding facts.)
- **F-F — in the SUBAGENT context the permission gate is NOT transparent to writes**
  (unlike the pass-1 orchestrator finding F-C). Under `auto`, gate / interactive
  operator-popup denials stop most vectors *before* Seatbelt, masking the boundary
  under test; only bare absolute `echo > file` slipped through and wrote.
  **Therefore: in-situ subagent containment claims MUST be measured under
  `bypassPermissions`**, or the gate confounds the result. The wrapper logs
  `permission_mode` per call so the matrix reads off the log.

## Operational
Hook registration loads at **session start only** (no hot-reload) — wire the hook,
then restart, then run. Fail-closed binding unchanged from the bwrap arm: pass
through ⟺ `agent_id` absent (orchestrator); present ⇒ `cwd` must be a git worktree
≠ main checkout, else deny. Never trust the red-team vector's self-report — the
independent canary checksum verifier is truth (F-D).
