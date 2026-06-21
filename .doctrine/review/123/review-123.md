# Review RV-123 — reconciliation of SL-131

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This audit reconciles SL-131 (MCP memory retrieval and find tools) against its
design and plan. The slice was executed solo (not dispatched) across 5 phases.

**Lines of attack:**
1. **Writer abstraction conformance** — do all five `run_*` functions accept
   `writer: &mut impl Write` and write to it (not stdout)? Do CLI call sites
   pass `&mut io::stdout()`? Are existing tests byte-identical?
2. **Structured helpers** — are `check_retrievable`, `retrieve_reference`,
   `find_for_mcp`, `filtered_list`, `list_for_mcp`, `backlink_rows_for` present
   and correctly wired? Does the single-gate contract hold (no parallel admission
   logic)?
3. **MCP dispatch refactor** — does `call_tool` return `Result<String>`? Does
   `handle_tools_call` use `McpToolResult::text(out)` (double-encoding guard)?
   Are all 14 tool definitions present?
4. **Handler wiring** — do the 4 memory handler arms correctly call the
   structured helpers? Is mutual exclusivity enforced in `memory_retrieve`?
   Does `memory_show` do the single-scan enrichment (check_retrievable +
   backlink_rows_for)?
5. **Skill file updates** — are retrieve-memory, reviewing-memory, and
   audit SKILL.md files updated per design §5?
6. **Test evidence** — do 13/13 E2E MCP tests pass? Are unit tests
   comprehensive? Is clippy clean?

**Invariants held:**
- Byte-identical output for all existing tests (behaviour-preservation gate)
- No double-encoding in MCP responses
- `check_retrievable` is the sole admission gate — no parallel logic
- `filtered_list` shared between CLI (`list_rows`) and MCP (`list_for_mcp`)
- Parse helpers wrap errors with `"invalid arguments: "` prefix for correct
  error code mapping

## Synthesis

SL-131 implemented MCP memory tools with high conformance to the design.

**What went well.** The writer-abstraction refactor (`writer: &mut impl Write`)
was clean — all five `run_*` functions now accept the writer param, CLI call
sites pass `&mut io::stdout()`, and all existing tests produce byte-identical
output (behaviour-preservation gate satisfied). The structured-helpers layer
(`check_retrievable`, `retrieve_reference`, `find_for_mcp`, `filtered_list`,
`list_for_mcp`, `backlink_rows_for`) is well-factored — no parallel admission
logic, zero duplication between CLI and MCP paths, and the single-gate contract
holds. The MCP dispatch refactor (`call_tool → String`, `McpToolResult::text`
double-encoding guard) is correct and the 10 review arms are untouched.

**Findings.** One minor finding (F-1): the installed copy of
`.agents/skills/audit/SKILL.md` lacked the MCP tool preference preamble. The
authored source (`plugins/doctrine/skills/audit/SKILL.md`) was correctly
updated in commit fd87ae6d, but the install hadn't been re-run to sync the
installed copy. Fixed inline; `doctrine install` is idempotent and will
preserve the fix.

**Test evidence.** 13/13 E2E MCP tests green (memory_find+memory_list
roundtrip, min_trust suppression, show consumable/notes/backlinks, retrieve
reference to held-back error). 32/32 unit tests green covering writer capture,
limit validation, check_retrievable gate matrix, retrieve_reference,
find_for_mcp, list_for_mcp, backlink_rows_for. `cargo clippy` zero warnings.
`just check` green modulo one pre-existing flaky test (`e2e_memory_sync`,
unrelated to SL-131).

**Standing risks.** None. The implementation is structurally aligned with the
design. The sole finding was a stale installed copy, not a design deviation.

**Conscious tradeoffs.** The `held_back_on_retrieve` flag in `memory_show`
conflates consumability and trust holdback (`!consumable || held_back(...)`)
rather than reporting them separately — this is a reasonable pragmatic choice
(an un-consumable memory is definitionally held back), and the `consumable`
field separately reports which gate failed.

## Reconciliation Brief

### Per-slice (direct edit)

_No per-slice design edits needed._ The implementation conforms to design.md.

### Governance/spec (REV)

_No governance or spec changes needed._ All design decisions were correctly
implemented; no ADR, spec, or policy surfaces need updating.

## Reconciliation Outcome

All findings were resolved with fix applied inline (F-1). No design or
governance/spec writes needed.

### Direct edits applied
- `.agents/skills/audit/SKILL.md`: MCP tool preference preamble added — matches
  authored source at `plugins/doctrine/skills/audit/SKILL.md` (RV-123 F-1)

### REVs completed
_None required._

### Withdrawn / tolerated
_None._
