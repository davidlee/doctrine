# IMP-401: Make plain subagents work; keep worktree isolation

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Set dispatch aside entirely. Get two things working on the Claude harness with
the simplest mechanism that can possibly work:

1. **Worktree-isolated subagents stay in their worktree.** Continued support for
   non-dispatch worktrees — a subagent given a worktree must not climb out of it
   and write across the repo.
2. **Subagents *not* in a worktree work at all.** Today they are over-confined to
   the point of uselessness. This is the acute defect.

RFC-025's capsule programme retires most of dispatch, worker stamping, and the
agent confinement apparatus. This item is explicitly **interim and deletable** —
it should not accrete anything the capsule cutover then has to unpick. Scope it
so that throwing it away costs nothing.

## The acute defect, precisely

`resolve_target` in `src/worktree/jail.rs` has three arms:

| caller | verdict |
|---|---|
| no `agent_id` (interactive/orchestrator) | `Orchestrator` → `PassThrough` |
| `agent_id` + cwd *is* a linked worktree | `Jail(cwd)` — rw only cwd, shared `.git` RO |
| `agent_id` + cwd is **not** a linked worktree | **`Reject`** (`jail.rs:387`) |

That third arm is every plain subagent. It is `Reject` **by design**: the
fail-closed rule was written as *"pass through iff `agent_id` is ABSENT"*,
deliberately **not** as *"jail when in a worktree, else pass through"* — because
the latter fails open for `isolation: none`, which carries `agent_id` with cwd =
repo root. Correct for dispatch. Fatal for ordinary use: an `isolation: none`
subagent gets Bash and Edit/Write denied
(`worktree-jail: cwd-not-a-worktree`) and can do nothing but read and spawn.

So this is not a bug to fix in place — it is a **policy that needs a third
verdict**, and the design question is what a non-worktree subagent should be
allowed to do, given that "the same as the orchestrator" is what the fail-closed
rule was written to prevent.

## What the wall actually covers (do not re-derive)

Verified live on 2.1.198; the coverage is much narrower than "confinement"
suggests, and that narrowness cuts both ways.

- `decide()` matches on `tool_name` only: `Bash` → nested-bwrap wrap,
  `Edit|Write` → `realpath ⊆ cwd`, **everything else → `PassThrough`**.
- `Agent` and `Read` **pass through even under `Reject`** — a rejected subagent
  still spawns nested subagents and reads files. Nested spawns work to ≥ depth-3.
- **Writable `mcp__*` tools bypass the wall entirely** — the MCP server is a
  stdio child of the top-level harness, outside every subagent's bwrap, and
  resolves paths against the primary repo root. Proven under *both* verdicts: a
  `Reject`-ed subagent wrote an authored entity to the primary tree, and a
  `Jail`-ed one's write landed in the primary tree, not its worktree. On the
  subprocess arm this does not happen (the server is a child of the jailed
  process). **Claude-arm only.**
- `PreToolUse` hooks **fail open** — only exit 2 blocks.

**Consequence for leg 1: the Bash wall cannot simply be dropped.** Proven by
control on 2026-07-01 — with the wrapper disabled, a worktree subagent's Bash
wrote everywhere: repo root, shared `.git`, `/tmp`, `$HOME`, the host mount-ns.
Re-arming flipped every vector to `Read-only file system`. The harness imposes
**zero** native Bash containment. It *does* natively deny subagent `Edit`/`Write`
to the shared checkout before any hook runs — so the asymmetry is: Edit/Write has
a native floor, Bash has none. Any simplification must keep a Bash wall for
worktree subagents or accept that "isolation" means nothing there.

## On the per-worktree `.claude` idea

Worth pursuing, but it splits into two cases that behave differently, and only
one of them is the case that needs the wall.

**Case A — a human (or subprocess) starts a session *in* the worktree.** A real
new session, own cwd, own project root, own `.claude/` resolution. Plausible, and
the cleanest form of "isolation binds where the work is". Note the storage
wrinkle: `.claude/settings.json` is tracked, so every worktree on the same commit
sees identical content; per-worktree divergence has to ride the gitignored
`settings.local.json`, written at fork time. `doctrine worktree fork` already
provisions and marks forks, so the seam exists.

**Case B — an in-session `isolation: worktree` subagent.** This is the case the
wall exists for, and the evidence says config does **not** re-bind at that
boundary. A worker inherits the orchestrator's environment frozen at spawn;
`CLAUDE_ENV_FILE` is unset in the subagent; and `CwdChanged` — the one hook that
would carry per-directory context — **does not fire** when the harness places a
worker in its worktree. That is about env rather than settings-file resolution,
so it is not proof, but it is strong evidence that the harness treats a subagent
as the same session, not a new one rooted at the new cwd.

**Verify this before designing on it.** If Case B does re-read a worktree-local
`.claude/`, the idea is a large simplification. If it does not, the idea helps
only Case A — which may still be worth having, but it does not replace the wall,
and the card should not be scoped as though it does.

## Shape of the simplest thing (sketch, not a decision)

- **Third verdict for non-worktree subagents.** Replace `Reject` with something
  useful: pass through, or a floor that denies only writes outside the repo root
  / to `.git`. The fail-closed argument that produced `Reject` was about
  *dispatch* workers; with dispatch set aside, the threat model is a subagent the
  user asked for and wants to be useful.
- **Keep isolation declarative.** Anything that must be deterministic should ride
  agent-def frontmatter, never a per-call `Agent` param — models (Opus included)
  intermittently omit the call-param and the worker then silently runs in-place
  with no fork. Frontmatter `isolation: worktree` is honoured, including for
  nested spawns. Caveat: the agent-def registry is frozen at harness launch.
- **Pin worker tool surfaces.** While writable MCP tools bypass the wall, any
  agent def that must stay confined should list only the tools it needs. This is
  a mitigation, not a fix.
- **Decide what the wall is for, now that it isn't for dispatch.** Probably:
  "don't let a worktree subagent write outside its worktree", with no claim of
  adversarial containment. Saying so plainly is cheaper than the current posture
  and matches what the mechanism can actually deliver given the MCP bypass.

## Relationship to other work

- **RFC-025** — the capsule target uses uniform sandboxed *subprocess* workers
  rather than in-session subagents. Every in-session confinement mechanism is a
  migration casualty by design. Build accordingly: cheap, small, deletable.
- **IMP-400** (retire the Claude plugin channel) — the wall is activated by
  `PreToolUse` hooks that currently ship in `plugins/doctrine/hooks/hooks.json`.
  If hook activation moves to direct-written settings, this wall moves with it,
  and a worktree-local `.claude/` interacts directly with where those hooks are
  written and whether they bind per-worktree. **Sequence or co-design these two.**
- **ADR-011 / ADR-020** — ADR-011 remains incumbent authority until the capsule
  cutover; ADR-020 adopts capsules as the dispatch authority boundary. This item
  sits under the incumbent, and must not pre-empt the successor.

## Open questions

- `OQ-1` — What should a non-worktree subagent be allowed to do? Full
  pass-through, or a repo-root floor? What is the threat being modelled once
  dispatch is out of scope?
- `OQ-2` — Does a worktree-local `.claude/` bind for an in-session
  `isolation: worktree` subagent (Case B)? Probe before designing.
- `OQ-3` — If it binds only for Case A, is Case A worth building on its own —
  i.e. are there real non-dispatch worktree sessions started from the worktree?
- `OQ-4` — Does the wall keep pretending to adversarial containment, or is it
  restated as a guard-rail against accident? The MCP bypass means the first
  claim is not currently true on the Claude arm.
- `OQ-5` — How much of `src/worktree/{jail,pretooluse}.rs` survives capsules? If
  little, prefer the smallest patch to the `Reject` arm over any restructuring.

## Related

- RFC-025: discrete dispatch execution capsules — the successor programme.
- IMP-400: retire the Claude plugin channel — shares the hook-activation seam.
- RSK-014: the probe series that established the confinement mechanism.
- ADR-011, ADR-020: incumbent and successor authority.
