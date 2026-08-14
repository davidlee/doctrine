# ISS-353: doctrine install never prunes orphaned skills and agent defs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine install` writes and updates the derived tier; it never **removes**. So
when a skill or agent def is deleted from the authored source, its installed copy
survives indefinitely — and stays invocable.

## Found at `SL-254`'s close

`SL-254` deleted the `/dispatch-agent` in-session arm. Its authored source is
gone (`install/skills/dispatch-agent` does not exist). After a full
`doctrine install -y` on the landing tree, three derived copies remained:

| path | what it is |
|---|---|
| `.doctrine/skills/dispatch-agent/SKILL.md` | the deleted skill, still installed |
| `.agents/skills/dispatch-agent/` | the same, in the universal skills tree |
| `.doctrine/agents/claude/` | a stale-layout directory install no longer writes |

Each still describes the deleted arm as live.

## Why it matters

This recreates `RV-356` `F-1` one tier down. `F-1` was *a surface still telling an
agent to use `worker_commit`* — fixed in the authored corpus. Nothing propagates
that fix to a copy the authored corpus no longer knows about, so the retired
instruction stays reachable in every tree that ever had it installed.

The failure is quiet in the way that matters: the skill loads and reads as
current. There is no version marker, no deprecation notice, and no error.

## What the close gate did and did not catch

`doctrine check gate`'s `[Agent Conformance]` leg **did** catch the sibling
defect — two hard errors, a `worker` agent-def holding
`mcp__doctrine__worker_commit` and `mcp__doctrine__observation_record` after the
tool was deleted — and refused the gate until `doctrine install` regenerated it.
That leg checks agent defs that *exist* against the rules; it has no notion of a
file that should no longer exist at all. Orphaned skills are unchecked entirely.

Note the asymmetry that made this visible only at close: the audit worktree
passed the same gate, because a fork has no installed derived tier to be stale
(see `ISS-352`). The landing tree had one, so it failed there and only there.

## Fix directions

Not settled — pick at design time:

- **Prune on install.** Reconcile the installed set against the shipped manifest
  and remove what is no longer shipped. Correct, and the most dangerous: it
  deletes from directories users may have hand-added to. Needs a rule for
  distinguishing *orphaned by us* from *added by them*.
- **Report, don't remove.** Install prints orphans and leaves them; a `doctor`
  check turns it into a standing signal. Cheapest and safest; leaves the stale
  file reachable.
- **Check, don't prune.** Extend the conformance leg to flag installed skills
  and agent defs with no shipped source. Catches it at the gate rather than at
  install, which is where the other agent-def rules already live.

Recommend the third as the floor — it fails closed in the same place the sibling
defect already fails — with the second as a cheap companion.

## Provenance

- `SL-254` — the slice whose deletions orphaned these copies.
- `RV-356` `F-1` — the same class of defect, one tier up, in the authored corpus.
- `ISS-352` — the mirror-image gap: forks never *gain* the derived tier. Together
  these two say the derived tier has no lifecycle at either end.
