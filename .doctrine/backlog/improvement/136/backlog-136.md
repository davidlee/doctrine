# IMP-136: Corpus-level relation query verb

## Source

IMP-133 UX review, second pass (RF-2). See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem

The only relation read surface is `inspect <ID>` — shows outbound and
inbound edges for ONE entity. There is no verb for:

- "Show all edges of type X across the corpus" (e.g., "list every
  `governed_by` edge")
- "Show all inbound edges to target Y" (e.g., "which entities reference
  ADR-001?" — currently you must guess a likely target and run inspect on it)
- "Show entities with zero relations" (orphan detection)
- Edge type census — which labels are in use and how many edges exist

The data already exists in the catalog graph (1004 edges, fully typed).
`catalog graph` emits raw JSON but is marked "developer-facing; not gating
for acceptance." `export lazyspec` exists but emits lazyspec format, not a
human-readable relation view.

## Proposed shape

```bash
# List all governed_by edges
doctrine relation list --label governed_by

# List all governed_by edges targeting ADR-001
doctrine relation list --label governed_by --target ADR-001

# List all relations from slices
doctrine relation list --source-kind SL

# Edge census (default: active labels only)
doctrine relation census
```

The `catalog` infrastructure already builds the full graph — this is a
rendering surface over existing data, not new machinery.

## Relation to existing surfaces

- `inspect <ID>` stays as the per-entity view
- `catalog graph` stays as the developer JSON dump
- This is a human-readable, filtered corpus-level view

## Scope

MVP: `doctrine relation list --label <LABEL> [--target <REF>] [--source-kind <PREFIX>]`
Plus `doctrine relation census` for label distribution.
