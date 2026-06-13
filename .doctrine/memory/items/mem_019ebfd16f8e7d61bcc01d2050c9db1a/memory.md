# Claude Agent worktree is harness-born, not fork-provisioned — a worktree-creation hook must provision+stamp (WorktreeCreate preferred, fail-closed)

When an orchestrator spawns a claude dispatch worker via the `Agent` tool with
`isolation: worktree`, **the harness creates the linked worktree itself** — the
orchestrator never runs `doctrine worktree fork`, so **ADR-006 D9 provisioning
(the gitignored-allowlist copy, withheld tier excluded) has not happened** and the
disk worker-marker has not been stamped. Both are normally `fork --worker`'s job on
the codex/pi path; on the claude path there is no fork, so **a hook must do them.**

**Preferred seam — the `WorktreeCreate` hook (fail-closed).** Per the Claude Code
hooks docs, a custom `WorktreeCreate` hook **replaces default git worktree creation**:
the command hook prints the worktree path on stdout and **any non-zero exit fails
creation.** So the hook can *own* create + provision + stamp as one trusted act
(mirroring `fork --worker`) — and crucially a provision/stamp failure **fails the
worktree**, so a worker **cannot exist unprovisioned or unstamped.** This beats
stamping *after* creation (e.g. at `SubagentStart`), which leaves a fail-open window:
created-but-unstamped, or stamp-step-dies → the worker runs unbranded and writes
freely.

**The agent_type gate.** Both hooks need to discriminate a dispatch worker from a
benign isolated subagent. `agent_type` (the orchestrator-controlled `subagent_type`)
is the discriminator. **Caveat / open spike:** a live probe (claude-code 2.1.173) saw
`name`, not `agent_type`, on WorktreeCreate — **but it used an *unnamed* subagent (no
custom agent defined)**, so agent_type absence is *expected*, not proof the field is
missing. Confirm a **named `dispatch-worker` subagent reliably propagates agent_type
through WorktreeCreate** before relying on it (the docs say it is present "when the
hook fires inside a subagent"). Also probe whether a WorktreeCreate **matcher** can
scope the hook to one agent_type (else the hook replaces creation for *every*
worktree — incl. `--worktree` launches and benign subagents — and must replicate
default creation for non-dispatch types).

**Fallback ladder if WorktreeCreate lacks agent_type:** (1) `SubagentStart`-stamp
(the probe *did* confirm SubagentStart carries agent_type + cwd, race-free per
subagent) — accept the fail-open window; (2) prompt-enforced worker-sole-writer.

**Why:** missed across 7 SL-056 inquisition rounds and again in the clean-rewrite
first draft (caught as internal-review finding SR-1) — stamping is the visible job,
provisioning is the silent prerequisite that rides the same harness gap, and the
fail-open-vs-fail-closed difference between the two hooks is easy to miss.

**How to apply:** the WorktreeCreate hook reads the payload on stdin and, on the
`dispatch-worker` agent_type, does `git worktree add` (base = session HEAD) + provision
+ `write_marker`, then prints the path; reading agent_type/dir from the **payload**,
never the hook's own process cwd. Relates to
[[mem.pattern.dispatch.claude-subagentstart-worker-identity]] (the empirical hook
facts + the unnamed-subagent caveat) and
[[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]] (the agnostic
floor). SL-056 design.md §4b.
