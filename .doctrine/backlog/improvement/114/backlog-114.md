# IMP-114: review_list lean-by-default: cap MCP output to a sane default with a non-silent total/truncated signal

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Follow-up to IMP-113 #3. That landed an **opt-in** `limit` on `review_list`,
deliberately leaving the default uncapped to avoid a surprising behaviour change.
But the default is the leak: the corpus is 101 RVs today (~9KB JSON per `{}`
call) and grows linearly. An agent that doesn't think to pass `limit` eats the
whole list. Lean-by-default is the right posture for an agent-facing surface.

The constraint (boot: **no silent caps** — "log what was dropped"): a default cap
must tell the caller the result is partial, or it reads as "here is everything"
when it isn't.

## Sketch

1. **Default cap.** `review_list` applies a default `limit` (propose **50**) when
   the caller passes none. Explicit `limit` overrides (raise or lower). An escape
   hatch for "all" — propose `limit: 0` ⇒ unbounded (0 rows is never a useful
   request, so the sentinel is free), or an explicit `all: true`.

2. **Non-silent signal.** When the cap truncates, surface the pre-truncation
   count so the omission is visible. Add `total: Option<usize>` to
   `ReviewOutput::Listed`, `#[serde(skip_serializing_if = "Option::is_none")]`
   — set MCP-side only when `rows.len()` was capped; `None` (absent) otherwise, so
   uncapped lists and the CLI path (`print_review`, field access) are unchanged.
   The MCP arm computes `total` before truncating.

3. **Order question (decide in design).** Today `limit` keeps the *first* N rows
   (RV-001…, oldest). For a default cap, the *most recent* N (highest RV-N) is
   almost always what an agent wants. Options: (a) reverse/tail for the default
   cap only — inconsistent with explicit `limit`; (b) make `review_list`
   newest-first across the board — cleaner, but a visible ordering change to the
   MCP surface (CLI table ordering is a separate decision — keep it). Lean toward
   (b) for the MCP wire, confirm at design.

## Scope / non-goals

- MCP-local. The shared `listing` engine and CLI ordering stay untouched
  (behaviour-preservation gate) — same posture as IMP-113.
- Not a pagination/offset cursor. A single default cap + total signal is enough
  for the agent-facing read; full pagination is a separate, larger want if it
  ever arises.

## Verification (when built)

- `review_list {}` returns ≤ 50 rows and carries `total: 101` (or current count).
- `review_list {limit: 200}` returns all, no `total` field (not truncated).
- `review_list {limit: 0}` (or `all`) returns all, no `total`.
- Uncapped/CLI `print_review` output byte-unchanged (no `total` leak).
- Ordering per the §3 decision is asserted (newest-first if (b)).
