# IMP-142: Expose `related` link label for backlog entity kinds

## Summary

Backlog entity kinds (ISS, IMP, CHR, RSK, IDE) currently support only `specs`,
`slices`, and `drift` as `link`-writable labels. The `related` label — a general
cross-kind association edge — is unavailable for backlog items, forcing ad-hoc
workarounds (`drift` free text, unvalidated cross-references in prose) when a
backlog item needs to express a meaningful but non-hierarchical relationship to
another entity.

## Motivation

IDE-018 (user-definable metadata) identified five related backlog items
(IMP-108, IMP-112, IMP-118, IDE-006, IDE-013). The only available mechanism was
`drift` — free-text that doesn't validate targets and shows as an unresolved
dangler. A proper `related` edge would:

- **Validate** the target resolves to a real entity
- **Render** in `doctrine inspect` as a resolved outbound relation
- **Traverse** — `doctrine inspect` shows inbound `related` edges too
- **Feed** into the relation graph, `survey`/`next`, and the concept map

## Scope

Add `related` to the `link`-writable label set for all five backlog kinds:
ISS, IMP, CHR, RSK, IDE. The target should resolve to any entity kind (the
label is already defined as general cross-kind in the relation rules).

This is a small change to `RELATION_RULES` in `src/relation.rs` — the label
and its cross-kind target validation already exist for other source kinds
(e.g., RFC, SL, ADR, SPEC).

## Related

- **RFC-002** — the consumption surfaces program; this is a consumption surface
  for backlog items
- **IDE-018** — motivated by the inability to link backlog items via `related`
- **IMP-082** (resolved) — prior work exposing `related` for slice kinds;
  this is the backlog-kind analogue
