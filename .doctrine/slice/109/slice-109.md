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

1. **`doctrine serve --mcp`** — a new subcommand that starts an MCP server on
   stdio, exposing review tools only (v1 scope). The server runs until the
   client disconnects; stateless between sessions.

2. **MCP tools — one per review verb:**
   - `review_new` — open a review ledger against a target entity
   - `review_list` — list reviews, filterable
   - `review_show` — show one review with derived status
   - `review_raise` — raise a finding (raiser-owned)
   - `review_dispose` — dispose a finding (responder-owned)
   - `review_verify` — verify an answered finding (raiser-owned)
   - `review_contest` — contest a disposition (raiser-owned)
   - `review_withdraw` — withdraw a finding (raiser-owned)
   - `review_status` — report derived state + baton
   - `review_prime` — seed the reviewer-context warm-cache

   Each tool's parameter schema is the typed equivalent of the existing CLI
   flags — no new semantics, no new validation.

3. **Thin wrapper architecture.** The MCP handler calls the same `review::run_*`
   functions that `main.rs` calls. No duplicated validation, state mutation, or
   entity engine logic. The MCP layer is transport + parameter unmarshalling +
   result marshalling only.

4. **Structured errors.** CLI error strings are mapped to MCP error codes with
   structured `data` payloads (e.g. finding state mismatch → code + current
   state + expected state).

5. **Session-agnostic v1.** The server is stateless across tool calls — each
   call resolves the project root from the working directory (same as the CLI
   does). No session-scoped "current review" context in v1; that is a natural
   follow-up.

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

- **RSK-001 — MCP library dependency.** Assumption: a lightweight MCP SDK crate
  exists (e.g. `mcp-server` or similar) that handles JSON-RPC framing and stdio
  transport without pulling in a heavy framework. If none is suitable, the
  protocol is simple enough to implement directly (~200 lines of serde +
  tokio codec).
- **RSK-002 — Visibility barriers.** The review verb handlers in `src/review.rs`
  are `pub(crate)`; `main.rs` already calls them directly. The MCP handler will
  live in a new module (`src/mcp_server/`) at the same crate level — no
  visibility change needed.
- **RSK-003 — Project root resolution.** The MCP server resolves the project
  root from its working directory on each call, same as the CLI. If the client
  (agent) sets cwd correctly, this is transparent. A `--path` equivalent can be
  a follow-up.
- **RSK-004 — Lock contention.** The review CLI uses per-review runtime locks
  (ADR-007 D-C4a). An MCP server with concurrent tool calls would contend on the
  same locks — same behaviour as concurrent CLI invocations; no new hazard.

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
