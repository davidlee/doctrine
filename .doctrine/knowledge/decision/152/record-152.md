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

A repo-root floor — `Jail(<main_root>)` — was the recommended option, and it
was rejected on its effects, not its price. The price is low: `Target::Jail`
already carries exactly one path and `bwrap_core_argv` (jail.rs:537) has no
worktree-specific logic in it, so the floor is the existing variant handed a
different argument.

What decided it:

- **In this environment the floor walls off nothing that matters.** Development
  runs inside an outer bubblewrap jail; `$HOME` and `/tmp` are already
  jail-local and disposable. The floor's whole additional catch is throwaway
  state. **This premise is environment-dependent** — see the caveat below.
- **It has a standing cost.** `Jail(p)` mounts `--tmpfs /tmp`
  (jail.rs:547-548), so a floored subagent gets a fresh empty `/tmp` and cannot
  see the orchestrator's. The session scratchpad lives there; losing it burns
  tokens on every delegation.
- **The objection to pass-through is disposed of by opt-in, not by the jail.**
  The wall ships in the binary to consumer projects that have no outer jail.
  But it is activated by hooks the project chooses to install, so an
  unconfined non-worktree subagent there is no worse off than in a project
  that never installed doctrine.

Two adjacent facts that support the posture without being what decided it: the
harness already denies subagent `Edit`/`Write` against the shared checkout
natively, before any hook runs; and `RSK-225` — writable `mcp__*` tools resolve
against the primary repo root, outside every subagent's bwrap, proven under
*both* verdicts — means a `Bash` floor is posture rather than containment
anyway.

## The environment caveat

Leg 1 is the decisive one and it does not travel. On an unjailed host, `$HOME`
and `/tmp` are not disposable, and a repo-root floor *would* stop an accidental
write that pass-through allows. Someone reading this record on such a host will
find its reasoning apparently intact while its load-bearing premise is false.
What makes pass-through safe to ship there is the hook opt-in, and nothing
else.

## The residual this leaves

The fourth arm is where "I could not confirm this cwd" currently lands, and
that case is not the same as "this is an ordinary subagent". Pass-through
grants both. Whether the design keeps a discriminator for the unconfirmable
case is `inq-5` in the SL-247 design run — not settled here.

Related: [[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].
