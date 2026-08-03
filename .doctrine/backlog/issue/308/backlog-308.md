# ISS-308: prompt resolve has no role for non-dispatch agents

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

 Bash(doctrine prompt resolve --role agent --band model --model claude-opus-5 2>&1 | head -40; echo "---ROLES---"; doctrine prompt resolve --help 2>&1 | head -30)
Error: unknown role "agent"; expected 'worker' or 'orchestrator'
---ROLES---
Resolve the prompt cascade and emit assembled markdown to stdout

Usage: doctrine prompt resolve [OPTIONS] --role <ROLE>

Options:
  --role <ROLE>        The role: "worker" or "orchestrator"
  --harness <HARNESS>  The harness (e.g. "claude", "pi")
  --model <MODEL>      Model key (e.g. "anthropic/claude-sonnet-4"). Repeatable — each occurrence adds a member to the context trait set (membership matching)
  --arm <ARM>          The dispatch arm: "subagent" or "subprocess"
  --stage <STAGE>      The stage label (e.g. "execute")
  --band <BAND>        Restrict output to specific bands (repeatable). Empty = all bands
  --json               Wrap stdout as a Cursor `sessionStart` hook JSON envelope (`{"additional_context": "<cascade>"}`) instead of raw markdown
  -p, --path <PATH>    Explicit project root (default: auto-detect)
  --color <COLOR>      Control colour output [default: auto] [possible values: auto, always, never]
  -h, --help           Print help

--
this agent is running a design session. It's neither worker nor orchestrator.

suggested starting point: 
- 'agent' as a general purpose prompt
- agent as default, --role optional


