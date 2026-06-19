# pi dispatch worker integration via RPC mode

## Context

The `/dispatch` subprocess arm (`dispatch-subprocess/SKILL.md`) currently
documents only `codex exec` as the spawn backend, despite ADR-011 D3 placing pi
in the same codex/pi column with the same capability altitude. The pi coding
agent harness is already installed in the project jail and shares all the
subprocess affordances (disk marker, env delivery, base-pinned fork, per-wt build
isolation, nested bwrap). Documenting and exercising the pi integration is
low-cost — the CLI contract (`doctrine worktree fork --worker`) is
harness-identical — and belongs in doctrine's own repo as a live integration
example of the harness-agnostic spawn interface.

pi differs from codex in one material way that unlocks post-hoc worker
digestibility: its **RPC mode** (`pi --mode rpc`) provides a structured JSONL
protocol with events, state queries, and an `agent_end` completion signal —
making it straightforward for the orchestrator to extract only the outcome
(success/failure, diff, last message) rather than feeding raw streaming output
into context. This is an optimisation-tier enhancement, never a required
element; the contract floor is pi as a subprocess spawn.

The token-efficiency concern is real at dispatch scale: a single worker run
streaming raw JSONL events produces 15-30K+ tokens of intermediate output before
the `agent_end` event. For multi-worker batches this compounds. A thin post-hoc
digest extracting outcome, diff, and final message reduces that to ~200 tokens
per worker — worthwhile but deferred: v1 captures raw `agent_end` messages and
layers the digest if token pressure demands it.

## Scope & Objectives

1. **Document pi RPC mode spawn** in `dispatch-subprocess/SKILL.md` alongside the
   existing `codex exec` spawn template, with framing for both fire-and-forget
   (print mode `-p`) and interactive (RPC mode) postures.

2. **End-to-end exercise:** run a real dispatch phase through a pi RPC worker in
   a forked worktree, validating the full spawn-to-funnel cadence (fork →
   marker → spawn → agent_end → import → commit).

3. **Session hygiene:** workers run under `--no-session` (ephemeral) or
   `--session-dir` pointing into the fork directory (so worker sessions are
   colocated and GC-able with the fork). Decision TBD at design.

4. **Post-hoc digest (deferred to follow-up):** if raw `agent_end` JSON proves
   too token-heavy for the orchestrator, a thin filter extracting status, diff
   summary, and final message — buildable as a trivial pipeline, not a custom
   extension.

## Non-Goals

- Building a pi extension or MCP bridge — pi's RPC mode is the integration
  surface.
- Changing the orchestrator funnel — the fork/marker/import/commit cadence is
  harness-identical (ADR-011 D2).
- Adding pi to the Claude `Agent` arm (`dispatch-agent`) — pi is a subprocess
  harness only.
- A full worker-output filtering extension — v1 accepts raw output; a filter is
  deferred and may never be needed.
- Modifying `doctrine worktree fork` or any CLI verb — the existing contract
  already serves pi.
- Session persistence or resume across handover — worker sessions are ephemeral
  by design.

## Summary

A thin integration: one documented spawn template in the existing
`dispatch-subprocess/SKILL.md`, one end-to-end exercise proving the RPC
worker → funnel cadence, and a follow-up ticket for the output digest if
scale demands it.

## Follow-Ups

- **Post-hoc digest:** if token pressure from raw `agent_end` JSONL becomes
  material, build a thin pipeline extracting (status, diff summary, final
  message). Track as a backlog item post-exercise.
