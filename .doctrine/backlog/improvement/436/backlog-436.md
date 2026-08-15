# IMP-436: Holistic memory access for subagents across worktree and capsule execution

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The state of play

**Subagents currently get no ambient memory surfacing at all.** `doctrine memory
surface` short-circuits the moment the hook envelope carries an `agent_id`:

    // INV-3: a subagent (`agent_id` present) surfaces nothing and runs no
    // retrieve (main-thread only, v1).
    if input.agent_id.is_some() { return Ok(()); }
    — src/memory.rs:10729-10733

Note the `v1`. This was a scoped first-version boundary, not a settled position,
and it has never been revisited against the agent populations that exist now.

The block is **doctrine's own choice, not a harness limitation.** Settings-file
hooks do fire inside subagents — tool events fire the same configured hooks as
in the main conversation, with `agent_id` / `agent_type` in the input
(`docs/claude/hooks.md:264`). The envelope arrives; we drop it.

The consequence is that the agents doing the actual implementation work are the
only ones operating without the memory corpus that exists to stop them
rediscovering footguns. The human's interactive seat — which is *least* likely
to be editing files blind — is the only seat that gets the push channel.

## Populations to reason about separately

They have genuinely different reach, and a single `agent_id.is_some()` test
cannot tell them apart:

| population | corpus on disk? | `doctrine` CLI? | MCP? | ambient today |
|---|---|---|---|---|
| confined dispatch worker (bwrap jail, linked worktree) | yes — `.doctrine/memory/items/` is authored and committed | `DOCTRINE_BIN` is forwarded into the jail (`flake.nix` `try-fwd-env`) — **verify** | no | no |
| capsule subagent, main worktree, unconfined (IMP-434) | yes | yes | by def (`mcpServers`) | no |
| harness fork (`isolation: worktree`) | yes | yes | inherits parent pool | no |

`agent_id` presence answers "is this a subagent", which is not the question. The
questions that matter are *can this agent reach the corpus another way*, *is
serving it cheap here*, and *is it confined*. A capsule worker in the main
worktree is a completely different case from a jailed dispatch worker, and both
are currently handled by the same blunt early return.

## What the alternative costs

Without the push channel, memory for a subagent is pull-only and the pull has to
be deliberate — `doctrine memory retrieve "<topic>"` / `doctrine memory show
<id-or-key>` over `Bash`. That works (see IMP-434), but it depends on an agent
head-down in TDD choosing to stop and ask, which is exactly the moment it won't.
Ambient surfacing exists because that judgement is unreliable in the main
thread; there is no reason to believe it is more reliable in a worker.

The alternative channel is curation: a planner tier that holds the MCP server,
retrieves for the phase, and distils into the runtime phase sheet. That covers
*foreseeable* memory only. Situational memory — the footgun that bites when the
worker touches an unexpected file — is precisely what ambient surfacing was
built for and precisely what no curation tier can anticipate.

## Decide

- Should INV-3 be replaced by a sharper gate than `agent_id.is_some()` — keyed on
  confinement, on `agent_type`, or on explicit opt-in from the agent def?
- If surfacing reaches subagents, does it emit headline + id (as today) or resolve
  bodies? A bare id is worse than silence for an agent with no retrieval path —
  it names knowledge the agent cannot reach.
- What is the token cost of surfacing across a deep agent tree, where the same
  memories may surface to many agents on the same files? Is dedup needed, or a
  per-agent budget?
- Does the jailed dispatch worker actually resolve the corpus and the CLI? Assumed
  above, not verified.
- Does the trust holdback behave correctly for a non-interactive seat that cannot
  be asked to adjudicate a low-trust, high-severity memory?

## Provenance

Surfaced while working IMP-434 (capsule agent defs) — the "worker gets no MCP"
decision looked free until the ambient channel turned out to be closed to
subagents too.
