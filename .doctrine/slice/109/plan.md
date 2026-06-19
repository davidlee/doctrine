# Implementation Plan SL-109: MCP server for review commands

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Four ordered phases, each independently testable and committed on completion.
The gating constraint is the structured return types — no MCP work can begin
until the review engine speaks in `ReviewOutput` variants, and no verb handler
refactor can land until the types exist.

## Sequencing & Rationale

### PHASE-01 → PHASE-02

**Types before refactor.** PHASE-01 adds `ReviewOutput` and `ReviewError` enums
plus the generic `with_turn<T>` parameter — all additive, zero behavioural
change. Existing tests pass because nothing calls the new types yet. This
keeps the `with_turn` change (one line, one generic parameter) isolated from
the 11-verb refactor, making each commit reviewable in a single sitting.

### PHASE-02 → PHASE-03

**Engine before server.** PHASE-02 is the behavioural cut: every `run_*`
returns `ReviewOutput` instead of writing stdout directly. `print_review()`
preserves CLI behaviour identically. Golden tests (VH-1) prove it. The MCP
server (PHASE-03) can then call `run_*` and get structured data to serialise —
the exact shape the design promises. Without PHASE-02, the MCP server would
need to parse stdout strings (the very heresy F-1 condemns).

### PHASE-03 → PHASE-04

**Integration last.** PHASE-03 delivers a functional MCP server with unit tests
for protocol types, transport framing, tool dispatch, and error mapping.
PHASE-04 spawns the real binary as a subprocess and drives the full protocol,
catching integration-level bugs (stdio framing edge cases, process lifecycle,
real disk state) that unit tests miss.

### Why four phases, not three or five

- **Three phases** would merge PHASE-01 + PHASE-02 (~400 changed lines + 150
  new lines). Too large for a single reviewable commit.
- **Five phases** would split the MCP server into protocol/transport and
  tools — but the two are tightly coupled (tool dispatch needs the protocol
  types; the serve loop needs tool dispatch). `src/mcp_server/` is ~300 lines
  total; splitting it into two phases creates artificial boundaries with
  no independent test value.
- **Four phases** gives each phase a clear test: types → unit, refactor →
  golden, server → unit + handler, integration → end-to-end.

## Phase Dependency Graph

```
PHASE-01 (types) ──► PHASE-02 (refactor) ──► PHASE-03 (server) ──► PHASE-04 (integration)
```

Linear, no parallelism. Each phase depends strictly on the previous.

## Key Decisions

- **`with_turn` generic in PHASE-01, not PHASE-02.** The signature change is a
  one-line diff (`()` → `T`, add generic parameter). It belongs with the types
  it enables, not the verb handlers that use it.
- **Golden tests in PHASE-02, not PHASE-04.** Behavioural preservation of CLI
  output is a refactor concern — it must pass before any MCP work touches
  `run_*`. Integration tests (PHASE-04) validate the MCP protocol, not CLI
  output parity.
- **`ReviewError` in PHASE-01, not PHASE-03.** The error enum is part of the
  review engine's type surface, not the MCP server's. MCP error mapping
  (PHASE-03) is a consumer of `ReviewError`, not its definer.
- **Zero new crates throughout.** Hand-rolled MCP protocol, manual
  `Display`/`From` impls for `ReviewError`. The design mandates no new
  dependencies; all phases honour this.

## Notes

- The decomposition of `src/review.rs` into submodules (design D7) is a
  separate follow-up slice. This plan does not include it.
- Session-scoped review context, other command suites, MCP resources/prompts,
  and HTTP transport are deferred per the slice scope's non-goals.
- The `--mcp` flag design (D6) allows future serve modes without breaking
  the interface; no future-proofing beyond the flag gate is needed now.
