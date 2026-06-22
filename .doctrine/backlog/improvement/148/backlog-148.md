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

**Shipped-memory authorship flow note:** edits to the shipped concept go in
`memory/` (the RustEmbed folder, `src/corpus.rs` → `#[folder = "memory/"]`),
then `cargo build` + `doctrine memory sync` to materialize into
`.doctrine/memory/shipped/`. Editing `shipped/` directly is ephemeral —
overwritten on next `sync` (or `doctrine claude install`, which calls it).
Analogous to skills: `plugins/` → `touch src/skills.rs` → `cargo build` →
`doctrine claude install`.

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

## Gap 7: shipped memory-model concept is thin → needs MCP response-field section

`memory/mem.concept.doctrine.memory-model/memory.md` currently explains the
two-face model (local vs shipped) and retrieval habits. It doesn't document
the MCP response surface at all. The "Response fields" section from Gap 1
should land here.

Fields to document: `uid`, `key`, `title`, `status`, `type`, `staleness`,
`severity`, `trust`, `spec`, `consumable`, `held_back_on_retrieve`,
`verification_state`, `weight`, `review_by`, `next_offset`, `backlinks`,
`backlinks_total`, `relations`, `wikilinks`.

## References

- Tool definitions: `src/mcp_server/tools.rs` L195–272
- Handler dispatch: `src/mcp_server/tools.rs` L469–694
- CLI validate verb: `src/memory.rs` L3043 (`run_validate`)
- CLI sync verb: `doctrine memory sync` (materializes `memory/` → `shipped/`)
- Shipped memory source: `memory/` (RustEmbed via `src/corpus.rs` L44)
- Memory model shipped concept: `memory/mem.concept.doctrine.memory-model/`
- Installed copy: `.doctrine/memory/shipped/` (gitignored, `memory sync` output)
