# SL-120 Plan

## Sequencing rationale

Three phases, strictly ordered. Each phase produces a verified deliverable that
the next phase builds on.

**PHASE-01** produces the extension. It's the bulk of the work — all the MCP
protocol logic, process lifecycle, error handling. The TS output is the
authoritative template that PHASE-02 will bake into the Rust generator.

**PHASE-02** integrates the extension into the install path. It needs the final
extension content from PHASE-01 to generate byte-for-byte. This is a thin Rust
change: two new pure functions (`plan_mcp_extension`, `plan_append_system` was
SL-119's — here it's `plan_mcp_extension` and `install_mcp_extension`), one new
field on `RefreshReport`, and one new call in `install_refresh`'s pi arm.

**PHASE-03** ties it together with integration tests against a live doctrine MCP
server, plus the project gates (eslint, clippy, behaviour-preservation).

## Phase boundaries

PHASE-01 is the hardest — it ships a working extension that can be tested
standalone with `vitest`. PHASE-02 is mechanical (follows the SL-119 pattern
closely). PHASE-03 is verification-only, no new production code.

## Dependencies

- SL-119 must be implemented or at least designed (for the `install_refresh`
  pi arm pattern). SL-120 extends it — `plan_mcp_extension` mirrors
  `plan_pi_extension`. The two can be developed in parallel if the SL-119 design
  is locked and the `install_refresh` pi arm shape is known.
- The extension depends on doctrine's MCP server (`serve --mcp`). No changes
  to the MCP server are needed — it's the stable dependency.
