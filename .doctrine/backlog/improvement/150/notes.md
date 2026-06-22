# IMP-150: Implementation Notes

## Phase A — Response Fields (2026-06-22)

**Done.** Added `Returns: { ... }` blocks to all 10 review MCP tool descriptions in `src/mcp_server/tools.rs`.

- 6 output shape classes mapped to 10 tools
- Shapes sourced from `ReviewOutput` enum in `src/review.rs` L767, verified against `project_show_summary` for `view=summary` behaviour
- Modeled on IMP-148 memory tool pattern (L206-277)
- Commit: `c00a0441` (edge) — `doc(IMP-150): add Returns blocks to all 10 review MCP tool descriptions (Phase A)`

**Verification:**
- `cargo clippy` — zero warnings (unchanged)
- `cargo test mcp_server::tools::tests` — 31 passed, 1 pre-existing failure (`tools_list_response_structure`: 15 tools vs expected 14, IMP-148 artefact)
- `just check` — same result (pre-existing failure only)

**Surprises:** None. All shapes matched the enum exactly. `ListRow.id` is `String` (canonical format), not `int`. `Primed.is_seed` is `#[serde(skip)]` — documented both modes without mentioning the skipped field.

**Next:** Phase B — workflow/protocol guidance.

## Phase B — Workflow/Protocol Guidance (2026-06-22)

**Done.** Added inline protocol notes to 6 role-bearing tool descriptions, sourcing invariants from `review-ledger.md`.

- `review_new`: protocol start, prime-before-raise workflow, parent-tree caveat
- `review_raise`: raiser-ownership of severity/title/detail, append-only ledger, --as note
- `review_dispose`: full 5-value disposition vocab, --as note
- `review_verify`: --note is ephemeral baton chatter, not durable rationale; --as note
- `review_contest`: same --note caveat; --as note
- `review_withdraw`: --as note
- Commit: `5f45294b`

## Phase C — Examples

**SKIPPED** (user decision). Examples would bloat tool description payloads sent on every `tools/list` response. The Returns blocks from Phase A already carry field semantics; full examples belong in reference docs like `review-ledger.md`.

## Phases D–F — Small Fixes (2026-06-22)

**Done.** Three targeted parameter description fixes in one commit (`a7858267`):
- Phase D: `review_dispose` disposition vocab expanded from 3 to 5 values
- Phase E: `review_show` view=summary — explicit field-level blanking behaviour
- Phase F: `review_prime` seed mode — noted count-zero shape divergence in parameter description

**Test fix-ups (pre-existing IMP-148 artefacts):**
- `95b2fb07` — unit test assertion 14→15
- `a7ebdaa2` — e2e vt2_tools_list assertion 14→15, added `memory_validate`

## Walkthrough Corrections (2026-06-22)

Walkthrough agent exercised all 10 tools end-to-end. Six corrections applied in commit `464fef79`:

1. **Variant wrappers:** all Returns blocks now show serde variant tags (`{"Created": ...}`, etc.)
2. **review_prime seed counts:** "counts zero" → `tracked_count` reflects git-changed path count
3. **review_status rounds:** "baton handoffs" → "all finding-state transitions"
4. **review_show format:** "default: table" → "default: json" (MCP context)
5. **--note destination:** "ephemeral baton chatter for the log" → "written to the baton handoff log (persisted but not surfaced in review_show or review_status)"
6. **review_prime seed param:** same counts fix applied to seed parameter description

**Spawned backlog items:**
- [IMP-151](../151/backlog-151.md) — `seed` field: JSON Schema says optional, Rust serde requires it
- [IMP-152](../152/backlog-152.md) — Document domain_map TOML format (shipped memory/reference)
- Path parameter on review tools: wontfix
