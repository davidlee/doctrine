# SL-164 Implementation notes

## PHASE-01: Engine writer parameter
- Added `writer: &mut impl Write` to `run_record` + `run_edit` signatures
- Updated ~36 call sites across `src/memory.rs` (CLI + ~26 tests), `src/boot.rs` ×4, `src/retrieve.rs` ×3
- stderr writes unchanged (advisory notices stay on stderr)
- Followed existing `run_show`/`run_list`/`run_validate` pattern

## PHASE-02: MCP memory write + onboard tools
- 3 new `McpTool` definitions: `memory_record`, `memory_edit`, `doctrine_onboard`
- `memory_record` dispatch: deserializes `RecordParams`, builds `RecordArgs`, calls `memory::run_record` with `Vec<u8>` buffer, parses output to extract uid/path for JSON response
- `memory_edit` dispatch: deserializes `EditParams`, builds `EditFields`, calls `memory::run_edit`
- `doctrine_onboard`: static mapping table + runtime-resolved signpost memory bodies via `retrieve_reference`
- Graceful fallback when signpost memories not found
- VT-2 counts: 15→18 across module header, 3 tests, E2E file
- New tests: 3 unit (error mapping) + 3 E2E (round-trips + onboard)

## PHASE-03: Boot footer and cleanup
- Updated `install/boot-footer.md` to MCP-first instruction with `/retrieving-memory` fallback
- Test rename + module header fix handled by PHASE-02

## Gotchas
- The dispatch worker WorktreeCreate hook didn't fire — workers executed inline on the coordination tree. Phase deliverables are correct, but the isolation spec was not met. Root cause not investigated (likely hook not installed or incompatible subagent isolation mode).
- `install/boot-footer.md` is the authored source; `.doctrine/boot-footer.md` is the install-time copy. Design initially declared the wrong path — selector fixed during audit.
