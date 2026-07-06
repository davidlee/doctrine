The User may allow *small* backlog items (cleanup, etc) without the full
doctrine slice workflow; just a quick design conversation / sketch, acceptance
of plan, implementation & close. Ask what they'd prefer, unless it's obviously
non-trivial in which case it should be sliced.

## Where things live

Research in approximate order: specs, ADRs, memories, backlog, slices, ...

## Project-local rules of the road

ALWAYS begin any high-level context gathering with the relevant PRD then SPEC
specs, then use the /retrieve-memory skill.

Finish every turn which references a doctrine entity by printing its ID:
```text
```
[SL-123 phase 03]: short session descriptor

if your first message is a handover from another agent, read it and follow 
the instructions.

## useful commands

just -l                    # list tasks
doctrine <kind> paths <id> # list all files 
doctrine status            # what's going on?
doctrine search            # BM25 entity search 

## CASE NOTES: Instrumentation

We are currently benchmarking token efficiency for RFC-011.

*Instrumentation*: during use of any skill, note (append using `cat >>`) any
incidental complexity, confusion, or other source of token-inefficiency to: 
`.doctrine/rfc/011/case-notes.md`, whether or they relate to the dispatch
orchestrator, worker, or any other agent or root cause. Identify each entry
with `[skill being used; a session-unique identifier]\n` **IMPORTANT:** append 
to the primary working tree, not a linked worktree.

# orchestration

pi dispatch under claude code - use: ./scripts/pi-spawn-confined.sh
note: on the **subprocess (pi) arm** the worker CANNOT self-commit (ro .git for
linked worktrees) → orchestrator imports the working-tree diff. Worthwhile trade.
(The **claude arm** now self-commits via the gated `worker_commit` MCP tool —
generic mechanics in the shipped `dispatch-mechanics.md`.)

cargo --bin doctrine memory # focused tests; don't use --lib

