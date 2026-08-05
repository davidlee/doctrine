# DEC-152: Non-worktree subagents pass through unconfined

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What was actually decided

The wall exists to keep **worktree** subagents inside their worktree. That is
the whole of its remit. A subagent with no worktree has no boundary to keep, so
it is not the wall's business, and the fourth arm should say so.

The arm was never written *for* ordinary subagents. The fail-closed rule was
phrased *"pass through iff `agent_id` is ABSENT"* — deliberately, because the
obvious alternative (*"jail when in a worktree, else pass through"*) fails open
for `isolation: none`, which carries an `agent_id` with cwd = repo root. That
phrasing is correct for a dispatch worker and fatal for ordinary use; every
ordinary subagent has been collateral since.

## Why not a floor

A repo-root floor is the intuitive middle, and it buys nothing here:

- **No threat model asks for it.** RFC-005 states one, for *dispatch workers*
  only, and already concedes the trust basis is weak — `agent_id` absence is
  "an unauthenticated tell… a named residual" (`rfc-005.md:636`).
- **The harness already has a native Edit/Write floor** for subagents against
  the shared checkout, before any hook runs.
- **RSK-225 outranks it.** Writable `mcp__*` tools bypass the wall entirely on
  the Claude arm — the MCP server is a stdio child of the top-level harness,
  outside every subagent's bwrap, resolving paths against the primary repo
  root. Proven under *both* verdicts. A floor on Bash while `mcp__*` walks
  around it is posture, not containment.

## The residual this leaves

The fourth arm is where "I could not confirm this cwd" currently lands, and
that case is not the same as "this is an ordinary subagent". Pass-through
grants both. Whether the design keeps a discriminator for the unconfirmable
case is `inq-5` in the SL-247 design run — not settled here.

Related: [[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].
