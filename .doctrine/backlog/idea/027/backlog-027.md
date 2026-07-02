# IDE-027: MCP tool surface for dispatch/worktree orchestrator

Project a curated slice of the `/dispatch` + `/worktree` surface as MCP tools —
following the memory/review precedent — to cut orchestrator token burn on the
most complex workflow we have. **Scoped deliberately: contextual help + state
queries, NOT state ownership or wholesale git-wrapping.**

## Origin

Raised from the RFC-011 token-efficiency benchmark. Dispatch/worktree is the
most complex orchestrator surface by a long margin (two arms, hook positional
discrimination, strict funnel ordering, base-pinning, heavy raw git). Question
posed: would an MCP tool — like the existing `memory`/`review` tools, which
earned MCP for their complex command interfaces — improve orchestrator
performance by *integrating contextual help*, absorbing raw git, carrying
runsheets, and optionally tracking its own state?

## The precedent (why it's cheap and additive)

Each doctrine MCP tool (`src/mcp_server/tools.rs`, 10 tools today) is a thin
JSON-schema wrapper over an existing CLI verb — dispatch is a `match name` arm
in `call_tool` delegating to the same `run_*` handler the CLI uses. The MCP
server is **stateless** (60-line `mod.rs`; `dispatch()` is a pure request→
response `match`). Marginal cost of a new tool = one schema entry + one match
arm + description + an e2e case. The **description field is the product**: it
carries per-call contextual help the CLI can't ("next: `review_prime` …",
"Returns {…}", "refuses worktree/fork-resolved roots"). That is exactly the
"integrating contextual help" lever.

## View: yes — but split the ask into three mechanisms, accept two, reject one

The framing conflates three distinct things. They have very different ROI/risk.

### 1. Contextual help in tool descriptions — STRONG YES (cheapest, safest)
An MCP description is a **guaranteed-in-context, per-call docstring** the
orchestrator cannot skip the way it skips re-reading a 135-line SKILL.md. For a
funnel with strict beat ordering (precond → delta → R-5 belt → import → verify →
branch-point → commit → record) and dense refusal semantics, this collapses
"route → read SKILL → derive command shape" into "call tool; schema states the
next beat." Anchor descriptions to durable invariant ids (INV-1, R-5, `S^==B`),
not re-prosed narrative, to bound drift against the SKILL.

### 2. State query + a reconciliation primitive — YES, but CLI-first
The dominant empirical token sinks from the case notes are **coordination-state**
problems, not command-shape problems:
- **Phase-status split-brain** — runtime state lives per-tree in gitignored
  `.doctrine/state/`; primary tree shows `2/5` while coord shows `4/4`. ~4-6
  investigative calls per occurrence. #1 sink.
- **Worktree discovery blindspot** — audit reads the main tree first; the
  implementation lives on a dispatch worktree; operator nearly concludes "nothing
  to audit." No `git worktree list` discovery beat.
- **Provisioning gaps** — gitignored embedded build roots (e.g. `web/map/dist/`)
  missing from candidate/audit worktrees → compile fail, manual `cp -r`.
- **Stale-binary corpus corruption** — a PATH binary predating a slice's new
  verbs silently drops labels / mis-appends TOML.

The fix for each is a **CLI verb** (which also serves codex/pi orchestrators that
may lack the doctrine MCP); MCP then wraps the complex/queried ones for ergonomics
+ self-documentation. Candidate reads: which-tree-is-canonical status, worktree
inventory + provenance, a provisioning/artifact check, binary-freshness assert,
and a **reconciliation** primitive that pulls phase-completion from the canonical
source (coord) before `prepare-review`.

### 3. State ownership + wholesale git-wrapping + strict enforcement — REJECT
The seductive over-reach:
- **MCP owning dispatch state** would be a **fourth source of truth** competing
  with git refs, the boundaries ledger, and the conformance registry — precisely
  the split-brain the case notes already flag as sink #1. The architecture is
  deliberately sole-writer + disposable gitignored runtime state + committed
  authored TOML + **git refs as the durable ledger**. Enforcement must stay in
  fail-closed CLI verbs and the `prepare-review` completeness gate. MCP may
  *surface* state (read refs/ledger); it must **not own** it. Making the stateless
  server stateful is a category change with real risk.
- **Wrapping git wholesale** is mostly already done (`worktree import`, `land`,
  `branch-point-check`, `verify-worker`, `gc`, `fork`, `coordinate`). Wrap
  **funnel beats** (capture-B → verify-clean → import → commit → record as one
  ordered, un-skippable call), not git semantics. A fat git-proxy tool duplicates
  git and invites the "drunk with a chainsaw" failure the CLAUDE.md warns against.

## Complicating factors / risks

- **CLI stays source-of-truth and complete (AGENTS.md).** MCP is an ergonomic
  projection, never the only path — else a codex/pi orchestrator without the MCP
  regresses. Mirrors memory/review (both CLI-complete).
- **Two-arm asymmetry.** MCP tools run in the *orchestrator's* harness. Workers
  (subagents/subprocesses) don't get the doctrine MCP, so no worker-mode-classing
  escape — but state it explicitly; orchestrator/worker classing must survive.
- **The cwd-positional spawn resists MCP.** The claude arm's arm-spawn/cd-in/
  cd-back is a Bash-cwd protocol; an MCP tool can't set the orchestrator's Bash
  cwd. So MCP wraps the deterministic **post-spawn funnel + queries**, not the
  harness-specific spawn choreography. Honest scoping.
- **RFC-005 is unsettled.** Topology/.git-writability/import-mode are becoming
  per-dispatch configurable (Path L / Path C). Do **not** build policy/config
  tools onto a moving target yet — start with the *stable* beats (status,
  reconcile, funnel-verify) + contextual help; defer a config/policy surface until
  RFC-005 settles. RFC-005 does not itself recommend MCP.
- **Not everything is MCP-shaped.** The case notes also surface problems MCP
  cannot fix: CLI id-form inconsistency (bare `166` vs prefixed `SL-166` across
  the same verb family — a parsing bug), design-discipline divergence, and the
  module-layering-gate chicken-egg. Route those to CLI/prose/architecture fixes,
  not this idea.

## Proposed shape (for the design conversation)

1. **CLI-first**: add the query + reconcile verbs (canonical-tree status,
   worktree inventory, provisioning/artifact check, binary-freshness,
   phase-status reconcile). Benefits all arms.
2. **MCP projection**: wrap the complex/queried verbs + a small set of funnel-beat
   tools, each with a contextual-help-rich, invariant-id-anchored description.
3. **Explicitly out of scope**: MCP-owned state, a git-proxy tool, config/policy
   tools (deferred to RFC-005), the cwd-positional spawn.

## Timing

"Settled CLI surface" is the right gate — but read it **per the specific verbs
being wrapped**, not globally. As of 2026-07-02 the funnel verbs are demonstrably
NOT settled: RFC-005 is open (updated today), and the exact wrap targets are all
in-flight — SL-180 (import-belt scope-creep refusal → *import* beat), SL-181
(worker ref-corruption guard → *spawn* safety), SL-185 (subprocess Seatbelt →
*spawn* arm), SL-187 (per-harness delivery/boot → *arm routing*), SL-189 (pi-arm
boundary recording scope → *Record* beat). Wrapping now = moving target + drift +
rework.

Split by stability:

- **Now** — the CLI-first state-query/reconcile verbs (mechanism 2's CLI half:
  canonical-tree phase-status + reconcile, worktree inventory, provisioning/
  artifact check, binary-freshness). Topology-independent (split-brain is an
  architectural constant, not an RFC-005 variable), #1 empirical token sink,
  all-arm benefit. Ripe; also the dependency for the MCP half.
- **Deferred** — MCP projection + contextual-help descriptions + funnel-beat/
  spawn/config wrapping. Gate: RFC-005 decides topology (Path L vs C, import-mode)
  **and** SL-180/181/189 close (import belt + ref-guard + Record beat stable),
  **and** arm-record converges (IMP-171/D6 symmetric derive — lets one tool wrap
  Record symmetrically rather than encoding today's per-arm asymmetry). Contextual
  help is drift-bait until the beats it narrates stop moving.

## Related

- RFC-011 — origin (token-efficiency benchmark / case notes).
- RFC-005 — dispatch funnel hazard survey + emerging per-dispatch config surface;
  gates the policy/config tools deferred above.
- `memory`/`review` MCP tools (`src/mcp_server/tools.rs`) — the precedent to ride.
