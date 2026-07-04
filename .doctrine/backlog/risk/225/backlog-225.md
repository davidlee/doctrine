# RSK-225: Worker MCP permissions bypass the SL-182 write wall; arm-divergent and unvalidated

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The risk

The SL-182 `PreToolUse` confinement wall mediates **only** `Bash|Edit|Write`
(`src/worktree/pretooluse.rs` `decide()` → `PassThrough` for every other tool). So a
dispatch **worker granted any writable MCP tool** performs writes the jail never sees.
Concretely: the doctrine MCP server already exports writable `memory_record` /
`memory_edit` (`src/mcp_server/tools.rs`); a worker holding the server broadly (rather
than one specific gated tool) could write `.doctrine/memory/**`, **violating its
source-only contract** — unmediated by the wall, on both the confinement floor and the
import belts. Surfaced by codex adversarial review of the RFC-005 design.

## Why it is a risk, not just a design note

The same passthrough is what RFC-005 **Mode B relies on** (privileged writes via
unconfined MCP tools). So the property is dual-use: an *enabler* if the worker toolset
is tightly pinned, a *hole* the moment it is not. The bad default is easy to reach —
attach a normal MCP server to a dispatch project and workers inherit its write tools.

## Two things that make this sharp

1. **Enforcement is not in place.** A subagent can only call tools in its def's
   `tools:` frontmatter (harness-enforced), so pinning is *possible* — but there is
   **no guard today** that a worker def stays tight. Nothing tests "the only writable
   MCP tool a worker holds is the gated commit tool". Needs a conformance lint on
   worker agent-defs.

2. **Arm divergence (the strong fork).** The passthrough is a **Claude-arm** property:
   the MCP server is a separate process reached through the harness, unconfined
   relative to the subagent's bwrap. On the **subprocess arm** (codex/pi), a worker's
   **stdio** MCP server is a *child of the jailed process* and **inherits the jail** —
   so it does NOT pass through. Therefore Mode B's "MCP server is unconfined" holds on
   the claude arm but **not** for stdio MCP on the subprocess arm. A **persistent
   (http/sse) MCP** server launched outside the jail could unify both arms as
   unconfined — but that is a deployment departure from stdio simplicity. The design
   must be explicit about **where unification (the ideal) is served vs where the arms
   diverge on pragmatic grounds**, and must not silently assume the claude-arm shape
   generalizes.

## Empirical gate — DISCHARGED 2026-07-04 (probe witnessed both arms)

The claim "an `mcp__*` **write** tool bypasses the wall" was inferred from `decide()`;
it is now **witnessed live** (Claude Code 2.1.198). A `general-purpose` subagent called
`mcp__doctrine__memory_record` under both wall verdicts:
- **Reject arm** (cwd = primary): raw Bash + Write denied
  (`worktree-jail: cwd-not-a-worktree`), MCP write **succeeded** → entity written to
  primary `.doctrine/memory/items/`.
- **Jail arm** (`isolation: worktree`): Bash writes to primary + shared `.git` blocked
  (RO), MCP write **succeeded** with `canonical_path` in the **primary** tree (files
  verified present in primary, absent in worktree) — escaped the jail cwd.

**Subprocess-arm half confirmed** (not live-driven — architectural): the claude-arm
MCP server is a stdio child of the top-level harness (outside every subagent bwrap),
whereas a subprocess worker's stdio MCP would be a child of the jailed `pi` and inherit
`bwrap --ro-bind / /` (`scripts/pi-spawn-confined.sh`) → no bypass. Current pi workers
run `--no-extensions` with no MCP at all. Full evidence + the promotion inferred→
witnessed: [[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].

**Gate result:** RFC-005 Mode B may now rest on MCP-mediated writes **on the claude
arm**. The risk itself stays **open** — the *enforcement* half (§"Two things", #1) is
unbuilt: no conformance lint pins worker `tools:`, and the arm-unification posture is
undecided. Those are the remaining mitigations below.

## Mitigations (candidate)

- Pin worker `tools:` to the single gated commit tool; never a broad `mcp__doctrine`.
- Conformance lint on worker agent-defs (fail on any other writable MCP tool).
- Decide the arm-unification posture explicitly (persistent http MCP vs per-arm).
- Consider extending the wall to classify `mcp__*` write tools (harder — the wall has
  no path arg for arbitrary MCP tools).

Relates to RFC-005, IMP-253 (the gated worker-commit tool), SL-182 (the wall),
[[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].
