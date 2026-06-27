# SL-164 Implementation plan

## Rationale

The work has one unavoidable dependency chain: the engine must accept a `writer`
parameter before the MCP tools can use it to capture output. Everything else
— tool definitions, dispatch, boot footer, cleanup — can follow naturally.

Three phases:

**PHASE-01** is the smallest blast-radius change: two function signatures and
~36 call sites. Doing it first keeps the behaviour-preservation gate narrow —
all existing tests must stay green with only the writer parameter added. No
MCP logic enters yet.

**PHASE-02** adds all three MCP tools in one pass. They share the same file
(`tools.rs`) and test surface, so splitting them would create merge friction for
no benefit. The onboard tool is a simple markdown renderer; the memory write
tools follow the established review write-tool pattern exactly. VT-2 count
bump happens here since the tool list changes.

**PHASE-03** is pure cleanup: boot footer prose + stale artifact renames. No
logic changes. Could merge into PHASE-02 but separated for audit clarity —
the boot footer is authored prose, not generated code, and merits its own
review step (VH-1).

### Verification strategy

- **VT** (test): all functional behaviour checked by unit + E2E tests
- **VA** (agent): onboard markdown output reviewed for correctness
- **VH** (human): boot footer wording reviewed

No phase carries a `VA` or `VH` where a test could judge — the onboard tool's
markdown structure can be asserted by test (VT), but the semantic correctness
of the mapping table and memory bodies benefits from an agent sanity-check (VA).
