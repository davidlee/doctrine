Source: <https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan>

## Why this record exists

`ADR-011`'s per-harness capability altitude (D3) rests its entire claude column
on one premise, stated in its Context:

> claude's `Agent` tool has no env channel, and `claude -p` is
> Anthropic-API-billed (not subscription) and harness-specific — so neither an
> env seam nor a subprocess spawn can be a *required* element of a
> harness-agnostic framework. **For claude the only viable backend is the
> in-session `Agent` tool**, which exposes no env seam and no exec wrapper.

The billing half of that premise is **false as of this record**. `claude -p`
draws from subscription usage limits.

## What the falsification reaches

Four incumbent mechanisms exist only because the claude worker could not be a
subprocess:

| mechanism | why it exists | under `claude -p` |
|---|---|---|
| disk marker as sole worker identity | `Agent` has no env channel (D2, D4) | `--setenv DOCTRINE_WORKER 1` in the bwrap prefix |
| `SubagentStart` stamp hook | the marker needs a writer and `Agent` has no exec wrapper (D3) | orchestrator stamps before exec, as on pi |
| `worktree pretooluse` confinement wall | D3: *"OS confinement — none: `Agent` is not a subprocess to wrap"* | nested bwrap, as on pi |
| gated `worker_commit` MCP tool | a hook-confined in-session worker cannot self-commit | a clone has a writable `.git` |

`ADR-011` D4 lists three enhancements as "codex/pi-only until a free claude env
backend lands". A subprocess spawn **is** that backend, and takes all three at
once.

## The provisionality is the point

This is a **pause**, not a settled position — Anthropic says it is working to
update the plan and will give notice before anything takes effect. Two
consequences, and the second is what makes it safe to build on:

1. The premise could move again. A design that is only justified by current
   billing is hostage to it.
2. It should not be justified by billing alone. The subprocess arm is the
   better shape independently: it is uniform across all three harnesses, it
   reaches real OS confinement rather than a fail-open hook wall, and it
   deletes mechanism rather than adding it. Billing removed the *blocker*; it
   is not the *reason*. Owner's position, 2026-08-12: not likely to change tack
   even if billing flips back.

Advance notice means the window is warned, not sudden.

## Downstream

- Corrects the `ADR-011` Context premise and its D3 table's claude column —
  routes through a `REV` per `ADR-013`.
- Dissolves `SL-247` (see [[DEC-152]], [[DEC-154]]), which patched the
  confinement wall's fourth arm.
- Firms `RFC-025`'s recorded target: *"uniform sandboxed subprocess workers
  (`claude -p` / codex / pi) rather than in-session subagents."*
