# Boot snapshot token efficiency & correctness hardening

## Context

The boot snapshot (`@.doctrine/state/boot.md`, ~8.6KB) is inlined into every agent
session's system prompt. Two issues surfaced in UX audit:

1. The Memory section lists every memory (~20 entries) with full metadata (uid, type,
   status, trust, key, title) — ~50 lines of the 115-line snapshot. Most are signpost
   references; an agent pays for this index every session regardless of what subsystem
   they touch.
2. `doctrine boot` regenerates the snapshot but the already-inlined prefix stays stale
   until session restart. Agents editing governance may think `doctrine boot` has
   refreshed their context.

**Origin:** IMP-094, IMP-095.

## Scope & Objectives

### Trim the Memory section (IMP-094)
Replace the full memory listing with a single reference line:
`Run 'doctrine memory retrieve' to surface relevant memories for your task.`
The `/retrieve-memory` skill already wraps this; the inlined index is redundant.
If discoverability is a concern, list only the signpost keys (one line each) without
full metadata.

### Boot post-regeneration warning (IMP-095)
`doctrine boot` should emit: `Snapshot regenerated. Restart session or /clear for
changes to take effect.` The `--check` sentry already exists but isn't sufficient —
agents need the warning at regeneration time, not just a later check.

## Non-Goals

- Changing the boot snapshot format or sections beyond Memory
- Automatic prefix refresh (requires harness-level session hook change)
- Skill description token budget (separate concern)

## Summary

Two small changes: trim the Memory section from the boot snapshot (saving ~50 lines
per session), and warn on `doctrine boot` that the inlined prefix is stale. Both are
high-leverage — they improve every agent session with minimal implementation risk.

## Follow-Ups

- Consider a `doctrine boot --compact` flag for projects with large memory corpora
- Audit other boot sections for bloat as the memory corpus grows
