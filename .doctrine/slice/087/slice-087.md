# Boot snapshot token efficiency & correctness hardening

## Context

The boot snapshot (`@.doctrine/state/boot.md`, ~8.6KB) is inlined into every agent
session's system prompt. Two issues surfaced in UX audit:

1. The Memory section lists every memory (~20 entries) with full metadata (uid, type,
   status, trust, key, title) — ~50 lines of the 115-line snapshot. Most are signpost
   references; an agent pays for this index every session regardless of what subsystem
   they touch.

**Origin:** IMP-094.

IMP-095 (boot post-regeneration warning) was dropped during design — the `--check`
sentry and `/route`'s freshen-now ritual already cover the concern; a bare printed
warning without verification would be theatre.

## Scope & Objectives

### Trim the Memory section (IMP-094)
Replace the full memory listing with a reference instruction (`/retrieve-memory`)
plus a compact listing of signpost keys only (one per line, no metadata). The
`/retrieve-memory` skill already provides the richer pull path; the inlined
table is redundant for the PUSH tier.

## Non-Goals

- Changing the boot snapshot format or sections beyond Memory
- Automatic prefix refresh (requires harness-level session hook change)
- Skill description token budget (separate concern)

## Summary

Trim the Memory section from the boot snapshot (saving ~30 lines per session):
replace the full metadata table with a reference instruction (`/retrieve-memory`)
plus a compact key-only listing. High-leverage — improves every agent session
with minimal implementation risk.

## Follow-Ups

- Consider a `doctrine boot --compact` flag for projects with large memory corpora
- Audit other boot sections for bloat as the memory corpus grows
