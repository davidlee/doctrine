# Agent-facing CLI discoverability & output format hardening

## Context

UX audit of doctrine's CLI from an agent perspective surfaced several discoverability
and format friction points that cost tokens and cause errors on every agent session.

## Scope & Objectives

### memory find: accept positional query (IMP-090)
`doctrine memory find cli` should work. Currently only `--query cli` is accepted;
a bare positional arg fails with `unexpected argument` — no hint toward `--query`.
Add a positional `<QUERY>` arg that maps to `--query`. Zero or one positional arg;
flag-only filtering still works without one. Mutually exclusive with `--query`.

### memory find/retrieve: pagination + truncation notice (IMP-091)
Add `--offset N` and `--page N` (sugar: offset = (page-1) * limit) to both
`memory find` and `memory retrieve`. When `--limit` truncates results, emit a
trailing notice: `N of M; --page <next> for next or specify a higher --limit`.
For `--json` output, the notice is suppressed (array length is its own count).

### --json on memory find (IMP-092)
`memory find` has no `--json` / `--format` flags. Add them, using the same
`listing::Format` enum and pattern as every other list/find command. JSON output
follows the `{ "kind": "memory_find", "rows": [...] }` envelope.

### doctrine status dashboard (IMP-093)
No single command answers "what is the state of the project?" in token-cheap form.
Add `doctrine status [--json]` — active slice count, open backlog by kind, blocked
items (top N), boot staleness, recent commits. 10–20 lines, token-efficient
orientation.

## Non-Goals

- `--compact` output mode (separate slice)
- `memory record --trust --severity` (IMP-081, already open)
- RSK-007 lexical sort fix (separate slice, different risk profile)
- Skill file cleanup / deduplication (separate concern)

## Summary

Four small, high-impact CLI changes that eliminate the most common agent friction
points: a positional query on `memory find`, pagination + truncation notice on
`memory find`/`memory retrieve`, consistent `--json` support on `memory find`, and
a `doctrine status` dashboard.

## Follow-Ups

- RSK-007: fix lexical sort on inbound relations at id ≥ 1000
- IMP-081: `memory record --trust --severity` flags
- `backlog tag` skill doc is stale (positional tags vs `--add`/`--remove`)
- `backlog list` default sort divergence from numeric order
