# IMP-253: Gated worker-commit MCP tool: jailed workers self-commit through the trusted MCP server

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Origin

Surfaced 2026-07-04 during the RFC-005 "subagent-as-orchestrator" design thread
(see `.doctrine/rfc/005/subagent-orchestrator-probe.md` +
`coordinator-exemption-design.md`). A codex adversarial review noted, as a *hole*,
that the SL-182 `PreToolUse` confinement jail only mediates `Bash|Edit|Write`
(`src/worktree/pretooluse.rs` `decide()` returns `PassThrough` for any other tool);
MCP tools, the `Agent` tool, and `Read` pass straight through — even under the
`Reject` verdict (probe-witnessed: a `Reject`'d depth-1 subagent still spawned a
nested subagent and read files). The owner flipped it from hole to **feature seam**.

## The idea

Expose a single, purpose-built doctrine **MCP tool** — call it
`mcp__doctrine__worker_commit` — that a **jailed dispatch worker** invokes to commit
its own delta. The tool runs in the **unconfined doctrine MCP server** (the
already-trusted sole-writer process), so it can write the shared `.git` that the
worker's own Bash cannot (ro under bwrap). It performs the commit **wrapped in the
funnel's existing belts**:

- pre-commit gate: `doctrine check prove` (fmt-check + lint), reject-and-halt on red;
- scope belt: `classify_import` equivalent — reject `.doctrine/`/`.claude/` writes,
  enforce the slice's design-target selectors (`undeclared-scope` refusal);
- one non-merge commit descended from base B (the importable-delta invariant).

## Why it matters (the payoff)

Today (CLAUDE.md `# orchestration`): *"worker CANNOT self-commit (ro .git for linked
worktrees) → orchestrator must import the working-tree diff."* That dance exists only
because the worker's Bash `git commit` is walled. A gated MCP commit tool **removes
the dance**: the worker self-commits through the trusted server, the orchestrator
imports a *commit* (not a working-tree diff), and **linked worktrees stay** — no need
to switch the confinement model from linked worktrees to raw clones to regain
self-commit. It simplifies both dispatch arms and every audit/close that currently
reasons about working-tree-vs-commit provenance.

It is also the **keystone instance of the "MCP-mediated trusted write" pattern** that
the RFC-005 design adopts as its *default* (B): a confined orchestrator/worker
performs privileged acts through gated MCP tools rather than via a jail exemption —
threat-model-independent (a poisoned agent calling a gated tool cannot exceed the
tool's contract), no fail-open-to-escalation surface.

## Security note (load-bearing)

Because MCP tools bypass the `Bash|Edit|Write` wall, the jail's completeness now
**depends on the worker toolset being pinned**: a worker may hold *only* this one
gated write tool and nothing else writable. Any *other* writable MCP tool granted to
a worker is an unmediated escape. This constraint must be documented wherever worker
toolsets are defined, and ideally enforced (allowlist the worker's MCP tools).

## Scope note

Not the core remit of the subagent-orchestrator work — captured here so the insight
isn't lost. Likely its own slice. Relates to RFC-005, RFC-011, SL-182 (the wall it
rides), and the CLAUDE.md orchestration note it retires.
