The User may allow *small* backlog items (cleanup, etc) without the full
doctrine slice workflow; just a quick design conversation / sketch, acceptance
of plan, implementation & close. Ask what they'd prefer, unless it's obviously
non-trivial in which case it should be sliced.

## Where things live

Research in approximate order: specs, ADRs, policies, standards, memories, backlog, slices, ...

# Symlinks

`doctrine` entity creation commands mint a symlink with the title slug as a convenience. 
Commit these with the entity itself.

## Project-local rules of the road

ALWAYS begin any high-level context gathering with the relevant PRD then SPEC
specs, then use the /retrieve-memory skill.

Finish every turn which references a doctrine entity by printing its ID:
```text
[SL-123 phase 03]: short session descriptor
```

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

## Dispatch precondition: DOCTRINE_BIN → the coord build

Set `DOCTRINE_BIN` to the **coord tree's** `./target/debug/doctrine` for any
dispatch session (the jail forwards it — `flake.nix` `try-fwd-env`; `.mcp.json`
launches the server via `${DOCTRINE_BIN:-doctrine}`). This binary is built from
`dispatch/<slice>` source, so it carries earlier phases' not-yet-promoted
binary-level rule changes (new role / allowlist / check). Without it the server —
and the `worker_commit` commit gate's `just validate` → `doctrine doctor` — run
the stale installed/PATH binary and **false-red a correct fork** (ISS-218; the
`just` recipes resolve `${DOCTRINE_BIN:-./target/debug/doctrine:-doctrine}`, but
the first rung must be set or a non-Rust phase falls through to a stale PATH).
This is a **project** rule (doctrine dogfooding itself), not platform behaviour —
POL-002 keeps cargo/`./target` layout out of the engine (SL-225, DEC-003).

