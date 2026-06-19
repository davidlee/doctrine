# MCP server for review commands

## Context

The `doctrine review` CLI has a turn-based state machine spanning seven verbs
(`new`, `raise`, `dispose`, `verify`, `contest`, `withdraw`, `status`) with
role-enforced transitions, per-finding state gating, and required flags with
closed-vocabulary enums (`--severity`, `--disposition`). Agents driving this
protocol via bash must hold the state machine in their "mind" (via skill prose),
rediscover exact flag shapes on each invocation, and parse human-readable output
for the next move.

An MCP (Model Context Protocol) server — stdio transport, JSON-RPC — would let
the server hold the protocol state, expose only valid-next-step tools, enforce
transitions at the protocol level, and return structured responses. This is a
thin wrapper over the existing `src/review.rs` pure core and verb handlers; the
server delegates to the same engine, never duplicates it.

ADR-011 established the pattern for "harness-agnostic interface, per-harness
capability altitude." The review MCP server is the same shape: one contract (the
review engine), delivered through a new transport (MCP/stdio) that agents can
discover and use natively. The CLI remains the source of truth — the MCP server
calls into the same functions.

## Scope & Objectives

1. **Refactor `run_*` return types** — each review verb handler returns a
   `ReviewOutput` variant enum instead of writing to `io::stdout()`. This is the
   gating change: the review engine produces structured data; callers format it.
   `with_turn` becomes generic over closure return type (`T` instead of `()`).
   CLI output preserved behaviorally via `print_review()` in `main.rs`.

2. **`doctrine serve --mcp [--path]`** — a new subcommand that starts an MCP
   server on stdio, exposing review tools only (v1 scope). The server runs until
   the client disconnects; resolves project root once at startup.

3. **MCP tools — one per review verb:**
   - `review_new` — open a review ledger against a target entity
   - `review_list` — list reviews, filterable (returns structured rows)
   - `review_show` — show one review with derived status (returns JSON + formatted)
   - `review_raise` — raise a finding (raiser-owned)
   - `review_dispose` — dispose a finding (responder-owned)
   - `review_verify` — verify an answered finding (raiser-owned)
   - `review_contest` — contest a disposition (raiser-owned)
   - `review_withdraw` — withdraw a finding (raiser-owned)
   - `review_status` — report derived state + baton + cache verdict
   - `review_prime` — seed the reviewer-context warm-cache

   Each tool's parameter schema is the typed equivalent of the existing CLI
   flags, including `--as` role overrides. No new semantics, no new validation.

4. **Thin wrapper architecture.** The MCP handler calls the same `review::run_*`
   functions that `main.rs` calls. No duplicated validation, state mutation, or
   entity engine logic. The MCP layer is transport + parameter unmarshalling +
   result marshalling only.

5. **Structured errors.** Review engine errors are mapped to MCP error responses
   with typed `data` payloads (`NOT_FOUND`, `ROLE_MISMATCH`, `STATE_MISMATCH`).

6. **Hand-rolled MCP protocol** — zero new crate dependencies. The tools-only
   MCP surface (initialize, tools/list, tools/call) is ~300 lines of
   serde + tokio codec. The project already has `serde`, `serde_json`, `tokio`.

## Non-Goals

- **Other doctrine commands** (memory, slice, backlog, etc.) — review only.
- **HTTP transport** — stdio is the MCP standard; HTTP can be a follow-up if
  needed.
- **Stateful sessions** — v1 is stateless-per-call. Session-scoped review
  context (remembering "current review RV-017") is deferred.
- **Duplicating the review engine** — the server calls existing `pub(crate)`
  functions; if visibility barriers exist, they are lifted via `pub(crate)`, not
  by copying logic.
- **MCP resources or prompts** — tools only. Resources (exposing review state as
  readable documents) and prompts (templated review workflows) are deferred.
- **Streaming/progress** — each tool call is request/response; no partial
  results or progress notifications.

## Risks & Assumptions

- **RSK-001 — Hand-rolled protocol correctness.** The MCP JSON-RPC framing is
  implemented directly (~300 lines). Mitigation: integration tests (VH-3, VH-4)
  validate the full protocol handshake and tool round-trips against real MCP
  client expectations.
- **RSK-002 — `Deserialize` on `Severity`/`Facet`.** These enums use custom
  `parse` methods. A `#[serde(deserialize_with)]` bridge forwards to the
  existing parser — no change to the enum representation.
- **RSK-003 — Baton CAS under batch mutation.** Multiple MCP tool calls in rapid
  sequence contend on the same per-review lock (ADR-007 D-C4a). Same behaviour
  as concurrent CLI invocations; verified in execute phase with an agent test
  run (VH-5).
- **RSK-004 — `PathBuf` in serialized JSON.** `ReviewOutput::Created.dir` is a
  `PathBuf` — serialises as a string. Acceptable for MCP transport; the path is
  relative to the project root.

## Summary

A `doctrine serve --mcp` command that exposes the review verb suite as MCP tools
over stdio. Each tool delegates to the existing review engine. The agent gets
typed tool discovery, protocol-level transition enforcement, and structured
errors — without the server duplicating a line of review logic.

## Follow-Ups

- **Session-scoped review context** — remember "current review" across tool calls
- **Other command suites** — memory, slice, backlog as MCP tools
- **MCP resources** — expose review state as readable documents
- **MCP prompts** — templated review workflows (open+prime, raise+dispose, etc.)
- **HTTP transport** — for non-stdio MCP clients
