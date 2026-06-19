# Notes SL-108: pi dispatch worker integration via RPC mode

Durable per-slice scratchpad — tracked in git.

## PHASE-01 (completed, af78d75f)

- Added pi RPC spawn arm to dispatch-subprocess/SKILL.md
- Bumped shrinkage cap ≤25→≤40
- Found: `.agents/skills/` is separate copy from `plugins/doctrine/skills/` (test source)
  → recorded as mem_019ede1d999579039d1774b214157eaa

## PHASE-02 (completed, e2e validation)

- Fork+spawn+agent_end cadence exercised with pi v0.79.6
- 1840 tests passing, lint green (`just check`)

### Critical findings (design amendments needed)

**F1: RPC stdin lifecycle** (mem_019ede2f6b487d02b160a07dea4759c6)
- pi RPC mode exits on stdin EOF, even with model call in-flight
- Heredoc approach (`<<'PI_MSGS'`) in design and skill template silently fails
- Workaround: named pipe (fifo) with delayed close, or use `-p` (print mode)
- Impact: design.md D1, spawn template in both design.md and SKILL.md need rewrite

**F2: set_auto_retry command format** (mem_019ede2f99a179d2968bfadfee2843a9)
- Design shows `{"type":"request","method":"set_auto_retry","params":{...}}`
- Correct format: `{"type":"set_auto_retry","enabled":false}`
- pi v0.79.6 responds with `Unknown command: request` error

**F3: extension_ui_request subagent-async widget**
- `pi-subagents` package emits `setWidget` with `widgetKey: "subagent-async"` 
- Not suppressed by `--no-extensions` (it's a package, not an extension)
- Fire-and-forget (per RPC docs § Extension UI Protocol), doesn't block execution
- Adds noise to output stream; extraction pipeline should ignore `extension_ui_request` events

### Verified (pass)
- Extraction fallback ladder: all 3 rungs produce correct statuses (VA-3)
- Timeout enforcement: `timeout 5` kills pi (exit 124) mid-execution (VA-4)
- Worker cwd binding: files edited in fork at correct paths (VA-1 partial)
- Delta clean: single-file comment change, no .doctrine touch (EX-2)
- Gate green: 1840 tests pass, zero lint warnings

### Partial
- Import funnel: `doctrine worktree import` refused (head-moved + dirty tree) but
  3-way apply mechanism (`git diff B..S | git apply --3way --index`) verified

### Memories recorded
- mem_019ede2f6b48 — pi RPC stdin lifecycle (pattern, high trust, high severity)
- mem_019ede2f99a1 — set_auto_retry command format (fact, high trust)
- mem_019ede1d9995 — plugins-vs-agents skill duplication (from PHASE-01)

## Next actions
- Design amendment: rewrite spawn template for RPC stdin lifecycle
- Skill amendment: apply corrected template to dispatch-subprocess/SKILL.md
- Consider documenting `-p` (print mode) as the simpler fire-and-forget path
- Deferred: structured extraction (IMP-104), filter for extension_ui_request noise
