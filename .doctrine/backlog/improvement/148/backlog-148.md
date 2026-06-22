# IMP-148: MCP memory tool inline help — document response fields, expose validate, add examples

The four MCP memory tools (`memory_find`, `memory_retrieve`, `memory_show`,
`memory_list`) defined in `src/mcp_server/tools.rs` L195–272 have solid safety
guidance and workflow chaining. But response fields are opaque to agents, two
parameter ambiguities exist, and `memory validate` isn't exposed.

**Context budget constraint.** MCP tool descriptions ride every turn —
`tools/list` re-sends them on every response. So inline help must be terse;
reference material belongs in on-demand retrievals. This reassessment
(2026-06-22) weights each gap against its per-turn token cost.

## What works (keep)

- **Workflow chaining.** `find` → `show` → `retrieve` is documented.
- **Safety rules.** Holdback-exempt on `find`, suppression on `retrieve`,
  held-back warning on `show` — all covered.
- **Token consciousness.** `view: summary` / `include_body: false` documented.

## Gap 1: response fields undocumented → enrich the shipped concept memory instead

**Do NOT inline a field reference into every tool description** (~1200-2000
tokens/turn). Instead:

1. **Enrich `mem.concept.doctrine.memory-model`** (the shipped concept, authored
   in `memory/mem.concept.doctrine.memory-model/memory.md`). Add an "MCP
   response fields" section documenting every field an agent sees:
   `staleness` (fresh/stale/reference/unknown semantics), `consumable`,
   `held_back_on_retrieve`, `verification_state`, `weight`, `spec`,
   `next_offset`. One canonical source, retrieved on demand.

2. **Add a single reference line to each tool description**, e.g.:
   > Response shape: see `mem.concept.doctrine.memory-model` for field semantics.

   ~15 tokens per tool. No duplication, no drift.

## Gap 2: `lifespan` parameter ambiguous → fix inline (~20 tokens)

Current: `"Filter by lifespan threshold"`

Fix: `"Filter to memories at or above this durability (semantic ⊃ episodic ⊃ procedural ⊃ working ⊃ identity)"`

The ambiguity cost (wrong memories surfaced, second round-trip) dwarfs 20
tokens.

## Gap 3: `memory_show` `include_body` vs `view` interaction → fix inline (~5 tokens)

Current: `"Include body text in result (default: true)"`

Add: `"(ignored when view=summary)"`.

## Gap 4: `memory validate` not exposed as MCP tool → add

No per-turn cost (tool descriptions only ride `tools/list` responses). Wrap
`memory::run_validate` (`src/memory.rs` L3043). Optional `reference` param for
scoped checks. Findings returned as structured array — exit-0/1 semantics
mapped to success/error with findings payload.

## Gap 5: no `memory record`/`memory edit` MCP exposure → defer

The `/record-memory` skill exists and works. Direct MCP verbs would be cleaner
(long-term) but aren't urgent. Lower priority than Gap 4.

## Gap 6: examples → skip (too expensive)

A compact JSON example block is ~200 tokens per tool, sent every turn. Skip
for now. If thrashing is observed (agents misusing tool output shapes),
revisit with one compact example on `memory_show` only (the most-used tool).

## Gap 7: shipped memory-model concept is thin → add MCP response-field section

`memory/mem.concept.doctrine.memory-model/memory.md` currently explains the
two-face model (local vs shipped) and retrieval habits. It doesn't document
the MCP response surface at all — agents see fields like `staleness`,
`held_back_on_retrieve`, `consumable`, `spec` with no definition.

Add a **client-facing** "MCP response fields" section documenting every
field an agent encounters: `uid`, `key`, `title`, `status`, `type`,
`staleness` (fresh/stale/reference/unknown semantics), `severity`, `trust`,
`spec`, `consumable`, `held_back_on_retrieve`, `verification_state`,
`weight`, `review_by`, `next_offset`, `backlinks`, `backlinks_total`,
`relations`, `wikilinks`.

This is the canonical reference that the Gap 1 reference lines point at.
Edits go in `memory/mem.concept.doctrine.memory-model/memory.md` (the
authored source).

Note: this gap is *client-facing* — the shipped concept is retrieved by
agents in any repo. The authorship flow for shipped memories (edit
`memory/`, rebuild, `doctrine memory sync`) is a project concern that
belongs in the project's own AGENTS.md or a project-local memory, not in
shipped guidance.

## Gap 8: `resolve_show` doesn't fall back to `shipped/` — read verbs fail for shipped memories

`resolve_show` (`src/memory.rs` L2110) resolves only against
`MEMORY_ITEMS_DIR` (`.doctrine/memory/items/`). It has no fallback to
`shipped/`, unlike `collect_all` and `read_body` which both union items +
shipped. Consequence:

| Verb | Works for shipped? | Error |
|---|---|---|
| `find` / `retrieve` / `list` | ✓ | (uses `collect_all`) |
| `show` (CLI + MCP) | ✗ | "memory not found" |
| `verify` | ✗ | "memory not found" |
| `edit` / `tag` / `status` | ✗ (correctly guarded) | "shipped/ is read-only" |

The `edit`-family verbs use `resolve_memory_toml_path` (L2309) which DOES
check `shipped/` as a fallback diagnostic — they give the right error.
`resolve_show` needs the same pattern: try `items/`, fall back to `shipped/`.

This blocks `memory_show` from rendering shipped memories (both CLI and MCP).
`verify` on shipped memories is arguably correct to reject (no git anchor
means nothing to verify against), but `show` is broken.

**Fix**: add `shipped/` fallback to `resolve_show`, mirroring
`resolve_memory_toml_path`. Shipped memories are read-only so the returned
`dir` path must not be used for writes — but `run_show` and `run_verify`
already only read the toml/body (verify writes via `stamp_verification` which
would fail on a shipped path, but verify on shipped is semantically wrong
anyway).

## Gap 9: shipped-concept installed copy is stale

`memory/mem.concept.doctrine.memory-model/` is the authored source, but the
installed copy in `.doctrine/memory/shipped/` dates from the last
`doctrine claude install` (2026-06-17). Any edits to the source won't reach
agents until a rebuild + reinstall cycle. This is a project concern — the
same footgun as skills (`mem.pattern.build.rust-embed-no-rerun`). A project
AGENTS.md or memory entry should document the full flow so future agents
don't lose work editing `shipped/` directly.

- Tool definitions: `src/mcp_server/tools.rs` L195–272
- Handler dispatch: `src/mcp_server/tools.rs` L469–694
- CLI validate verb: `src/memory.rs` L3043 (`run_validate`)
- CLI show resolution: `src/memory.rs` L2110 (`resolve_show`) — items/ only, no shipped/ fallback
- CLI edit resolution: `src/memory.rs` L2309 (`resolve_memory_toml_path`) — items/ → shipped/ with guard
- Collective load: `src/memory.rs` L2589 (`collect_all`) — items/ ∪ shipped/, items wins
- CLI sync verb: `doctrine memory sync` (materializes `memory/` → `shipped/`)
- Shipped memory source: `memory/` (RustEmbed via `src/corpus.rs` L44)
- Memory model shipped concept: `memory/mem.concept.doctrine.memory-model/`
- Installed copy: `.doctrine/memory/shipped/` (gitignored, `memory sync` output)
- Re-embed footgun: `mem.pattern.build.rust-embed-no-rerun` (touch src/corpus.rs to force recompile)
