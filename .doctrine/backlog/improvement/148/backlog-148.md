# IMP-148: MCP memory tool inline help — document response fields, expose validate, add examples

The four MCP memory tools (`memory_find`, `memory_retrieve`, `memory_show`,
`memory_list`) defined in `src/mcp_server/tools.rs` L195–272 have solid safety
guidance and workflow chaining but are missing response-field documentation,
examples, and the `memory validate` verb.

## What works

- **Workflow guidance.** `memory_find`: "Use first to probe context … use
  `memory_show` for inspection then `memory_retrieve` for safe recall."
  `memory_show`: "Use only after selecting an exact uid via `memory_find`."
  `memory_list`: "Prefer scoped `memory_find` for targeted discovery." — clear
  chain.
- **Safety rules.** Holdback-exempt on `find`, low-trust × high-severity
  suppression on `retrieve`, held-back warning on `show` — all covered.
- **Token consciousness.** `view: summary` / `include_body: false` on
  `memory_show` documented.

## Gap 1: response fields undocumented

Every tool returns JSON but no tool description explains the response shape.
An agent sees fields with opaque semantics:

| Field | Values | Tool | Issue |
|---|---|---|---|
| `staleness` | `"fresh"`, `"stale"`, `"reference"`, `"unknown"` | `memory_find` rows, `memory_list` | Agent can't interpret these without guessing. `"stale"` = verified_sha behind HEAD? `"reference"` = signpost? `"unknown"` = never verified? |
| `consumable` | `true`/`false` | `memory_show` | Not mentioned in any tool description. |
| `held_back_on_retrieve` | `true`/`false` | `memory_find` rows, `memory_show` | `memory_find` says rows "may include memories suppressed by `memory_retrieve`" and points at `held_back_on_retrieve` but doesn't name the field. |
| `verification_state` | `"verified"`/`"unverified"` | `memory_show` | Not documented at all. |
| `weight` | integer | `memory_show` | Scoring weight — never explained. |
| `review_by` | date string or empty | `memory_show` | Not mentioned. |
| `next_offset` | integer or null | `memory_find` | Pagination cursor — implicit, but a word would help. |
| `spec` | string or `"-"` | `memory_find` rows, `memory_list` | Links to governing spec — undocumented. |

**Fix**: add a short "Response fields" section to each tool description, or a
single reference block linked from each tool (the memory model concept is
already `mem.concept.doctrine.memory-model`).

## Gap 2: `lifespan` parameter semantics ambiguous

`memory_find` and `memory_retrieve` expose `lifespan` as:
> Filter by lifespan threshold

The lifespans are a durability hierarchy:
`semantic > episodic > procedural > working > identity` (most to least
durable). But "threshold" doesn't say which direction — does it return
memories *at or above* the given level, or *at or below*? An agent has to
guess.

**Fix**: clarify e.g. "Filter to memories at or above this durability
threshold (semantic ⊃ episodic ⊃ procedural ⊃ working ⊃ identity)."

## Gap 3: `memory_show` default interaction confusing

`view` defaults to `"summary"` (body skipped). `include_body` defaults to
`true`. When `view=summary`, `include_body` is silently ignored — setting
both produces the summary view, and the agent may not understand why.

**Fix**: either (a) note the interaction in the `include_body` description,
or (b) change `include_body` default to `null` / omit it, or (c) always
respect `include_body` even under `view=summary` (the cleanest contract).

## Gap 4: `memory validate` not exposed as MCP tool

The CLI verb `doctrine memory validate` (staleness, dangling relations, draft
expiry) is an essential maintenance operation — used in the dreaming skill's
step 1. The MCP surface has no equivalent, so an MCP-bound agent can't check
memory health programmatically.

**Fix**: add a `memory_validate` MCP tool wrapping `memory::run_validate`.
Optional `reference` parameter for scoped checks. Exit 0 / 1 semantics
mapped to response; findings returned as a structured array.

## Gap 5: no `memory record`/`memory edit` MCP exposure

Agents can read memories through MCP but can't create or edit them. The
`/record-memory` skill exists as a workaround (drives the CLI), but direct
MCP verbs would be cleaner and avoid shell round-trips. Lower priority than
Gap 4 since the skill works.

## Gap 6: no examples

None of the four tool descriptions include example invocations or response
shapes. Even a single `memory_show` response example (with the key fields
labelled) would eliminate most guesswork.

**Fix**: one example JSON response block per tool, e.g.:

```json
// memory_show mem_019ed0b6 reference=mem.fact.clap.colorchoice-case-sensitive:
{
  "memory": {
    "uid": "mem_019ed0b6...",
    "key": "mem.fact.clap.colorchoice-case-sensitive",
    "title": "clap ColorChoice parser case sensitivity",
    "status": "active",
    "type": "fact",
    "consumable": true,
    "held_back_on_retrieve": false,
    "verification_state": "verified",
    ...
  }
}
```

## References

- Tool definitions: `src/mcp_server/tools.rs` L195–272
- Handler dispatch: `src/mcp_server/tools.rs` L469–694
- CLI validate verb: `src/memory.rs` L3043 (`run_validate`)
- Memory model concept: `mem.concept.doctrine.memory-model`
