---
seq: 0022
scope: codebase
target: src/mcp_server/tools.rs — MCP tool surface (generalises IDE-012)
confidence: high
reversible: yes (proposal only; read-only analysis — nothing authored)
---
## What
The doctrine **MCP server exposes only the adversarial-review workflow** — nothing
that lets an agent harness *read* the graph. `src/mcp_server/tools.rs::tools()`
registers exactly the `review_*` set (`review_new`, `review_list`, `review_show`,
`review_raise`, `review_dispose`, `review_status`, …); there is **no** MCP tool for
`memory retrieve`, `inspect`, `backlog list`, `next`/`survey`, or any read surface.
(Confirmed: `grep -n 'name:' src/mcp_server/tools.rs` yields only `review_*`.)

So the programmatic, harness-facing channel into doctrine is **write-review-only**.
An agent embedded in a harness (Claude Code, CI bot, IDE) can contest and dispose
review findings over MCP, but **cannot ask doctrine the questions that would inform
it**: "what should I work on?" (`next`/`survey`), "what does this change touch?"
(`inspect` — and the transitive version from proposal 0003), "what do we already
know?" (`memory retrieve`), "what's in the backlog?". Those answers exist as CLI
verbs and as a web UI, but not as MCP tools — so a harness must shell out to the
`doctrine` binary (requires the install, the cwd, parsing stdout) instead of a typed
MCP call.

**IDE-012** ("Read-only doctrine memory retrieval tool for agent harnesses") is the
*memory slice* of this exact gap — and note it is **not** a duplicate of the
existing `doctrine memory retrieve` CLI verb (which already ships, with a
non-bypassable trust holdback). IDE-012 is about the *delivery surface* (harness/MCP),
not the retrieval logic. The general finding: the MCP surface needs read tools, and
IDE-012 is one of them.

This is the agent-facing edge of the 0014 thesis (the graph's value is gated on
consumption surfaces): MCP is *the* way agents consume doctrine, and today it
carries only one workflow. The read surfaces are the higher-leverage half — an agent
that can be *informed by* the graph (next, inspect, memory) is far more valuable than
one that can only push review verdicts.

Security note (load-bearing, not a blocker): `memory retrieve` already frames output
as "data, not instruction" with a trust holdback (low-trust/high-severity
suppressed). Any MCP memory tool **must** route through that same holdback — the MCP
boundary is exactly where untrusted memory must not become instruction. The CLI
already solved this; the MCP tool reuses it.

## Options
1. **Add read-surface MCP tools, starting with the highest-value few.** Expose
   `inspect`, `next`/`survey`, `memory retrieve` (holdback-preserving), `backlog
   list` as MCP tools over the existing command-layer functions. Tradeoff: the
   logic exists (thin MCP wrappers over `run_*`/surface functions); cost is schema +
   wiring per tool + preserving security framing. Highest harness-integration ROI.
2. **Ship IDE-012 only (memory retrieve over MCP).** The narrowest slice. Tradeoff:
   smallest; delivers the one read surface most sensitive to get right (trust
   holdback), proves the read-tool pattern; leaves inspect/next/backlog for later.
3. **Leave MCP review-only; harnesses shell the CLI for reads.** Tradeoff: zero
   work; but every harness re-implements CLI-shelling + stdout parsing, and the typed
   MCP contract (the reason MCP exists) is absent for reads — the integration story
   stays half-built.

## Recommendation
Option 1, sequenced with IDE-012 (memory) and `inspect` first. Rationale: MCP is the
agent-consumption channel and it currently carries only review; the read tools are
the half that makes doctrine *inform* an agent's work, which is the indispensable-to-
teams payoff. Start with `memory retrieve` (IDE-012 — reuses the existing holdback,
proves the security pattern at the MCP boundary) and `inspect` (the graph view, and
the natural home for proposal 0003's transitive query later); add `next`/`survey` and
`backlog list` next. Every tool is a thin wrapper over a shipped command function —
no new domain logic, same reuse posture as 0003/0007/0011/0017.

Decisions deferred to YOU:
- (a) **build read MCP tools, or keep MCP review-only** (is shelling the CLL the
  intended harness path)?
- (b) **which surfaces & order** — memory/inspect first (my pick), or lead with
  `next`/`survey` (the worklist)?
- (c) **security review of each read tool at the MCP boundary** — memory's holdback
  is mandatory; does `inspect`/`backlog` output need any framing, or is it inert?
- (d) is **IDE-012** widened into "MCP read surface" or kept as the memory-only slice
  with siblings filed alongside?

## Next doctrine move
```
# confirm the surface (read-only):
grep -n 'name:' src/mcp_server/tools.rs        # only review_*
doctrine backlog show IDE-012                   # the memory slice of this gap

# capture the generalisation (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "MCP read surface: expose inspect / next / memory \
  retrieve (holdback-preserving) / backlog list as MCP tools — doctrine MCP is \
  review-only today; thin wrappers over existing run_*/surface fns. IDE-012 is the \
  memory slice" --tag area:mcp --tag area:relations --tag area:memory
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — each tool is a thin MCP wrapper over an existing command function; the design
question is *which surfaces + the security framing per tool* (decision c), which a
speculative schema would prejudge.
